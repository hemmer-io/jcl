//! JCL Language Server Protocol binary

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_writer(std::io::stderr) // LSP uses stdout for protocol, so log to stderr
        .init();

    tracing::info!("Starting JCL Language Server");

    jcl::lsp::run_server().await?;

    Ok(())
}
