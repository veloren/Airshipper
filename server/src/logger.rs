use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub fn init() {
    let env_filter = EnvFilter::from_default_env();

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(env_filter)
        .with_filter_reloading()
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed!");
}
