use ans104_indexer::indexer::indexer_default::Indexer;
use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    /// ANS-104 bundle transaction ID
    #[clap(short, long)]
    pub transaction_id: String,

    /// Output file path
    #[clap(short, long, default_value = "./storage")]
    pub storage_folder: String,

    #[clap(short, long, default_value = "https://arweave.net")]
    pub arwaeve_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let indexer =
        Indexer::default(&cli.arwaeve_url, &cli.transaction_id, &cli.storage_folder).await?;
    indexer.index(cli.transaction_id).await?;
    Ok(())
}
