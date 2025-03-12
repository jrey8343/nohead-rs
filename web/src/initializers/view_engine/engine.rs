
use super::error::Error as ViewEngineError;
use axum::{Extension, extract::FromRequestParts, http::request::Parts};
use minijinja::path_loader;
use tower_livereload::{LiveReloadLayer, Reloader};
use minijinja_autoreload::AutoReloader;
use nohead_rs_config::Config;
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;

use super::plugins::components::ComponentEngine;

pub trait ViewRenderer {
    /// Render a view template located by `key`
    ///
    /// # Errors
    ///
    /// This function will return an error if render fails
    fn render<S: Serialize>(&self, key: &str, data: S) -> Result<String, ViewEngineError>;
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ViewEngine<E>(pub E);

impl<E> ViewEngine<E> {
    /// Creates a new [`Engine`] that wraps the given engine
    pub fn new(engine: E) -> Self {
        Self(engine)
    }
}

/// A struct representing an inline Minijinja view renderer.
///
/// This struct provides functionality to render templates using the Minijinja templating engine
/// directly from raw template strings.
pub fn template<S>(template: &str, data: S) -> Result<String, ViewEngineError>
where
    S: Serialize,
{
    let minijinja = minijinja::Environment::new();
    Ok(minijinja.render_str(template, minijinja::Value::from_serialize(data))?)
}

impl<E> From<E> for ViewEngine<E> {
    fn from(inner: E) -> Self {
        Self::new(inner)
    }
}

impl<S, E> FromRequestParts<S> for ViewEngine<E>
where
    S: Send + Sync,
    E: Clone + Send + Sync + 'static,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let Extension(tl): Extension<Self> = Extension::from_request_parts(parts, state)
            .await
            .expect("ViewLayer missing. Is the ViewLayer installed?");

        Ok(tl)
    }
}

#[derive(Clone)]
pub struct View {
    pub reloader: Arc<AutoReloader>,
    pub component_engine: ComponentEngine,
}

impl View {
    pub fn build(config: &Config) -> Result<Self, ViewEngineError> {
        let templates_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join(Path::new(&config.templates.path));

        let reloader = AutoReloader::new(move |notifier| {
            let templates_path = templates_path.clone();
            let mut env = minijinja::Environment::new();
            // Watch the template directory for changes in debug mode
            if cfg!(debug_assertions) {
                notifier.set_fast_reload(true);
                notifier.watch_path(&templates_path, true);
            }
            // Load in the templates from the specified directory
            env.set_loader(path_loader(templates_path));
            Ok(env)
        });
        let component_engine = ComponentEngine::build(config)?;
        Ok(Self {
            reloader: Arc::new(reloader),
            component_engine,
        })
    }
}

impl ViewRenderer for View {
    fn render<S: Serialize>(&self, key: &str, data: S) -> Result<String, ViewEngineError> {
        let env = self.reloader.acquire_env()?;
        let template = env.get_template(key)?;
        let base_html = template.render(minijinja::Value::from_serialize(data))?;
        let rendered = self.clone().component_engine.inject(&base_html)?;
        Ok(rendered)
    }
}

pub struct ViewEngineInitializer {
    pub live_reload_layer: LiveReloadLayer,
    pub browser_reloader: Reloader,
}

impl Default for ViewEngineInitializer {
    fn default() -> Self {
        let live_reload_layer = LiveReloadLayer::new();
        let browser_reloader = live_reload_layer.reloader();
        Self {
            live_reload_layer,
            browser_reloader,
        }
    }
}
