use openapi::run;

mod openapi;
mod template;
mod validate;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run().await
}
