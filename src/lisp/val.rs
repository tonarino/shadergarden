use crate::graph::NodeId;

#[derive(Debug, Clone)]
pub enum Val {
    Node(NodeId),
    Number(f64),
    Bool(bool),
    String(String),
}

impl Val {
    pub fn to_node(&self) -> Result<NodeId, String> {
        match self {
            Val::Node(n) => Ok(*n),
            other => Err(format!(
                "Type mismatch: expected a Node, found `{:?}`",
                other
            )),
        }
    }

    pub fn to_nat(&self) -> Result<usize, String> {
        match self {
            Val::Number(u) => Ok(*u as usize),
            other => Err(format!(
                "Type mismatch: expected a Number, found `{:?}`",
                other
            )),
        }
    }

    pub fn to_float(&self) -> Result<f64, String> {
        match self {
            Val::Number(u) => Ok(*u),
            other => Err(format!(
                "Type mismatch: expected a Number, found `{:?}`",
                other
            )),
        }
    }

    pub fn to_string(&self) -> Result<String, String> {
        match self {
            Val::String(s) => Ok(s.to_string()),
            Val::Number(f) => Ok(format!("{}", f)),
            other => Err(format!(
                "Type mismatch: expected a String, found `{:?}`",
                other
            )),
        }
    }

    pub fn to_bool(&self) -> Result<bool, String> {
        match self {
            Val::Bool(b) => Ok(*b),
            other => Err(format!(
                "Type mismatch: expected a Boolean, found `{:?}`",
                other
            )),
        }
    }
}
