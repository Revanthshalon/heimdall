use clap::{Parser, Subcommand};

mod server;
mod status;

#[derive(Parser)]
#[command(
    name = "heimdall",
    version = "1.0",
    about = "Global and consistent permission and authorization server"
)]
pub struct Cli {
    #[command(subcommand)]
    command: CommandType,
}

#[derive(Subcommand)]
pub enum CommandType {
    Status(status::StatusCommand),
    Server(server::ServerCommand),
}

impl Cli {
    pub async fn execute(&self) {
        match &self.command {
            CommandType::Status(cmd) => cmd.execute(),
            CommandType::Server(cmd) => cmd.execute().await,
        }
    }
}
