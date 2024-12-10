use openapi::run;

mod openapi;
mod template;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run().await
}
