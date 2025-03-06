//! Bot
#[tokio::main]
async fn main() {
  if let Err(e) = hivin_bot::run().await {
    eprintln!("Error: {}", e);
  }
}

