use clap::Parser;
use cmd::Cli;

mod cmd;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    cli.execute().await;
}
