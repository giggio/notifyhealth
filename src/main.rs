use log::error;
mod lib;

#[tokio::main]
async fn main() {
    if let Err(err) = lib::run().await {
        error!("{}", err);
        std::process::exit(1);
    }
    std::process::exit(0);
}
