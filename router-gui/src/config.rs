use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use include_dir::{include_dir, Dir};

// Include the web-gui/build directory at compile time
static WEB_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/web-gui/build");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlAssets {
    pub filename: String,
    pub content: String,
}

pub fn init() -> HashMap<String, Vec<u8>> {
    let mut assets: HashMap<String, Vec<u8>> = HashMap::new();
    
    // Process all files in the embedded directory
    process_dir(&WEB_DIR, "", &mut assets);
    
    assets
}

// Recursively process directories and files
fn process_dir(dir: &Dir, path_prefix: &str, assets: &mut HashMap<String, Vec<u8>>) {
    for entry in dir.entries() {
        match entry {
            include_dir::DirEntry::Dir(subdir) => {
                let new_prefix = if path_prefix.is_empty() {
                    format!("/{}", subdir.path().file_name().unwrap().to_string_lossy())
                } else {
                    format!("{}/{}", path_prefix, subdir.path().file_name().unwrap().to_string_lossy())
                };
                process_dir(subdir, &new_prefix, assets);
            },
            include_dir::DirEntry::File(file) => {
                let file_name = file.path().file_name().unwrap().to_string_lossy();
                let full_path = if path_prefix.is_empty() {
                    format!("/{}", file_name)
                } else {
                    format!("{}/{}", path_prefix, file_name)
                };
                
                // Read file content as bytes directly
                let content = file.contents();
                assets.insert(full_path, content.to_vec());
            }
        }
    }
}
