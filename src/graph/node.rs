use glium::{
    uniforms::UniformValue,
    Texture2d,
};

use crate::{
    graph::{
        NodeId,
        UniformMap,
    },
    util::RectStrip,
};

/// Represents a generalized shader in a shader graph.
/// Implement this trait to add arbitrary nodes to the
/// shader graph.
pub trait Node: std::fmt::Debug {
    /// Not a fan, requires allocation.
    fn inputs(&self) -> Vec<NodeId>;

    /// Returns (kind, uniforms) tuple.
    fn outputs(&self) -> (&str, UniformValue);

    // TODO: remove texture in favor of `outputs`?
    /// Denotes whether the node produces an output texture.
    /// This function should be consistent,
    /// e.g. if it returns `Some` it should always return
    /// `Some`.
    fn texture(&self) -> Option<&Texture2d>;

    // TODO: should I pass a rect strip or a context?
    // I can build a rect strip from a context, but that takes
    // time. Is the performance hit worth the generalized
    // interface? Is there a performance hit?
    // Should rect strip have a reference to a Rc<Context>?
    fn forward(&mut self, rect_strip: &RectStrip, uniforms: UniformMap);
}
