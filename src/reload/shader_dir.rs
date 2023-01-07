use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs,
    io,
    path::Path,
    io::BufRead,
};

use include_dir::{
    include_dir,
    Dir,
};

pub const BASE_PROJECT: Dir = include_dir!("./demos/base");

/// Represents a directory of shaders, and a shader graph
/// lisp configuration file.
pub struct ShaderDir {
    pub lisp:    String,
    pub shaders: BTreeMap<String, String>,
}

impl ShaderDir {
    /// Creates a new `ShaderDir` from component parts.
    pub fn new(
        lisp_graph: String,
        shaders: BTreeMap<String, String>,
    ) -> ShaderDir {
        ShaderDir {
            lisp: lisp_graph,
            shaders,
        }
    }

    // TODO: abstract and combine following new methods

    /// Creates a `ShaderDir` from a directory included at
    /// compile time. Note that `lisp_graph` must be a
    /// parsable lisp expression, not a path.
    pub fn new_from_included(
        dir: Dir,
        lisp_graph: String,
    ) -> Result<ShaderDir, String> {
        let mut shaders = BTreeMap::new();
        for file in dir.files() {
            if file.path().is_dir()
                || file.path().extension() != Some(OsStr::new("frag"))
            {
                continue;
            }

            // get the key and value, insertomundo!
            let name = file
                .path()
                .file_stem()
                .ok_or("Could not infer shader name from file name")?
                .to_str()
                .ok_or("Could not convert file name to UTF8 string")?
                .to_string();
            let contents = String::from_utf8(file.contents().to_vec())
                .map_err(|_| "Could not get shader contents")?;
            shaders.insert(name, contents);
        }

        Ok(ShaderDir {
            lisp: lisp_graph,
            shaders,
        })
    }

    /// Creates a new `ShaderDir` from a directory.
    pub fn new_from_dir<T>(path: T, config: T) -> Result<ShaderDir, String>
    where
        T: AsRef<Path>,
    {
        let config_str = config.as_ref().to_str().unwrap();

        let lisp = //custom stdin s-expression consuming function here 
            if config_str == "-" {
                read_stdin_config().map_err(|s| {
                    format!("Could not read config from stdin: {}", s)
                })?
            } else {
                fs::read_to_string(&config).map_err(|_| {
                    format!(
                        "Could not read `{}` in shader directory",
                        config_str
                    )
                })?
            };

        let mut shaders = BTreeMap::new();
        let files = fs::read_dir(path)
            .map_err(|_| "Could not read shader directory".to_string())?;

        for p in files {
            // some type stuff, you know the deal
            let path = p.map_err(|_| "Got a bad file path".to_string())?.path();

            // only include `.frag` files
            if path.is_dir() || path.extension() != Some(OsStr::new("frag")) {
                continue;
            }

            // get the key and value, insertomundo!
            let name = path
                .file_stem()
                .ok_or("Could not infer shader name from file name")?
                .to_str()
                .ok_or("Could not convert file name to UTF8 string")?
                .to_string();
            let contents = fs::read_to_string(path)
                .map_err(|_| "Could not get shader contents")?;
            shaders.insert(name, contents);
        }

        Ok(ShaderDir { lisp, shaders })
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
                Ok(read) => {
                    //count the number of opening parenthesis in read bytes
                    let len = byte_vec.len();
                    for i in len-read..len {
                        if b'(' == byte_vec[i] {
                            count += 1;
                        }
                    };                 
                },
                Err(err) => return Err(format!("Reading STDIN error: {}", err)),
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