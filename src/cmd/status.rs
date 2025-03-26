use clap::Parser;

#[derive(Parser)]
pub struct StatusCommand;

impl StatusCommand {
    pub fn execute(&self) {
        println!("Executing status command");
    }
}
