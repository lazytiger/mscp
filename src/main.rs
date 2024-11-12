#[tokio::main]
async fn main() {
    if let Err(err) = mscp::run().await {
        eprintln!("{}", err);
    }
}
