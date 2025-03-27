use clap::Parser;

#[derive(Parser)]
#[command(name = "status", about = "Check server status")]
pub struct StatusCommand;

impl StatusCommand {
    pub fn execute(&self) {
        println!("Executing status command");
    }
}
