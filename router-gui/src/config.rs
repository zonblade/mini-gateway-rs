use mini_config::Configure;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Configure)]
pub enum HtmlTemplate {
    GlobCSSAssets,
    GlobJSAssets,
    GlobLayout,
    GlobHead,
    Home,
    Login,
    SettingProxy,
    SettingGwNode,
    SettingGateway,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlAssets {
    pub filename: String,
    pub content: String,
}

pub fn init(){
    let mut css_assets: Vec<HtmlAssets> = vec![];
    let mut js_assets: Vec<HtmlAssets> = vec![];
    let mut html_assets: Vec<HtmlAssets> = vec![];

    // check if /tmp/gwrs/html/assets exists
    let asset_path = "/tmp/gwrs/html/assets";
    if !std::path::Path::new(asset_path).exists() {
        std::fs::create_dir_all(asset_path).expect("Failed to create asset directory");
    }
    // check if /tmp/gwrs/html/assets/css exists
    let css_path = format!("{}/css", asset_path);
    if !std::path::Path::new(&css_path).exists() {
        std::fs::create_dir_all(&css_path).expect("Failed to create css directory");
    }
    // check if /tmp/gwrs/html/assets/js exists
    let js_path = format!("{}/js", asset_path);
    if !std::path::Path::new(&js_path).exists() {
        std::fs::create_dir_all(&js_path).expect("Failed to create js directory");
    }

    // get base dir
    let base_dir = std::env::current_dir().expect("Failed to get current directory");
    let base_dir = base_dir.to_str().expect("Failed to convert path to string");

    // get the path of the assets
    let asset_files = std::fs::read_dir(css_path).expect("Failed to read asset directory");
    for entry in asset_files {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();
            let content = std::fs::read_to_string(path).expect("Failed to read file");
            css_assets.push(HtmlAssets { filename, content });
        }
    }

    // get the path of the js
    let js_files = std::fs::read_dir(js_path).expect("Failed to read asset directory");
    for entry in js_files {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();
            let content = std::fs::read_to_string(path).expect("Failed to read file");
            js_assets.push(HtmlAssets { filename, content });
        }
    }

    // get the path of the html
    let html_path = format!("{}/interface/html", base_dir);
    let html_files = std::fs::read_dir(html_path).expect("Failed to read asset directory");
    for entry in html_files {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();
            let content = std::fs::read_to_string(path).expect("Failed to read file");
            html_assets.push(HtmlAssets { filename, content });
        }
    }

    HtmlTemplate::GlobCSSAssets.xset(css_assets);
    HtmlTemplate::GlobJSAssets.xset(js_assets);
    HtmlTemplate::GlobLayout.xset(html_assets);


    println!("Base dir: {}", base_dir);
}