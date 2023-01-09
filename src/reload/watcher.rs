use std::{
    path::{
        Path,
        PathBuf,
    },
    rc::Rc,
    sync::{
        atomic::{
            AtomicBool,
            Ordering,
        },
        Arc,
    },
    time::{
        Duration,
        Instant,
    },
    thread,
    sync::mpsc::{Sender, Receiver},
    sync::mpsc,
    io,
    io::BufRead,
    fs,
};

use glium::backend::Context;

use notify::{
    RecommendedWatcher,
    RecursiveMode,
    Watcher,
};

#[cfg(target_family = "unix")]
use signal_hook::{consts::SIGUSR1, iterator::Signals};

use crate::{
    graph::ShaderGraph,
    lisp::graph_from_sexp,
    map,
    reload::ShaderDir,
};

/// A struct that watches a directory for changes,
/// and hot-reloads a shader graph if changes have been
/// made.
pub struct ShaderGraphWatcher {
    context:      Rc<Context>,
    last_reload:  Instant,
    path:         PathBuf,
    config:       PathBuf,
    changed:      Arc<AtomicBool>,
    _watcher:     RecommendedWatcher,
    shader_graph: ShaderGraph,
    stdin_rx:    Option<Receiver<String>>
}

pub enum WatchResult {
    /// No changes were made.
    NoChange,
    /// Changes were made and the graph was rebuilt.
    Rebuilt,
    /// Changes were made but the graph could not be
    /// rebuilt.
    Err(String),
}

impl ShaderGraphWatcher {
    /// Creates a new watcher over a certain dir.
    /// Returns an error if the directory could not be
    /// loaded, Or the graph could not be built.
    pub fn new_watch_dir<T>(
        context: &Rc<Context>,
        path: T,
        config: T,
    ) -> Result<ShaderGraphWatcher, String>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();
        let config = config.as_ref().to_path_buf();

        let changed = Arc::new(AtomicBool::new(false));

        // build the watcher
        let mut watcher = RecommendedWatcher::new({
            let changed = changed.clone();
            move |res| match res {
                Ok(_) => changed.store(true, Ordering::SeqCst),
                Err(e) => println!("[warn] Watch error: `{:?}`.", e),
            }
        })
        .unwrap();
        watcher.watch(&path, RecursiveMode::Recursive).unwrap();

        // SIGUSR handling thread
        #[cfg(target_family = "unix")]
        {
            let signals = Signals::new(&[SIGUSR1]);
            match signals {
                Ok(mut s) => {
                        let changed = changed.clone();
                        thread::spawn(move || {
                            for sig in s.forever() {
                                changed.store(true, Ordering::SeqCst);
                                println!("[info] Received signal {:?}", sig);
                            }
                        });
                    }
                Err(e) => println!("[warn] Signal listen error: `{:?}`.", e)
            };
        }
        
        //initial build
        let shader_graph = ShaderGraphWatcher::build_initial(context, &path, &config)?;
        let last_reload = Instant::now();

        // STDIN reading thread
        let stdin_rx = match config.to_str().unwrap() {
            "-" => {
                let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
                {
                    // let thread_tx = tx.clone();
                    let changed = changed.clone();
                    thread::spawn(move || {
                        loop {
                            println!("[info] Reading config from STDIN");
                            let maybe_config = read_stdin_config();
                            match maybe_config {
                                Ok(c) => {
                                    println!("[info] STDIN config received, sending to receiver");
                                    tx.send(c).unwrap();
                                    changed.store(true, Ordering::SeqCst);
                                },
                                Err(e) => {
                                    println!("[warn] STDIN config read error: `{:?}`.", e);
                                    return;
                                },
                            }
                        }
                    });
                }
                Some(rx)
            },
            _  => None,
        };

        Ok(ShaderGraphWatcher {
            context: context.clone(),
            last_reload,
            path,
            config,
            changed,
            _watcher: watcher,
            shader_graph,
            stdin_rx: stdin_rx
        })
    }

    // TO-DO: reduce some code duplication here (build => build_initial AND build_reload)
    //Refactor with helper methods, maybe in a separate file.
    pub fn build_initial(
        context: &Rc<Context>,
        path: &Path,
        config: &Path,
    ) -> Result<ShaderGraph, String> {
        let shader_dir = match config.to_str().unwrap() {
            "-" => 
                ShaderDir::new_from_dir(path, || {
                    read_stdin_config().map_err(|s| {
                        format!("Could not read config from stdin: {}", s)
                    })
                })?,
            cfg => 
                ShaderDir::new_from_dir(path, || {
                    fs::read_to_string(&config).map_err(|_| {
                        format!("Could not read `{}` in shader directory", cfg)
                    })
                })?,
        };
        let shader_graph = graph_from_sexp(context, shader_dir, map! {})?;
        Ok(shader_graph)
    }

    fn build_reload(
        context: &Rc<Context>,
        path: &Path,
        config: &Path,
        stdin_rx: &Option<Receiver<String>>,
    ) -> Result<ShaderGraph, String> {
        let shader_dir = match stdin_rx {
            Some(rx) => 
                ShaderDir::new_from_dir(path, || {
                    rx.recv().map_err(|s| {
                        format!("Could not read config from stdin: {}", s)
                    })
                })?,
            None => 
                ShaderDir::new_from_dir(path, || {
                    fs::read_to_string(&config).map_err(|_| {
                        format!("Could not read `{}` in shader directory", 
                            config.to_str().unwrap()
                        )
                    })
                })?,
        };
        let shader_graph = graph_from_sexp(context, shader_dir, map! {})?;
        Ok(shader_graph)
    }

    /// Gets the shader graph without trying to reload
    /// Note that `graph` will only reload when needed,
    /// And tries to de-duplicate redundant reloads,
    /// So only use this for fine-grained control over
    /// reloads.
    pub fn graph_no_reload(&mut self) -> &mut ShaderGraph {
        &mut self.shader_graph
    }

    /// Forces a rebuild of the graph. Do not call this in a
    /// loop! As with `graph_no_reload`, only use this
    /// for fine-grained control over reloads.
    pub fn graph_force_reload(&mut self) -> (&mut ShaderGraph, WatchResult) {
        let watch_result = match ShaderGraphWatcher::build_reload(
            &self.context,
            &self.path,
            &self.config,
            &self.stdin_rx,
        ) {
            Ok(graph) => {
                self.shader_graph = graph;
                WatchResult::Rebuilt
            },
            Err(error) => WatchResult::Err(error),
        };

        self.last_reload = Instant::now();
        (&mut self.shader_graph, watch_result)
    }

    /// Reloads a shader graph if there have been changes,
    /// And the graph hasn't been rebuilt recently.
    /// Note that if compilation fails, the old graph will
    /// remain in use. Returns a borrowed `ShaderGraph`,
    /// and whether the graph was rebuilt.
    pub fn graph(&mut self) -> (&mut ShaderGraph, WatchResult) {
        if self.last_reload.elapsed() > Duration::from_millis(300)
            && self.changed.swap(false, Ordering::SeqCst)
        {
            self.graph_force_reload()
        } else {
            (self.graph_no_reload(), WatchResult::NoChange)
        }
    }
}

fn read_stdin_config() -> Result<String, String> {
    let mut byte_vec: Vec<u8> = Vec::new();
    let stdin = io::stdin(); // We get `Stdin` here.
    
    {
        let mut handle = stdin.lock();

        let mut count = 0; //the number of open parentheses with no corresponding closing
        let mut start = 0;

        //consume ONE s-expression
        loop {
            let bytes_read = handle.read_until(b')', &mut byte_vec);
            match bytes_read {
                Ok(0) => {
                    return Err("No bytes were read (STDIN closed?)".to_string());
                },
                Ok(_) if *byte_vec.last().unwrap() != b')' => {
                    return Err("Last byte read not ')' (STDIN closed?)".to_string());
                },
                Ok(read) => {
                    //count the number of opening parenthesis in read bytes
                    // println!("Number of bytes read: {}", read);
                    // println!("Last byte read: {}", *(byte_vec.last().unwrap()) as char);
                    let len = byte_vec.len();
                    for i in len-read..len {
                        if b'(' == byte_vec[i] {
                            count += 1;
                        }
                    };                 
                },
                Err(err) => {
                    return Err(format!("Reading STDIN error: {}", err));
                },
            };

            //subtract 1 since we reached a closing parenthesis
            count -= 1;

            if count < 0 {
                return Err("Unbound s-expression error".to_string());
            } else if count > 0 {
                continue; //s-expression not finished
            } else {
                //check if s-expression is an output one
                if did_read_output_sexpr(&byte_vec, start) {
                    break;
                } else {
                    start = byte_vec.len() - 1;
                }
            }
        } 
    }

    //All s-expressions read, convert to string
    let is_valid_utf8 = std::str::from_utf8(&byte_vec);
    match is_valid_utf8 {
        Ok(the_str) => Ok(the_str.to_string()),
        Err(err) => Err(format!("Parsing STDIN utf8 error: {}", err)),
    }
}

fn did_read_output_sexpr(bytes : &Vec<u8>, start : usize) -> bool {
    let mut i = start;

    //skip everything before initial '('
    while bytes[i] != b'(' {
        i += 1;
    }

    //skip '('
    i += 1;

    //find index past initial '(' and past all whitespace
    while (bytes[i] as char).is_whitespace() {
        i += 1;
    }

    //check if the rest of the bytes Vec has enough space for 'output', if not, false
    if bytes.len() - i < 6 {
        return false;
    }
    
    //convert slice to str and use equality comparison
    let maybe_str = std::str::from_utf8(&bytes[i..i+6]);
    match maybe_str {
        Ok(s)  => s == "output",
        Err(_) => false, 
    }
}