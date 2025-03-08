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
        let browser_reloader = self.browser_reloader.clone();
        let last_events = Arc::new(Mutex::new(HashMap::new()));
        // Spawn a task to keep the watcher alive
        tokio::spawn(async move {
            // Create the watcher inside the task
            // let mut watcher = notify::recommended_watcher(move |_| {
            //     tracing::info!("♼ reloading browser");
            //     // let _ = generate_css();
            //     browser_reloader.reload();
            // })
            // .expect("Failed to create watcher");
            //
            let mut watcher = notify::recommended_watcher({
                let last_events = Arc::clone(&last_events);

                move |res: Result<notify::Event, _>| {
                    match res {
                        Ok(event) => {
                            if let Some(path) = event.paths.first() {
                                let mut last_events = last_events.lock().unwrap();

                                // Ignore temp/backup files
                                if path.to_string_lossy().ends_with('~')
                                    || path
                                        .extension()
                                        .map(|ext| ext == "swp" || ext == "swx" || ext == "bak")
                                        .unwrap_or(false)
                                {
                                    return;
                                }

                                // Get current time and check for duplicates
                                let now = Instant::now();
                                let last_time = last_events.entry(path.clone()).or_insert(now);

                                // If the same file was modified within 300ms, ignore it
                                if now.duration_since(*last_time)
                                    < std::time::Duration::from_millis(300)
                                {
                                    return;
                                }

                                // Update last event time
                                *last_time = now;

                                tracing::info!("File changed: {:?}", path);
                                browser_reloader.reload();
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
            tracing::info!("live reload enabled for max dx");
            router = router.layer(self.live_reload_layer);
        }

        router = router.layer(Extension(ViewEngine::from(minijinja_engine)));

        Ok(router)
    }
}

fn generate_css() -> Result<()> {
    todo!()
}
