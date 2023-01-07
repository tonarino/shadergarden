use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs,
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
use shadergarden::{
    png,
    reload,
    util,
    reload::watcher::ShaderGraphWatcher
};
use structopt::{
    clap::AppSettings,
    StructOpt,
};

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

#[derive(StructOpt, Debug)]
struct New {
    #[structopt(default_value = ".", parse(from_os_str = package_dir))]
    project: PathBuf,
}

#[derive(StructOpt, Debug)]
struct Run {
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

#[derive(StructOpt, Debug)]
struct Render {
    #[structopt(flatten)]
    run:    Run,
    #[structopt(short, long)]
    output: PathBuf,
    /// Starting frame
    #[structopt(short, long, default_value = "0")]
    start:  u64,
    /// Ending frame
    #[structopt(short, long, default_value = "150")]
    end:    u64,
    #[structopt(long, default_value = "30")]
    fps:    f64,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Shader Garden", bin_name = "shadergarden", about, global_settings(&[AppSettings::ColoredHelp, AppSettings::DeriveDisplayOrder]))]
enum Cli {
    New(New),
    Run(Run),
    Render(Render),
}

/// Main function
/// Runs the shader graph, and displays the output in the
/// window. If changes are detected, we rebuild the shader
/// graph and swap it out.
fn main() {
    // parse arguments and extract
    let args = Cli::from_args();

    match args {
        Cli::Run(r) => run(r),
        Cli::Render(r) => render(r),
        Cli::New(n) => new(n.project),
    }
}

fn new(path: PathBuf) {
    fs::create_dir_all(&path).unwrap();
    if let Ok(mut dir) = fs::read_dir(&path) {
        if dir.next().is_some() {
            eprintln!("[fatal] Specified project path is not empty");
            panic!();
        }
    }

    if reload::BASE_PROJECT.extract(&path).is_err() {
        eprintln!("[fatal] Could not create base project");
        panic!();
    }

    eprintln!(
        "[info] successfully created new project in `{}`",
        path.display()
    );
}

// TODO: factor out common parts of render and run

fn render(render: Render) {
    let args = render.run;
    let lisp_config = args
        .graph
        .to_owned()
        .unwrap_or_else(|| args.project.join("shader.graph"));
    let inputs = args.inputs;

    // set up the main event loop
    let (event_loop, display) = util::create(
        "Shader Garden Renderer".to_string(),
        args.width as f64,
        args.height as f64,
    );

    // set up hot code reloading
    let mut graph = ShaderGraphWatcher::build_initial(display.get_context(), &args.project, &lisp_config).unwrap();

    eprintln!("[info] Built initial graph");

    // build a table of textures
    #[cfg(feature = "ffmpeg")]
    let mut input_textures =
        util::input_textures(&display, &inputs, args.width, args.height);

    #[cfg(not(feature = "ffmpeg"))]
    assert!(inputs.is_empty(), "Inputs are not supported when running without ffmpeg");

    eprintln!("[info] Starting Render...");

    let mut frame_number = 0;
    let frames_output = render.output;
    let frame_start = render.start;
    let frame_end = render.end;
    let frame_nanos = (1000000000.0 / render.fps) as u64;

    event_loop.run(move |event, _, mut control_flow| {
        // waits until next frame, keep at top
        *control_flow = wait_nanos(0);
        handle_event(event, &mut control_flow);

        // get the input and output handles
        let input_nodes = graph.get_inputs();
        let output = if let [output] = graph.get_outputs().as_slice() {
            *output
        } else {
            eprintln!("[fatal] Graph has invalid output signature.");
            panic!();
        };

        #[cfg(not(feature = "ffmpeg"))]
        assert!(input_nodes.is_empty());

        #[cfg(feature = "ffmpeg")]
        assert_eq!(
            input_nodes.len(),
            input_textures.len(),
            "The number of graph inputs and provided textures does not match up",
        );

        // render the shader graph, display the primary output
        #[allow(unused_mut)]
        let mut input_map = BTreeMap::new();

        #[cfg(feature = "ffmpeg")]
        for (node_id, texture) in input_nodes.iter().zip(input_textures.iter_mut()) {
            input_map.insert(*node_id, texture.next_frame());
        }

        // dumb hack to make the playback smooth(er)
        graph.created = std::time::Instant::now()
            - std::time::Duration::from_nanos(frame_nanos * frame_number);
        let output_map = graph.forward(input_map);

        // set up the draw target and draw
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        let texture = output_map[&output];
        util::texture(&display, &mut target, texture);
        target.finish().unwrap();

        if frame_number >= frame_start {
            png::write_png(texture, &frames_output.join(
                format!("frame-{:0>4}.png", frame_number - frame_start)
            ));
        }
        if frame_number > frame_end {
            panic!("Finished render, bailing pathetically");
        }
        frame_number += 1
    });
}

fn run(args: Run) {
    let lisp_config = args
        .graph
        .to_owned()
        .unwrap_or_else(|| args.project.join("shader.graph"));
    let inputs = args.inputs;

    // set up the main event loop
    let (event_loop, display) = util::create(
        "Shader Garden Playground".to_string(),
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
    #[cfg(feature = "ffmpeg")]
    let mut input_textures =
        util::input_textures(&display, &inputs, args.width, args.height);

    #[cfg(not(feature = "ffmpeg"))]
    assert!(inputs.is_empty(), "Inputs are not supported when running without ffmpeg");

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

        #[cfg(not(feature = "ffmpeg"))]
        assert!(input_nodes.is_empty());

        #[cfg(feature = "ffmpeg")]
        assert_eq!(
            input_nodes.len(),
            input_textures.len(),
            "The number of graph inputs and provided textures does not match up",
        );

        // render the shader graph, display the primary output
        #[allow(unused_mut)]
        let mut input_map = BTreeMap::new();

        #[cfg(feature = "ffmpeg")]
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
