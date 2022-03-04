mod lib;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    lib::run().await?;
    Ok(())
}
