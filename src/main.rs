#[tokio::main(flavor = "current_thread")]
async fn main() {
    let code = nori_lint::cli::run().await;
    std::process::exit(code);
}
