use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_tungstenite::tungstenite::Message;

pub async fn run(url: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (ws, _response) = tokio_tungstenite::connect_async(url).await?;
    let (mut ws_out, mut ws_in) = ws.split();

    // The server expects the username as the first frame.
    ws_out.send(Message::text(name)).await?;
    println!("connected as {name} — type to chat, `/msg <user> <text>` for a DM");

    // stdin -> server
    tokio::spawn(async move {
        let mut lines = BufReader::new(tokio::io::stdin()).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if ws_out.send(Message::text(line)).await.is_err() {
                break;
            }
        }
    });

    // server -> stdout
    while let Some(msg) = ws_in.next().await {
        if let Message::Text(text) = msg? {
            println!("{text}");
        }
    }

    println!("disconnected");
    Ok(())
}
