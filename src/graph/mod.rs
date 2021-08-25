use std::{
    collections::BTreeMap,
    rc::Rc,
    time::Instant,
};

use glium::{
    backend::Context,
    uniforms::AsUniformValue,
    Texture2d,
};

use crate::util::{
    compile_shader,
    default_buffer,
    RectStrip,
};

mod uniform;
mod shader_node;
mod compute_node;
mod node;

pub use compute_node::{
    ComputeNode,
    ComputeNodeFn,
};
pub use node::Node;
pub use shader_node::{
    Buffer,
    ShaderNode,
};
use uniform::UniformMap;

// TODO: remove the distinction between uniforms and
// textures as inputs

/// Handle that represents a particular node,
/// in the context of a shader graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(usize);

/// Represents a Directed Acyclic Graph of shaders.
/// Each shader is run sequentially, and can be the input to
/// shaders later down the line.
pub struct ShaderGraph {
    context:    Rc<Context>,
    rect_strip: RectStrip,
    created:    std::time::Instant,

    /// None is an input node.
    nodes: Vec<Option<Box<dyn Node>>>,

    // TODO: use sets?
    inputs:  Vec<NodeId>,
    outputs: Vec<NodeId>,
}

impl std::fmt::Debug for ShaderGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaderGraph")
            .field("nodes", &self.nodes)
            .field("inputs", &self.inputs)
            .field("outputs", &self.outputs)
            .finish()
    }
}

impl ShaderGraph {
    /// Creates a new shader graph within a specific glium
    /// context.
    pub fn new(context: &Rc<Context>) -> ShaderGraph {
        ShaderGraph {
            context:    context.clone(),
            rect_strip: RectStrip::new(context),
            nodes:      vec![],
            inputs:     vec![],
            outputs:    vec![],
            created:    Instant::now(),
        }
    }

    pub fn get_inputs(&self) -> &Vec<NodeId> { &self.inputs }

    pub fn get_outputs(&self) -> &Vec<NodeId> { &self.outputs }

    /// Adds anything that implements the `Node` trait to
    /// the graph Will panic if the `Node` does not
    /// preserve DAG structure.
    pub fn add_node(&mut self, node: Option<Box<dyn Node>>) -> NodeId {
        if let Some(ref node) = node {
            self.assert_dag(&node.inputs())
        }

        self.nodes.push(node);
        NodeId(self.nodes.len() - 1)
    }

    fn assert_dag(&self, nodes: &[NodeId]) {
        // to preserve acyclic structure, can only ref backwards
        for NodeId(input) in nodes {
            assert!(input < &self.nodes.len());
        }
    }

    /// Adds an input to the shader graph.
    /// Use the returned `NodeId` to mark it as a
    /// texture input to other shaders.
    pub fn add_input(&mut self) -> NodeId {
        let id = self.add_node(None);
        self.inputs.push(id);
        id
    }

    // TODO: custom uniforms!
    /// Adds a shader to a shader graph.
    /// Each shader has its own underlying `Texture2d` of a
    /// particular size. Shader is bound to the graph's
    /// context.
    pub fn add_shader(
        &mut self,
        source: &str,
        inputs: Vec<NodeId>,
        width: u32,
        height: u32,
    ) -> Result<NodeId, String> {
        self._add_shader(
            source,
            inputs,
            Buffer::Single(default_buffer(&self.context, width, height)),
        )
    }

    /// Add a recurrent shader to the graph.
    pub fn add_rec_shader(
        &mut self,
        source: &str,
        inputs: Vec<NodeId>,
        width: u32,
        height: u32,
    ) -> Result<NodeId, String> {
        // set up the shader and its buffers
        self._add_shader(
            source,
            inputs,
            Buffer::new_double(|| default_buffer(&self.context, width, height)),
        )
    }

    fn _add_shader(
        &mut self,
        source: &str,
        inputs: Vec<NodeId>,
        buffer: Buffer,
    ) -> Result<NodeId, String> {
        let shader = compile_shader(&self.context, source)?;

        let shader_node = ShaderNode {
            shader,
            inputs,
            buffer,
        };
        Ok(self.add_node(Some(Box::new(shader_node))))
    }

    /// Adds a compute node, which produces
    /// a set of uniforms for use in the next shader.
    pub fn add_compute<T: AsUniformValue + 'static>(
        &mut self,
        compute_node: ComputeNode<T>,
    ) -> Result<NodeId, String> {
        Ok(self.add_node(Some(Box::new(compute_node))))
    }

    /// Mark a node in the graph as an output.
    /// When calling `forward`, this node's texture will be
    /// included in the output map. To access it, index
    /// the map with the `NodeId` this method returns.
    /// Returns `None` if the requested `Node` does not
    /// support being an output.
    pub fn mark_output(&mut self, id: NodeId) -> Option<NodeId> {
        if let Some(node) = &self.nodes[id.0] {
            node.texture()?;
        }

        if !self.outputs.contains(&id) {
            self.outputs.push(id)
        }

        Some(id)
    }

    fn time(created: Instant) -> f32 {
        ((Instant::now() - created).as_millis() as f64 / 1000.0) as f32
    }

    fn build_inputs<'a>(
        mut uniforms: UniformMap<'a>,
        previous: &'a [Option<Box<dyn Node>>],
        inputs: &'a [NodeId],
        input_map: &'a BTreeMap<NodeId, &'a Texture2d>,
    ) -> UniformMap<'a> {
        // TODO: decide what other default uniforms to use
        // let mut uniforms = UniformMap::new();

        for input in inputs.iter() {
            match &previous[input.0] {
                Some(node) => {
                    let (kind, uniform_value) = node.outputs();
                    uniforms.add(kind, uniform_value);
                },
                None => {
                    uniforms
                        .add("texture", input_map[input].as_uniform_value());
                },
            };
        }

        uniforms
    }

    fn pull_outputs<'a>(
        &'a self,
        input_map: BTreeMap<NodeId, &'a Texture2d>,
    ) -> BTreeMap<NodeId, &'a Texture2d> {
        // pull all output textures
        // note that an input is fair game to be pulled as an output
        // texture
        let mut output_map = BTreeMap::new();
        for id in self.outputs.iter() {
            let texture = match &self.nodes[id.0] {
                // unwrap: checked before insertion
                Some(node) => node.texture().unwrap(),
                None => input_map[id],
            };
            output_map.insert(*id, texture);
        }

        output_map
    }

    // TODO: represent inputs as actual nodes, not Nones

    /// Does a forward pass of the entire shader graph.
    /// Takes a set of `N` inputs, and produces a set of `M`
    /// outputs. Each shader in the DAG is run, from
    /// front to back, previous input textures are bound
    /// as uniforms of the form: `u_texture_0, ..,
    /// u_texture_n`. Use the `map!` macro to quickly
    /// build a `BTreeMap` to pass to this function. All
    /// marked outputs will be included in the output map.
    pub fn forward<'a>(
        &'a mut self,
        input_map: BTreeMap<NodeId, &'a Texture2d>,
    ) -> BTreeMap<NodeId, &'a Texture2d> {
        // ensure all input textures have been passed in
        // use btreemap because `inputs` is small
        // and node ids are cheap to compare
        for input in self.inputs.iter() {
            assert!(input_map.contains_key(input));
        }

        for split_index in 0..self.nodes.len() {
            // this is a DAG, so we can only ever reference
            // previous nodes from the current one
            // we split here so we can have multiple mutible borrows.
            let (previous, current) = self.nodes.split_at_mut(split_index);

            if let Some(ref mut node) = current[0] {
                let mut uniforms = UniformMap::new();
                let time = Self::time(self.created);
                uniforms.add("time", time.as_uniform_value());

                let inputs = node.inputs();
                let uniforms = Self::build_inputs(
                    uniforms, &*previous, &inputs, &input_map,
                );

                node.forward(&self.rect_strip, uniforms);
            }
        }

        // pulls and returns all the output textures
        self.pull_outputs(input_map)
    }
}
