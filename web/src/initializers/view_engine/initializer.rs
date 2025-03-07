use axum::{Extension, Router as AxumRouter};
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
                //FIX: Get css path from config
                Command::new("npx @tailwind/cli")
                    .arg("-i")
                    .arg("../../../styles/input.css")
                    .arg("../../../static/css/output.css")
                    .output()
                    .expect("Failed to run tailwind");
                browser_reloader.reload();
            })
            .expect("Failed to create watcher");

            for path in &[
                state.config.templates.path,
                state.config.wasm_components.path,
            ] {
                let _ = watcher.watch(Path::new(path), notify::RecursiveMode::Recursive);
            }
            // Keep the task running indefinitely to keep the watcher alive
            pending::<()>().await;
        });
        Ok(())
    }

    pub fn after_routes(&self, router: AxumRouter, state: &AppState) -> Result<AxumRouter> {
        let minijinja_engine = View::build(&state.config)?;
        let live_reload = self.live_reload.clone();
        Ok(router
            .layer(live_reload)
            .layer(Extension(ViewEngine::from(minijinja_engine))))
    }
}
