use openapi::run;

mod openapi;
mod template;
mod validate;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let template = template::load_templates().await?;

    println!("{:?}", template);

    run().await
}
