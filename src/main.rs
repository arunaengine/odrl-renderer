use openapi::run;
use tracing_subscriber::EnvFilter;

mod openapi;
mod template;
mod validate;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or("none".into())
        .add_directive("odrl_renderer=trace".parse().unwrap())
        .add_directive("tower_http=info".parse().unwrap());

    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(filter)
        .init();

    run().await
}
