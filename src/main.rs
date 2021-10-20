use elearn;

#[tokio::main]
async fn main() -> elearn::Result<()> {
    elearn::run().await
}
