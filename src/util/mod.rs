use std::rc::Rc;

use glium::{
    backend::{
        Context,
        Facade,
    },
    glutin::{
        dpi::LogicalSize,
        event_loop::EventLoop,
        window::WindowBuilder,
        ContextBuilder,
    },
    implement_vertex,
    index::NoIndices,
    texture::{
        MipmapsOption,
        UncompressedFloatFormat,
    },
    uniform,
    uniforms::{
        MagnifySamplerFilter,
        Sampler,
    },
    Display,
    Frame,
    Program,
    Surface,
    Texture2d,
    VertexBuffer,
};

#[derive(Copy, Clone)]
pub struct Vertex {
    position:   [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

pub struct RectStrip {
    pub buffer:  VertexBuffer<Vertex>,
    pub indices: NoIndices,
}

impl RectStrip {
    /// Builds a triangle strip and a vertex buffer
    /// used to render basically everything.
    pub fn new<F: ?Sized + Facade>(facade: &F) -> RectStrip {
        let mut shape = vec![];
        for a in [0.0, 1.0] {
            for b in [0.0, 1.0] {
                shape.push(Vertex {
                    position:   [a * 2. - 1., b * 2. - 1.],
                    tex_coords: [a, b],
                });
            }
        }

        let buffer = VertexBuffer::new(facade, &shape).unwrap();
        let indices = NoIndices(glium::index::PrimitiveType::TriangleStrip);

        RectStrip { buffer, indices }
    }
}

/// Makes a window on which to display something.
/// Note that width and height are specified according to
/// [`LogicalSize`]. Fore more fine-grained control, use the
/// individual components.
pub fn create_window(title: String, width: f64, height: f64) -> WindowBuilder {
    WindowBuilder::new()
        .with_inner_size(LogicalSize::new(width, height))
        .with_title(title)
}

/// Sets up a window on which to draw.
/// Note that this returns a simple [`EventLoop`] and
/// [`Display`].
pub fn create(
    title: String,
    width: f64,
    height: f64,
) -> (EventLoop<()>, Display) {
    let event_loop = EventLoop::new();
    let wb = create_window(title, width, height);
    let cb = ContextBuilder::new();
    let display =
        Display::new(wb, cb, &event_loop).expect("Unable to create display");
    (event_loop, display)
}

/// Writes an output texture to the entire window.
pub fn texture(display: &Display, target: &mut Frame, texture: &Texture2d) {
    let rect_strip = RectStrip::new(display);
    let vertex_shader_src = include_str!("./texture.vert");
    let fragment_shader_src = include_str!("./texture.frag");

    let program = glium::Program::from_source(
        display,
        vertex_shader_src,
        fragment_shader_src,
        None,
    )
    .unwrap();

    target
        .draw(
            &rect_strip.buffer,
            &rect_strip.indices,
            &program,
            &uniform! {
                tex: Sampler::new(texture)
                    .magnify_filter(MagnifySamplerFilter::Nearest),
            },
            &Default::default(),
        )
        .unwrap();
}

pub fn default_buffer(
    context: &Rc<Context>,
    width: u32,
    height: u32,
) -> Texture2d {
    // set up the shader and its buffer
    Texture2d::empty_with_format(
        context,
        UncompressedFloatFormat::U16U16U16U16, // 4 16-bit color channels
        MipmapsOption::NoMipmap,
        width,
        height,
    )
    .unwrap()
}

pub fn compile_shader(
    context: &Rc<Context>,
    source: &str,
) -> Result<Program, String> {
    Program::from_source(context, include_str!("./texture.vert"), source, None)
        .map_err(|e| format!("{}", e))
}
