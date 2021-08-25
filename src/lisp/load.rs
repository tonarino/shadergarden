use std::{
    collections::BTreeMap,
    fs,
    path::Path,
};

/// Loads a directory of shaders into a big ol' `BTreeMap`.
pub fn load_shaders<T>(path: T) -> Result<BTreeMap<String, String>, String>
where
    T: AsRef<Path>,
{
    let mut map = BTreeMap::new();
    let files = fs::read_dir(path)
        .map_err(|_| "Could not read shader directory".to_string())?;

    for p in files {
        // some type stuff, you know the deal
        let path = p.map_err(|_| "Got a bad file path".to_string())?.path();

        // only include `.frag` files
        if path.is_dir() {
            continue;
        }
        match path.extension() {
            Some(x) if x.to_str() == Some("frag") => (),
            _ => {
                continue;
            },
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
        map.insert(name, contents);
    }

    Ok(map)
}
