use clap::{Parser, Subcommand};

mod status;

#[derive(Parser)]
#[command(version, author, name = "heimdall")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<CommandType>,
}

#[derive(Subcommand)]
pub enum CommandType {
    Status(status::StatusCommand),
}

impl Cli {
    pub async fn execute(&self) {
        let command_type = self.command.as_ref();
        if let Some(command_type) = command_type {
            match command_type {
                CommandType::Status(cmd) => cmd.execute(),
            }
        } else {
            println!("Running default behaviour");
        }
        // TODO: Configuration Flag Provider to be added
        if let Err(_e) = heimdall::start_heimdall_service().await {
            todo!()
        }
    }
}
