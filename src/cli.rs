use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    /// ANS-104 bundle transaction ID
    #[clap(short, long)]
    pub transaction_id: String,

    /// Output file path
    #[clap(short, long)]
    pub output_file: String,
}
