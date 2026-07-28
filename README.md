# IM is an Instant Messaging service written in Rust

A small terminal chat program. One binary is both the server and the client, and
they talk to each other over plain WebSocket (`tokio-tungstenite`, no HTTP
framework).

## What it does right now

- The server listens on `127.0.0.1:9001` and handles any number of clients at once.
- A client picks a name on startup. Everyone else is told when you join or leave.
- Anything you type is broadcast to every other connected user.
- `/msg <user> <text>` sends a private message to one person by name. If nobody has
  that name, you get told so.

That's the whole feature set. There is no login, no encryption, no message history —
a message is only seen by whoever happens to be connected at the time. Nothing is
saved anywhere.

## Try it

The Cargo project lives in the `IM/` subdirectory, so run these from there:

```bash
cd IM

# terminal 1
cargo run -- server

# terminal 2
cargo run -- client alice

# terminal 3
cargo run -- client bob
```

Then type in alice's window and watch it show up in bob's. Try `/msg bob hello` for
a direct message.
