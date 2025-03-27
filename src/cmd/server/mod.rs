use clap::Args;

/// Command for starting the Heimdall server
///
/// This struct represents the command-line arguments for starting the
/// Heimdall server which provides HTTP REST and gRPC API endpoints.
#[derive(Args)]
#[command(
    name = "server",
    about = "Starts the server and serves the HTTP REST and gRPC APIs",
    long_about = "This command opens the network ports and listens to HTTP and gRPC API requests
## Configuration

JustID Heimdall can be configured using environment variables as well as a configuration file. For more information
on configuration options, open the configuration documentation:

>> link <<"
)]
pub struct ServerCommand {
    /// Flag to disable the collection of anonymized telemetry data
    ///
    /// When this flag is set, the server will not send any telemetry
    /// reports, even if they are anonymized.
    #[arg(long = "sqa-opt-out", help = "Disable anonymized telemetry reports")]
    pub sqa_opt_out: bool,
}

impl ServerCommand {
    /// Executes the server command, starting the Heimdall service
    ///
    /// This method starts the Heimdall server with the configured options.
    /// It handles the startup process and reports the status of telemetry
    /// collection based on the provided command-line arguments.
    ///
    /// # Returns
    ///
    /// Returns nothing on success. On failure, the process will exit with code 1.
    ///
    /// # Errors
    ///
    /// If the Heimdall service fails to start, this method will print an error
    /// message to stderr and exit the process with code 1.
    pub async fn execute(&self) {
        println!("Starting the server with the following configurations");
        println!("SQA Opt-Out: {}", self.sqa_opt_out);

        if self.sqa_opt_out {
            println!("Anonymized Telemetry Reports have been disabled");
        } else {
            println!("Anonymized Telemetry Reports enabled");
        }
        // Attempt to start the Heimdall service
        if let Err(_e) = heimdall::start_heimdall_service().await {
            eprintln!("Error starting service");
            std::process::exit(1)
        }
    }
}
