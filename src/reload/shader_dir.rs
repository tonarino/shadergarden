use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs,
    path::Path,
};

use include_dir::{
    include_dir,
    Dir,
};

pub const BASE_PROJECT: Dir = include_dir!("./demos/base");

/// Represents a directory of shaders, and a shader graph
/// lisp configuration file.
pub struct ShaderDir {
    pub lisp:    String,
    pub shaders: BTreeMap<String, String>,
}

impl ShaderDir {
    /// Creates a new `ShaderDir` from component parts.
    pub fn new(
        lisp_graph: String,
        shaders: BTreeMap<String, String>,
    ) -> ShaderDir {
        ShaderDir {
            lisp: lisp_graph,
            shaders,
        }
    }

    // TODO: abstract and combine following new methods

    /// Creates a `ShaderDir` from a directory included at
    /// compile time. Note that `lisp_graph` must be a
    /// parsable lisp expression, not a path.
    pub fn new_from_included(
        dir: Dir,
        lisp_graph: String,
    ) -> Result<ShaderDir, String> {
        let mut shaders = BTreeMap::new();
        for file in dir.files() {
            if file.path().is_dir()
                || file.path().extension() != Some(OsStr::new("frag"))
            {
                continue;
            }

            // get the key and value, insertomundo!
            let name = file
                .path()
                .file_stem()
                .ok_or("Could not infer shader name from file name")?
                .to_str()
                .ok_or("Could not convert file name to UTF8 string")?
                .to_string();
            let contents = String::from_utf8(file.contents().to_vec())
                .map_err(|_| "Could not get shader contents")?;
            shaders.insert(name, contents);
        }

        Ok(ShaderDir {
            lisp: lisp_graph,
            shaders,
        })
    }

    /// Creates a new `ShaderDir` from a directory.
    pub fn new_from_dir<T>(path: T, get_lisp : impl Fn() -> Result<String, String>) -> Result<ShaderDir, String>
    where
        T: AsRef<Path>,
    {

        let lisp = get_lisp()?;

        let mut shaders = BTreeMap::new();
        let files = fs::read_dir(path)
            .map_err(|_| "Could not read shader directory".to_string())?;

        for p in files {
            // some type stuff, you know the deal
            let path = p.map_err(|_| "Got a bad file path".to_string())?.path();

            // only include `.frag` files
            if path.is_dir() || path.extension() != Some(OsStr::new("frag")) {
                continue;
            }

            // get the key and value, insertomundo!
            let name = path
                .file_stem()
                .ok_or("Could not infer shader name from file name")?
                .to_str()
                .ok_or("Could not convert file name to UTF8 string")?
                .to_string();
            let contents = fs::read_to_string(path)
                .map_err(|_| "Could not get shader contents")?;
            shaders.insert(name, contents);
        }

        Ok(ShaderDir { lisp, shaders })
    }
}