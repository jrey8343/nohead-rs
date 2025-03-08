use axum::{Extension, Router as AxumRouter};
use nohead_rs_config::Environment;
use notify::Watcher;
use std::{future::pending, path::Path, process::Command};

use crate::error::Result;
use crate::state::AppState;

use super::engine::{View, ViewEngine, ViewEngineInitializer};

impl ViewEngineInitializer {
    pub fn name(&self) -> String {
        "view-engine".to_string()
    }

    pub fn before_run(&self, state: AppState) -> Result<()> {
        let browser_reloader = self.browser_reloader.clone();
        // Spawn a task to keep the watcher alive
        tokio::spawn(async move {
            // Create the watcher inside the task
            let mut watcher = notify::recommended_watcher(move |_| {
                let _ = generate_css();
                browser_reloader.reload();
            })
            .expect("Failed to create watcher");

            let templates_path =
                Path::new(env!("CARGO_MANIFEST_DIR")).join(Path::new(&state.config.templates.path));
            let components_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(Path::new(&state.config.components.path));

            for path in &[templates_path, components_path] {
                let _ = watcher.watch(path, notify::RecursiveMode::Recursive);
            }
            // Keep the task running indefinitely to keep the watcher alive
            pending::<()>().await;
        });
        Ok(())
    }

    pub fn after_routes(self, mut router: AxumRouter, state: &AppState) -> Result<AxumRouter> {
        let minijinja_engine = View::build(&state.config)?;

        if state.env == Environment::Development {
            tracing::info!("live reload enabled for max dx");
            router = router.layer(self.live_reload_layer);
        }

        router = router.layer(Extension(ViewEngine::from(minijinja_engine)));

        Ok(router)
    }
}

pub fn generate_css() -> Result<()> {
    Command::new("npx")
        .arg("@tailwind/cli")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("-i")
        .arg("./styles/input.css")
        .arg("-o")
        .arg("./static/css/output.css")
        .output()
        .expect("failed to run tailwind");
    Ok(())
}
