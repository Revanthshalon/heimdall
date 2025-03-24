#[tokio::main]
async fn main() {
    if let Err(_e) = heimdall::start_heimdall_service().await {
        todo!()
    }
}
