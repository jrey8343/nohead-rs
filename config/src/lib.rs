use std::{
    env,
    fmt::{Display, Formatter},
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use color_eyre::eyre::{Context as _, Error, eyre};
use dotenvy::dotenv;
use figment::{
    Figment,
    providers::{Env, Format as _, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use tracing::info;
/// The application configuration.
///
/// This struct is the central point for the entire application configuration. It holds the [`ServerConfig`] [`DatabaseConfig`] [`TracingConfig`] as well as [`StaticAssetsConfig`] and can be extended with any application-specific configuration settings that will be read from the main `app.toml` and the environment-specific configuration files.
///
/// For any setting that appears in both the `app.toml` and the environment-specific file, the latter will override the former so that default settings can be kept in `app.toml` that are overridden per environment if necessary.
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub tracing: TracingConfig,
    pub static_assets: StaticAssetsConfig,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ServerConfig {
    /// The port to bind to, e.g. 3000
    pub port: u16,

    /// The ip to bind to, e.g. 127.0.0.1 or ::1
    pub ip: IpAddr,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 3000,
        }
    }
}
impl ServerConfig {
    /// Returns the full address the server binds to, including both the ip and port.
    ///
    /// This can be used when creating a TCP Listener:
    ///
    /// ```rust
    /// let config: Config = load_config(Environment::Development);
    /// let listener = TcpListener::bind(&config.server.addr).await?;
    /// serve(listener, app.into_make_service()).await?;
    ///  ```
    pub fn addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }
}

#[derive(Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct DatabaseConfig {
    /// The URL to use to connect to the database, e.g. "sqlite://database.db"
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct StaticAssetsConfig {
    pub precompressed: bool,
    pub path: String,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct TracingConfig {
    pub enable: bool,
    pub env_filter: String,
}

/// Loads the application configuration for a particular environment.
///
/// Depending on the environment, this function will behave differently:
/// * for [`Environment::Development`], the function will load env vars from a `.env` file at the project root if that is present
/// * for [`Environment::Test`], the function will load env vars from a `.env.test` file at the project root if that is present
/// * for [`Environment::Staging`], the function will only use the process env vars, and not load a `.env` file
/// * for [`Environment::Production`], the function will only use the process env vars, and not load a `.env` file
///
/// In case the .env or .env.test files live in another directory,
/// you can set that location using the APP_DOTENV_CONFIG_DIR environment variable.
/// This is useful when they are mounted at separate locations in a Docker container, for example.
///
/// Configuration settings are loaded from these sources (in that order so that latter soruces override former):
/// * the `config/app.toml` file
/// * the `config/environments/<development|staging|production|test>.toml` files depending on the environment
/// * environment variables
pub fn load_config<'a, T>(env: &Environment) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let dotenv_config_dir = env::var("APP_DOTENV_CONFIG_DIR")
        .ok()
        .map(std::path::PathBuf::from);

    match (env, dotenv_config_dir) {
        (Environment::Development, None) => {
            dotenv().ok();
        }
        (Environment::Test, None) => {
            dotenvy::from_filename(".env.test").ok();
        }
        (Environment::Development, Some(mut dotenv_config_dir)) => {
            dotenv_config_dir.push(".env");
            dotenvy::from_filename(dotenv_config_dir).ok();
        }
        (Environment::Test, Some(mut dotenv_config_dir)) => {
            dotenv_config_dir.push(".env.test");
            dotenvy::from_filename(dotenv_config_dir).ok();
        }
        _ => { /* don't use any .env file for production */ }
    }

    let env_config_file = match env {
        Environment::Development => "development.toml",
        Environment::Staging => "staging.toml",
        Environment::Production => "production.toml",
        Environment::Test => "test.toml",
    };

    let config: T = Figment::new()
        .merge(
            Serialized::defaults(ServerConfig::default())
                .key("server")
                .key("database")
                .key("tracing")
                .key("static_assets"),
        )
        .merge(Toml::file("config/app.toml"))
        .merge(Toml::file(format!(
            "config/environments/{}",
            env_config_file
        )))
        .merge(Env::prefixed("APP_").split("__"))
        .extract()
        .wrap_err("Could not read configuration!")?;

    Ok(config)
}

/// The environment the application runs in.
///
/// The application can run in 3 different environments: development, production, and test. Depending on the environment, the configuration might be different (e.g. different databases) or the application might behave differently.
#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    /// The development environment is what developers would use locally.
    Development,
    /// The staging environment would typically be used in a staging deployment of the app.
    Staging,
    /// The production environment would typically be used in the released, user-facing deployment of the app.
    Production,
    /// The test environment is using when running e.g. `cargo test`
    Test,
}

impl Display for Environment {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Staging => write!(f, "staging"),
            Environment::Production => write!(f, "production"),
            Environment::Test => write!(f, "test"),
        }
    }
}

/// Returns the currently active environment.
///
/// If the `APP_ENVIRONMENT` env var is set, the application environment is parsed from that (which might fail if an invalid environment is set). If the env var is not set, [`Environment::Development`] is returned.
pub fn get_env() -> Result<Environment, Error> {
    match env::var("APP_ENVIRONMENT") {
        Ok(val) => {
            info!(r#"Setting environment from APP_ENVIRONMENT: "{}""#, val);
            parse_env(&val)
        }
        Err(_) => {
            info!("Defaulting to environment: development");
            Ok(Environment::Development)
        }
    }
}

/// Parses an [`Environment`] from a string.
///
/// The environment can be passed in different forms, e.g. "dev", "development", "prod", etc. If an invalid environment is passed, an error is returned.
pub fn parse_env(env: &str) -> Result<Environment, Error> {
    let env = &env.to_lowercase();
    match env.as_str() {
        "dev" => Ok(Environment::Development),
        "development" => Ok(Environment::Development),
        "stage" => Ok(Environment::Staging),
        "staging" => Ok(Environment::Staging),
        "test" => Ok(Environment::Test),
        "prod" => Ok(Environment::Production),
        "production" => Ok(Environment::Production),
        unknown => Err(eyre!(r#"Unknown environment: "{}"!"#, unknown)),
    }
}
