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

    // Compile-time embedding of CSS files
    // Add your CSS files here
    css_assets.push(HtmlAssets {
        filename: "layout.css".to_string(),
        content: include_str!("../interface/css/layout.css").to_string(),
    });
    css_assets.push(HtmlAssets {
        filename: "component.css".to_string(),
        content: include_str!("../interface/css/component.css").to_string(),
    });
    // Add more CSS files as needed

    // Compile-time embedding of JS files
    // Add your JS files here
    // js_assets.push(HtmlAssets {
    //     filename: "main.js".to_string(),
    //     content: include_str!("../interface/assets/js/main.js").to_string(),
    // });
    // Add more JS files as needed

    // Compile-time embedding of HTML files
    // Add your HTML files here
    html_assets.push(HtmlAssets {
        filename: "layout-login.html".to_string(),
        content: include_str!("../interface/html-glob/layout-login.html").to_string(),
    });
    html_assets.push(HtmlAssets {
        filename: "layout-dashboard.html".to_string(),
        content: include_str!("../interface/html-glob/layout-dashboard.html").to_string(),
    });
    html_assets.push(HtmlAssets {
        filename: "home.html".to_string(),
        content: include_str!("../interface/html/home.html").to_string(),
    });
    html_assets.push(HtmlAssets {
        filename: "login.html".to_string(),
        content: include_str!("../interface/html/login.html").to_string(),
    });
    // Add more HTML files as needed

    HtmlTemplate::GlobCSSAssets.xset(css_assets);
    HtmlTemplate::GlobJSAssets.xset(js_assets);
    HtmlTemplate::GlobLayout.xset(html_assets);

    // The base_dir is not needed anymore since we're not reading files at runtime
    println!("Assets embedded at build time");
}