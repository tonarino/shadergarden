use std::{
    collections::BTreeMap,
    ffi::OsStr,
    path::PathBuf,
    time::{
        Duration,
        Instant,
    },
};

use glium::{
    backend::Facade,
    glutin::{
        event::{
            Event,
            WindowEvent,
        },
        event_loop::ControlFlow,
    },
    Surface,
};
use shadergraph::{
    input::FrameStream,
    reload,
    util,
};
use structopt::StructOpt;

/// Waits a specific number of nanoseconds before rendering
/// the next frame. Returns a control_flow that should be
/// set in the event loop. This is based on the current
/// time, so must be recalculated every frame.
fn wait_nanos(nanos: u64) -> ControlFlow {
    let wait = Instant::now() + Duration::from_nanos(nanos);
    ControlFlow::WaitUntil(wait)
}

/// Simple event handler that quits when the window is
/// closed. Must be called from within the event loop.
pub fn handle_event(event: Event<()>, control_flow: &mut ControlFlow) {
    if let Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
    } = event
    {
        *control_flow = ControlFlow::Exit;
    }
}

pub fn dir(path: &OsStr) -> PathBuf {
    if path == "." {
        std::env::current_dir().expect("Can not determine package directory")
    } else {
        PathBuf::from(path)
    }
}

pub fn package_dir(path: &OsStr) -> PathBuf {
    if path == "." {
        std::env::current_dir().expect("Can not determine package directory")
    } else {
        PathBuf::from(path)
    }
}

// #[derive(StructOpt, Debug)]
// struct New {
//     #[structopt(default_value = ".", parse(from_os_str =
// package_dir))]     project: PathBuf,
// }

#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(default_value = ".", parse(from_os_str = package_dir))]
    project: PathBuf,
    #[structopt(short, long)]
    graph:   Option<PathBuf>,
    #[structopt(short, long, default_value = "512")]
    width:   u32,
    #[structopt(short, long, default_value = "512")]
    height:  u32,
    #[structopt(short, long)]
    inputs:  Vec<PathBuf>,
}

// #[derive(StructOpt, Debug)]
// #[structopt(name = "Shader Graph", bin_name =
// "shadergraph", about,
// global_settings(&[AppSettings::ColoredHelp,
// AppSettings::DeriveDisplayOrder]))] enum Cli {
//     New(New),
//     Run(Run),
// }

/// Main function
/// Runs the shader graph, and displays the output in the
/// window. If changes are detected, we rebuild the shader
/// graph and swap it out.
fn main() {
    // parse arguments and extract
    let args = Cli::from_args();

    let lisp_config = args
        .graph
        .to_owned()
        .unwrap_or_else(|| args.project.join("shader.graph"));
    let inputs = args.inputs;

    // set up the main event loop
    let (event_loop, display) = util::create(
        "Shader Graph Playground".to_string(),
        args.width as f64,
        args.height as f64,
    );

    // set up hot code reloading
    let mut watcher = reload::ShaderGraphWatcher::new_watch_dir(
        display.get_context(),
        args.project,
        lisp_config,
    )
    .map_err(|e| {
        eprintln!("[fatal] Could not build initial graph:");
        eprintln!("{}", e);
        panic!();
    })
    .unwrap();
    eprintln!("[info] Built initial graph");

    // build a table of textures
    let mut input_textures = vec![];
    for texture_path in inputs {
        eprintln!(
            "[info] Building frames for {}",
            texture_path.to_string_lossy()
        );
        input_textures.push(
            FrameStream::new(
                &texture_path,
                args.width as u32,
                args.height as u32,
                &display,
            )
            .expect("Couldn't open frame source"),
        );
    }

    eprintln!("[info] Starting...");

    event_loop.run(move |event, _, mut control_flow| {
        // waits until next frame, keep at top
        *control_flow = wait_nanos(16_666_667);
        handle_event(event, &mut control_flow);

        // get the graph, notify if updated
        let (graph, watch_result) = watcher.graph();
        match watch_result {
            reload::WatchResult::NoChange => (),
            reload::WatchResult::Rebuilt => eprintln!("[info] Graph rebuilt"),
            reload::WatchResult::Err(e) => {
                eprintln!("[warn] Could not rebuild graph:");
                eprintln!("{}", e);
            }
        }

        // get the input and output handles
        let input_nodes = graph.get_inputs();
        let output = if let [output] = graph.get_outputs().as_slice() {
            *output
        } else {
            eprintln!("[fatal] Graph has invalid output signature.");
            panic!();
        };
        assert_eq!(
            input_nodes.len(),
            input_textures.len(),
            "The number of graph inputs and provided textures does not match up",
        );

        // render the shader graph, display the primary output
        let mut input_map = BTreeMap::new();
        for (node_id, texture) in input_nodes.iter().zip(input_textures.iter_mut()) {
            input_map.insert(*node_id, texture.next_frame());
        }
        let output_map = graph.forward(input_map);

        // set up the draw target and draw
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        util::texture(&display, &mut target, output_map[&output]);
        target.finish().unwrap();
    });
}
