use apalis::prelude::*;
use nohead_rs_db::DbPool;
use nohead_rs_mailer::{EmailClient, EmailPayload};
use tokio::task::JoinHandle;

mod jobs;

pub use apalis::prelude::Storage;
pub use apalis_sql::sqlite::SqliteStorage as WorkerStorage;

pub struct Worker {
    pub storage: WorkerStorage<EmailPayload>,
    pub monitor_task: JoinHandle<Result<(), std::io::Error>>,
}

impl Worker {
    pub fn new(db_pool: &DbPool, email_client: EmailClient) -> Self {
        let storage = WorkerStorage::new(db_pool.clone());

        let storage_cloned = storage.clone();
        let monitor_task = tokio::task::spawn(async move {
            Monitor::new()
                .register({
                    WorkerBuilder::new("email-worker")
                        .concurrency(2)
                        .data(email_client)
                        .enable_tracing()
                        .backend(storage_cloned)
                        .build_fn(jobs::send_email)
                })
                .run()
                .await
                .unwrap();
            Ok::<(), std::io::Error>(())
        });
        Self {
            storage,
            monitor_task,
        }
    }
}

/// Errors that can occur as a result of a data layer operation.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error occured while interacting with worker storage.
    ///
    /// Return `500 Internal Server Error` on a worker storage error.
    #[error("error interacting with worker storage")]
    WorkerStorage(#[from] apalis_sql::SqlError),
}
