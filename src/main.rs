use log::error;

#[tokio::main]
async fn main() {
    if let Err(err) = notifyhealth::run().await {
        error!("{}", err);
        std::process::exit(1);
    }
    std::process::exit(0);
}
