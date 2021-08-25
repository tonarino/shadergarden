use std::collections::BTreeMap;

use lexpr::Value;

use crate::{
    graph::{
        NodeId,
        ShaderGraph,
    },
    lisp::Val,
};

pub type FnDef = (Vec<String>, Vec<Value>);

#[derive(Debug)]
pub struct Scope<T> {
    items: Vec<BTreeMap<String, T>>,
}

impl<T> Scope<T> {
    pub fn new() -> Self {
        Scope {
            items: vec![BTreeMap::new()],
        }
    }

    pub fn get(&self, name: &str) -> Result<&T, String> {
        for item in self.items.iter().rev() {
            if let Some(item) = item.get(name) {
                return Ok(item);
            }
        }

        Err(format!("Item `{}` is not defined", name))
    }

    pub fn set(&mut self, name: String, item: T) {
        self.items.last_mut().unwrap().insert(name, item);
    }

    pub fn enter_scope(&mut self) { self.items.push(BTreeMap::new()) }

    pub fn exit_scope(&mut self) -> BTreeMap<String, T> {
        self.items.pop().unwrap()
    }
}

pub type ExternalFn =
    Box<dyn Fn(&mut ShaderGraph, &[NodeId]) -> Result<NodeId, String>>;
pub type External = BTreeMap<String, ExternalFn>;

// Note that functions are not first class.
// We're trying to describe a graph,
// not make a turing complete language ;)
pub struct Env {
    /// Maps name to value, separate from function scope.
    vars:      Scope<Val>,
    /// Maps name to args, body.
    functions: Scope<FnDef>,
    /// Maps shader name to shader source.
    shaders:   BTreeMap<String, String>,
    /// Maps names to rust functions that construct
    /// subgraphs.
    external:  External,
}

impl std::fmt::Debug for Env {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Env")
            .field("vars", &self.vars)
            .field("functions", &self.functions)
            .field("shaders", &self.shaders.keys().collect::<Vec<&String>>())
            .field("external", &self.external.keys().collect::<Vec<&String>>())
            .finish()
    }
}

impl Env {
    pub fn new(shaders: BTreeMap<String, String>, external: External) -> Env {
        Env {
            vars: Scope::new(),
            functions: Scope::new(),
            shaders,
            external,
        }
    }

    pub fn get(&self, name: &str) -> Result<&Val, String> {
        self.vars.get(name)
    }

    pub fn set(&mut self, name: String, item: Val) { self.vars.set(name, item) }

    pub fn get_fn(&self, name: &str) -> Result<&FnDef, String> {
        self.functions.get(name)
    }

    pub fn set_fn(&mut self, name: String, item: FnDef) {
        self.functions.set(name, item)
    }

    pub fn enter_scope(&mut self) {
        self.vars.enter_scope();
        self.functions.enter_scope();
    }

    pub fn exit_scope(&mut self) {
        self.vars.exit_scope();
        self.functions.exit_scope();
    }

    pub fn shader(&self, name: &str) -> Result<&String, String> {
        self.shaders.get(name).ok_or(format!(
            "Could not load shader `{}`, it is not defined",
            name
        ))
    }

    pub fn external(&self, name: &str) -> Result<&ExternalFn, String> {
        self.external.get(name).ok_or(format!(
            "Could not load external function `{}`, it is not defined",
            name
        ))
    }
}
