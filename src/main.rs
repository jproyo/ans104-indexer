mod cli;
mod indexer;
mod tags;
use crate::cli::Cli;
use crate::indexer::index_bundle;
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    index_bundle(&cli.transaction_id, &cli.output_file).await?;
    Ok(())
}
