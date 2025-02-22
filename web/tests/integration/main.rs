mod login_test;
mod todos_test;

use std::sync::OnceLock;

use axum_test::{TestServer, TestServerBuilder};
use fake::{Fake, Faker};
use nohead_rs_config::Environment;
use nohead_rs_db::{
    DbPool,
    entities::user::{RegisterUser, User, UserCredentials},
};
use nohead_rs_web::{app::App, state::AppState, tracing::Tracing};

fn lazy_tracing(app_state: &AppState) {
    static TRACING: OnceLock<()> = OnceLock::new();
    TRACING.get_or_init(|| Tracing::init(&app_state.config.tracing));
}

fn lazy_eyre() {
    static EYRE: OnceLock<()> = OnceLock::new();
    EYRE.get_or_init(|| color_eyre::install().expect("failed to initialize Eyre"));
}

pub async fn mock_logged_in_state(request: &TestServer, pool: &DbPool) -> User {
    let user: RegisterUser = Faker.fake();

    let saved_user = User::create(user.clone(), pool).await.unwrap();

    request
        .post("/auth/login")
        .form(&UserCredentials {
            email: user.email,
            password: user.password,
            next: None,
        })
        .await;

    saved_user
}
pub async fn authenticated_request<F, Fut>(test_db: DbPool, callback: F)
where
    F: FnOnce(TestServer) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    lazy_eyre();

    let mut app_state = AppState::build(Environment::Test)
        .await
        .expect("failed to build app state");

    // [sqlx::test] sets up a test database when running the test and cleans up afterwards
    // https://docs.rs/sqlx/latest/sqlx/attr.test.html
    app_state.db_pool = test_db.clone();

    if std::env::var("TEST_LOG").is_ok() {
        lazy_tracing(&app_state);
    }

    let app = App::build(app_state).expect("failed to boot test app");

    let config = TestServerBuilder::new()
        .transport(axum_test::Transport::HttpRandomPort)
        .default_content_type("application/json")
        .save_cookies()
        .into_config();

    let server = TestServer::new_with_config(app.router, config)
        .expect("unable to parse axum test server config");

    mock_logged_in_state(&server, &test_db).await;

    callback(server).await;
}

pub async fn test_request_with_db<F, Fut>(test_db: DbPool, callback: F)
where
    F: FnOnce(TestServer) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    lazy_eyre();

    let mut app_state = AppState::build(Environment::Test)
        .await
        .expect("failed to build app state");

    // [sqlx::test] sets up a test database when running the test and cleans up afterwards
    // https://docs.rs/sqlx/latest/sqlx/attr.test.html
    app_state.db_pool = test_db;

    if std::env::var("TEST_LOG").is_ok() {
        lazy_tracing(&app_state);
    }

    let app = App::build(app_state).expect("failed to boot test app");

    let config = TestServerBuilder::new()
        .transport(axum_test::Transport::HttpRandomPort)
        .default_content_type("application/json")
        .into_config();

    let server = TestServer::new_with_config(app.router, config)
        .expect("unable to parse axum test server config");

    callback(server).await;
}

pub async fn test_request<F, Fut>(callback: F)
where
    F: FnOnce(TestServer) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    lazy_eyre();

    let app_state = AppState::build(Environment::Test)
        .await
        .expect("failed to build app context");

    if std::env::var("TEST_LOG").is_ok() {
        lazy_tracing(&app_state);
    }

    let app = App::build(app_state).expect("failed to boot test app");

    let config = TestServerBuilder::new()
        .transport(axum_test::Transport::HttpRandomPort)
        .default_content_type("application/json")
        .into_config();

    let server = TestServer::new_with_config(app.router, config)
        .expect("unable to parse axum test server config");

    callback(server).await;
}
