use glium::{
    uniforms::{
        AsUniformValue,
        UniformValue,
    },
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

pub type ComputeNodeFn<T> = Box<dyn FnMut(UniformMap) -> T>;

/// Represents a compute node, i.e. a node that runs on the
/// GPU. Still operates on uniforms, takes a single input.
pub struct ComputeNode<T: AsUniformValue> {
    pub func:   ComputeNodeFn<T>,
    pub input:  NodeId,
    pub output: T,
}

impl<T: AsUniformValue> std::fmt::Debug for ComputeNode<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComputeNode")
            .field("input", &self.input)
            .finish()
    }
}

impl<T: AsUniformValue> Node for ComputeNode<T> {
    fn inputs(&self) -> Vec<NodeId> { vec![self.input] }

    fn outputs(&self) -> (&str, UniformValue) {
        ("compute", self.output.as_uniform_value())
    }

    fn texture(&self) -> Option<&Texture2d> { None }

    fn forward(&mut self, _rect_strip: &RectStrip, uniforms: UniformMap<'_>) {
        self.output = (self.func)(uniforms);
    }
}
