use std::collections::HashMap;

use mini_config::Configure;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Configure)]
pub enum BuildHtml {
    Content,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlAssets {
    pub filename: String,
    pub content: String,
}

// Change the return type to use Vec<u8> instead of String
pub fn init() -> HashMap<String, Vec<u8>> {
    let project_path = std::env::current_dir().unwrap();
    let path = project_path.join("router-gui/web-gui/build");
    println!("project_path: {:?}", path);

    let mut all_paths = Vec::new();

    if path.exists() {
        all_paths = visit_dirs(&path).unwrap_or_default();
    }

    let str_path = path.to_str().unwrap_or_default();

    let assets: HashMap<String, Vec<u8>> = all_paths
        .iter()
        .map(|p| {
            let clean = p.clone().replace(str_path, "");
            (
                clean.to_string(),
                std::fs::read(p).unwrap_or_default(), // Use read() instead of read_to_string()
            )
        })
        .collect();

    assets
}

fn visit_dirs(dir: &std::path::Path) -> std::io::Result<Vec<String>> {
    let mut paths = Vec::new();
    paths.push(dir.display().to_string());

    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let sub_paths = visit_dirs(&path)?;
            paths.extend(sub_paths);
        }
    }

    Ok(paths)
}
