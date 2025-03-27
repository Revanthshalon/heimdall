use clap::Args;

#[derive(Args)]
#[command(
    name = "server",
    about = "Starts the server and serves the HTTP REST and gRPC APIs",
    long_about = "This command opens the network ports and listens to HTTP and gRPC API requests
## Configuration

JUSTID heimdall can be configured using environment variables as well as a configuration file. For more information
on configuration options, open the configuration documentation:

>> link <<"
)]
pub struct ServerCommand {
    #[arg(long = "sqa-opt-out", help = "Disable anonymized telemetry reports")]
    pub sqa_opt_out: bool,
}

impl ServerCommand {
    pub async fn execute(&self) {
        println!("Starting the server with the following configurations");
        println!("SQA Opt-Out: {}", self.sqa_opt_out);

        if self.sqa_opt_out {
            println!("Anonymized Telemetry Reports have been disabled");
        } else {
            println!("Anonymized Telemetry Reports enabled");
        }
        if let Err(_e) = heimdall::start_heimdall_service().await {
            eprintln!("Error starting service");
            std::process::exit(1)
        }
    }
}
