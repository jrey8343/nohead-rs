use nohead_rs_config::TracingConfig;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::{Layer, Registry, fmt};

pub struct Tracing;

impl Tracing {
    pub fn init(config: &TracingConfig) {
        let mut layers: Vec<Box<dyn Layer<Registry> + Sync + Send>> = Vec::new();

        let env_filter = init_env_layer(config);

        if config.enable {
            let stdout_layer = fmt::Layer::default()
                .with_ansi(true)
                .with_writer(std::io::stdout)
                .compact()
                .boxed();
            layers.push(stdout_layer);
        }

        tracing_subscriber::registry()
            .with(layers)
            .with(env_filter)
            .with(ErrorLayer::default())
            .init();
    }
}

fn init_env_layer(config: &TracingConfig) -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| config.env_filter.clone().into())
}
