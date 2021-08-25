use std::collections::BTreeMap;

use glium::uniforms::{
    UniformValue,
    Uniforms,
};

#[derive(Default)]
pub struct UniformMap<'a>(BTreeMap<String, Vec<UniformValue<'a>>>);

impl<'a> UniformMap<'a> {
    /// Create a new empty `UniformMap`.
    pub fn new() -> UniformMap<'a> { UniformMap(BTreeMap::new()) }

    /// Inserts a kind of uniform into the uniform map.
    /// Will automatically number the item `u_{kind}_N`,
    /// starting at `N = 0`.
    /// Returns the index of the item, i.e. `N`.
    pub fn add(&mut self, kind: &str, uniform: UniformValue<'a>) -> usize {
        if !self.0.contains_key(kind) {
            self.0.insert(kind.to_string(), vec![]);
        }

        let uniforms = self.0.get_mut(kind).unwrap();
        uniforms.push(uniform);
        uniforms.len() - 1
    }

    /// Get a specific kind of uniform at a given index.
    pub fn get(&self, kind: &str, index: usize) -> Option<&UniformValue<'a>> {
        self.0.get(kind)?.get(index)
    }

    /// Get all uniforms of a given kind.
    pub fn get_kind_all(&self, kind: &str) -> Option<&Vec<UniformValue<'a>>> {
        self.0.get(kind)
    }

    /// Join two maps by appending one to the other.
    /// This works by appending all lists of the same kind
    /// together, With `self` being first, and `other`
    /// being second.
    pub fn append(&mut self, other: Self) {
        for (key, values) in other.0.into_iter() {
            for value in values {
                self.add(&key, value);
            }
        }
    }
}

impl<'a> Uniforms for UniformMap<'a> {
    fn visit_values<'b, F: FnMut(&str, UniformValue<'b>)>(
        &'b self,
        mut output: F,
    ) {
        for (kind, uniforms) in self.0.iter() {
            // if there is only one uniform of a kind,
            // no subscript is required.
            // this lets us have `u_time` instead of `u_time_0`,
            // but if an input adds another `time` uniform,
            // the user *must* disambiguiate which one is intended.
            if uniforms.len() == 1 {
                output(&format!("u_{}", kind), uniforms[0]);
            }

            for (index, uniform) in uniforms.iter().enumerate() {
                output(&format!("u_{}_{}", kind, index), *uniform)
            }
        }
    }
}
