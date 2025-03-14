use axum::{Extension, Router as AxumRouter};
use nohead_rs_config::Environment;
use notify::Watcher;
use std::{
    collections::HashMap,
    future::pending,
    path::Path,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::error::Result;
use crate::state::AppState;

use super::engine::{View, ViewEngine, ViewEngineInitializer};

impl ViewEngineInitializer {
    pub fn name(&self) -> String {
        "view-engine".to_string()
    }

    pub fn before_run(&self, state: AppState) -> Result<()> {
        let last_events = Arc::new(Mutex::new(HashMap::new()));

        let browser_reloader = self.browser_reloader.clone();

        // Spawn a task to keep the watcher alive
        tokio::spawn(async move {
            let mut watcher = notify::recommended_watcher({
                let last_events = Arc::clone(&last_events);
                move |res: Result<notify::Event, _>| {
                    match res {
                        Ok(event) => {
                            if let Some(path) = event.paths.first() {
                                let mut last_events = last_events.lock().unwrap();

                                // Ignore temp/backup files
                                // This stops the reloader
                                // from re-running
                                // unnecessarily
                                if path.to_string_lossy().ends_with('~')
                                    || path
                                        .extension()
                                        .map(|ext| ext == "swp" || ext == "swx" || ext == "bak")
                                        .unwrap_or(false)
                                {
                                    return;
                                }

                                let now = Instant::now();

                                // Only reload if enough time has passed since the last accepted reload
                                match last_events.get(path) {
                                    Some(last_time)
                                        if now.duration_since(*last_time)
                                            < std::time::Duration::from_millis(300) =>
                                    {
                                        // Too soon, skip this reload
                                    }
                                    _ => {
                                        // Accept this event and record time *after* accepting it
                                        tracing::info!("File changed: {:?}", path);

                                        browser_reloader.reload();

                                        last_events.insert(path.clone(), now);
                                    }
                                }
                            }
                        }
                        Err(e) => tracing::error!("Watch error: {:?}", e),
                    }
                }
            })
            .expect("Failed to create watcher");

            let templates_path =
                Path::new(env!("CARGO_MANIFEST_DIR")).join(Path::new(&state.config.templates.path));
            let components_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(Path::new(&state.config.components.path));
            let output_css_path =
                Path::new(env!("CARGO_MANIFEST_DIR")).join(Path::new("static/css"));

            for path in &[templates_path, components_path, output_css_path] {
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
            tracing::info!("live reload enabled in development mode");
            router = router.layer(self.live_reload_layer);
        }

        router = router.layer(Extension(ViewEngine::from(minijinja_engine)));

        Ok(router)
    }
}
