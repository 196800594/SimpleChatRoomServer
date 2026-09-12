pub mod server;

use server::Server;

#[tokio::main]
async fn main() {
    Server::run().await;
}
