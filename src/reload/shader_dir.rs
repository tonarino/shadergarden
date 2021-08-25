use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs,
    path::Path,
};

use include_dir::Dir;

/// Represents a directory of shaders, and a shader graph
/// lisp configuration file.
pub struct ShaderDir {
    pub lisp:    String,
    pub shaders: BTreeMap<String, String>,
}

impl ShaderDir {
    /// Creates a new `ShaderDir` from component parts.
    pub fn new(lisp: String, shaders: BTreeMap<String, String>) -> ShaderDir {
        ShaderDir { lisp, shaders }
    }

    // TODO: abstract and combine following new methods

    pub fn new_from_included(
        dir: Dir,
        lisp: String,
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

        Ok(ShaderDir { lisp, shaders })
    }

    /// Creates a new `ShaderDir` from a directory.
    pub fn new_from_dir<T>(path: T, config: T) -> Result<ShaderDir, String>
    where
        T: AsRef<Path>,
    {
        let lisp = fs::read_to_string(&config).map_err(|_| {
            format!(
                "Could not read `{}` in shader directory",
                config.as_ref().to_str().unwrap()
            )
        })?;

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
