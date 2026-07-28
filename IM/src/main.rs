mod client;
mod server;

const ADDR: &str = "127.0.0.1:9001";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.first().map(String::as_str) {
        Some("server") => server::run(ADDR).await,
        Some("client") => {
            let name = args.get(1).map(String::as_str).unwrap_or("anon");
            client::run(&format!("ws://{ADDR}"), name).await
        }
        _ => {
            eprintln!("usage:\n  cargo run -- server\n  cargo run -- client <name>");
            Ok(())
        }
    }
}
