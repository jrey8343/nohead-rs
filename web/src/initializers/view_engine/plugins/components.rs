use nohead_rs_config::Config;
use serde_json::json;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use crate::initializers::view_engine::error::Error as ViewEngineError;

#[derive(Clone)]
pub struct ComponentEngine {
    pub plugin: Arc<Mutex<extism::Plugin>>,
    pub path: PathBuf,
}

impl ComponentEngine {
    pub fn build(config: &Config) -> Result<Self, ViewEngineError> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(Path::new(&config.components.path));
        let wasm_path = path.join(&config.components.wasm);
        let enhance_wasm = extism::Wasm::file(wasm_path);
        let manifest = extism::Manifest::new([enhance_wasm]);
        let plugin = extism::Plugin::new(&manifest, [], true)?;
        Ok(Self {
            plugin: Arc::new(Mutex::new(plugin)),
            path,
        })
    }
    /*
        Call the SSR function via wasm
    */
    pub fn render(
        &mut self,
        data: &serde_json::Value,
    ) -> Result<serde_json::Value, ViewEngineError> {
        let input = serde_json::to_string(data)?;
        let mut plugin = self.plugin.lock().map_err(|_| ViewEngineError::Mutex)?;
        let res = plugin.call::<&str, &str>("ssr", &input)?;
        let json = serde_json::from_str(res)?;
        Ok(json)
    }
    /*
        Read custom elements from the directory and call the SSR function
        This can be passed to the minijinja render function to enhance the HTML
    */
    pub fn inject(&mut self, base_html: &str) -> Result<String, ViewEngineError> {
        let elements = read_elements(&self.path); // Read custom elements from the directory
        let data = json!({
            "markup": base_html,
            "elements": elements,
        });

        let res = self.render(&data)?; // Call the SSR function

        Ok(res["document"].as_str().unwrap().to_string())
    }
}

fn read_elements(path: &Path) -> HashMap<String, String> {
    let mut elements = HashMap::new();
    let _ = read_directory(path, path, &mut elements);
    elements
}

fn read_directory(
    base_path: &Path,
    current_path: &Path,
    elements: &mut HashMap<String, String>,
) -> Result<(), ViewEngineError> {
    if let Ok(entries) = std::fs::read_dir(current_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let _ = read_directory(base_path, &path, elements);
            } else {
                match path.extension().and_then(|s| s.to_str()) {
                    // Enhance SSR allows for .mjs, .js, and .html files
                    // It will inject html into a js like file
                    Some("mjs") | Some("js") | Some("html") => {
                        let content = std::fs::read_to_string(&path)?;

                        let key = generate_key(base_path, &path)?;
                        let processed_content = match path.extension().and_then(|s| s.to_str()) {
                            Some("html") => {
                                format!(r#"function ({{html, state}}){{return html`{}`}}"#, content)
                            }
                            _ => content,
                        };
                        elements.insert(key, processed_content);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn generate_key(base_path: &Path, path: &Path) -> Result<String, ViewEngineError> {
    let relative_path = path.strip_prefix(base_path)?;

    let maybe_parent = relative_path.parent();
    let file_stem = path.file_stem().unwrap().to_str().unwrap();

    let key = match maybe_parent {
        Some(parent) if parent != Path::new("") => {
            let parent_str = parent
                .to_str()
                .unwrap()
                .replace("/", "-")
                .replace("\\", "-");
            format!("{}-{}", parent_str, file_stem)
        }
        _ => file_stem.to_owned(),
    };

    Ok(key)
}
