use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

/// A connected user: their name, and a channel to push messages at them.
///
/// The channel is the important part. A task that has just read a message off
/// Alice's socket cannot write to Bob's socket directly — it does not own it.
/// Instead it sends into Bob's channel, and the task that *does* own Bob's
/// socket does the writing.
struct Peer {
    name: String,
    tx: mpsc::UnboundedSender<Message>,
}

type Peers = Arc<Mutex<HashMap<usize, Peer>>>;

pub async fn run(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    println!("listening on ws://{addr}");

    let peers: Peers = Arc::new(Mutex::new(HashMap::new()));
    let mut next_id = 0usize;

    while let Ok((stream, remote)) = listener.accept().await {
        let id = next_id;
        next_id += 1;

        let peers = Arc::clone(&peers);
        tokio::spawn(async move {
            if let Err(e) = handle(stream, id, Arc::clone(&peers)).await {
                eprintln!("{remote}: {e}");
            }
            // Runs however the connection ended — clean exit, error, or drop.
            let departed = peers.lock().unwrap().remove(&id);
            if let Some(peer) = departed {
                broadcast(&peers, id, &format!("* {} left", peer.name));
            }
        });
    }

    Ok(())
}

async fn handle(stream: TcpStream, id: usize, peers: Peers) -> Result<(), Box<dyn std::error::Error>> {
    let ws = tokio_tungstenite::accept_async(stream).await?;
    let (mut ws_out, mut ws_in) = ws.split();

    // Anything sent into `tx` gets flushed to this client by the task below.
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_out.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Protocol: the first text frame a client sends is its username.
    let name = match ws_in.next().await {
        Some(Ok(Message::Text(text))) => text.to_string(),
        _ => return Ok(()),
    };

    peers.lock().unwrap().insert(id, Peer { name: name.clone(), tx });
    broadcast(&peers, id, &format!("* {name} joined"));

    while let Some(msg) = ws_in.next().await {
        let Message::Text(text) = msg? else { continue };
        let text = text.trim();
        if text.is_empty() {
            continue;
        }

        match text.strip_prefix("/msg ") {
            Some(rest) => {
                let (target, body) = rest.split_once(' ').unwrap_or((rest, ""));
                direct(&peers, &name, target, body);
            }
            None => broadcast(&peers, id, &format!("{name}: {text}")),
        }
    }

    Ok(())
}

/// Send to everyone except `from`.
fn broadcast(peers: &Peers, from: usize, line: &str) {
    let peers = peers.lock().unwrap();
    for (id, peer) in peers.iter() {
        if *id != from {
            let _ = peer.tx.send(Message::text(line));
        }
    }
}

/// Send to one named user, or tell the sender there is no such user.
fn direct(peers: &Peers, from: &str, to: &str, body: &str) {
    let peers = peers.lock().unwrap();
    let find = |name: &str| peers.values().find(|p| p.name == name);

    match find(to) {
        Some(peer) => {
            let _ = peer.tx.send(Message::text(format!("[dm] {from}: {body}")));
        }
        None => {
            if let Some(sender) = find(from) {
                let _ = sender.tx.send(Message::text(format!("* no such user: {to}")));
            }
        }
    }
}
