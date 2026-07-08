#[tokio::main(flavor = "current_thread")]
async fn main() {
    lode_lsp::run().await;
}
