use glium::{
    uniforms::{
        AsUniformValue,
        UniformValue,
    },
    Program,
    Surface,
    Texture2d,
};

use crate::{
    graph::{
        node::Node,
        NodeId,
        UniformMap,
    },
    util::RectStrip,
};

pub enum Buffer {
    Single(Texture2d),
    Double(Texture2d, Texture2d),
}

impl Buffer {
    pub fn new_double<F: Fn() -> Texture2d>(f: F) -> Buffer {
        Buffer::Double(f(), f())
    }

    pub fn front(&self) -> &Texture2d {
        match self {
            Buffer::Single(ref texture) => texture,
            Buffer::Double(ref front, _back) => front,
        }
    }
    pub fn back(&self) -> Option<&Texture2d> {
        match self {
            Buffer::Single(_) => None,
            Buffer::Double(_front, back) => Some(back),
        }
    }

    pub fn swap(&mut self) {
        // Using `Box`, the swap is cheap.
        if let Buffer::Double(ref mut front, ref mut back) = self {
            std::mem::swap(front, back);
        }
    }
}

// TODO: Remove `pub` on struct fields

/// Represents a single shader, the inputs it expects,
/// and the texture it owns that is updated in each forward
/// pass.
pub struct ShaderNode {
    pub shader: Program,
    pub inputs: Vec<NodeId>,
    pub buffer: Buffer,
}

impl std::fmt::Debug for ShaderNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaderGraph")
            .field("inputs", &self.inputs)
            .finish()
    }
}

impl Node for ShaderNode {
    fn inputs(&self) -> Vec<NodeId> { self.inputs.to_owned() }

    fn outputs(&self) -> (&str, UniformValue) {
        match self.buffer {
            Buffer::Single(ref texture) => {
                ("texture", texture.as_uniform_value())
            },
            Buffer::Double(ref texture, _) => {
                ("texture", texture.as_uniform_value())
            },
        }
    }

    fn texture(&self) -> Option<&Texture2d> { Some(self.buffer.front()) }

    fn forward(&mut self, rect_strip: &RectStrip, uniforms: UniformMap) {
        self.buffer.swap();

        let front = self.buffer.front();
        let resolution =
            [front.get_width() as f32, front.get_height().unwrap() as f32];

        let mut uniforms = uniforms;
        uniforms.add("resolution", UniformValue::Vec2(resolution));
        if let Some(back) = self.buffer.back() {
            uniforms.add("previous", back.as_uniform_value());
        }

        // taking the previous inputs,
        // render out the next texture using a shader,
        // overwriting its previous contents.
        front
            .as_surface()
            .draw(
                &rect_strip.buffer,
                &rect_strip.indices,
                &self.shader,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
    }
}
