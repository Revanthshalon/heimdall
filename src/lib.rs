mod dtos;
mod error;
mod state;

pub async fn start_heimdall_service() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Heimdall Service");
    Ok(())
}
