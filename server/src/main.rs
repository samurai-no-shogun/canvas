use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{ws::WebSocket, Filter};
use futures::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};

type Grid = Arc<RwLock<Vec<String>>>;
type Clients = Arc<RwLock<Vec<tokio::sync::mpsc::UnboundedSender<warp::ws::Message>>>>;

#[derive(Serialize, Deserialize)]
struct Update {
    x: usize,
    y: usize,
    color: String,
}

#[tokio::main]
async fn main() {
    let grid: Grid = Arc::new(RwLock::new(vec!["#FFFFFF".to_string(); 50 * 500]));
    let clients: Clients = Arc::new(RwLock::new(Vec::new()));
    
    let grid = warp::any().map(move || grid.clone());
    let clients = warp::any().map(move || clients.clone());

    let websocket = warp::path("ws")
        .and(warp::ws())
        .and(grid)
        .and(clients)
        .map(|ws: warp::ws::Ws, grid, clients| {
            ws.on_upgrade(move |socket| handle_connection(socket, grid, clients))
        });

    let static_files = warp::path::end()
        .and(warp::fs::file("../static/index.html"))
        .or(warp::path("script.js")
            .and(warp::fs::file("../static/script.js")))
        .or(warp::path("styles.css")
            .and(warp::fs::file("../static/styles.css")))
        .or(warp::path("collaboration_canvas_client.js")
            .and(warp::fs::file("../static/collaboration_canvas_client.js")))
        .or(warp::path("collaboration_canvas_client_bg.wasm")
            .and(warp::fs::file("../static/collaboration_canvas_client_bg.wasm")));

    let routes = websocket.or(static_files);

    println!("Server starting on http://localhost:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_connection(ws: WebSocket, grid: Grid, clients: Clients) {
    let (mut ws_sender, mut ws_receiver) = ws.split();
    let (client_sender, mut client_receiver) = tokio::sync::mpsc::unbounded_channel();

    clients.write().await.push(client_sender);

    tokio::task::spawn(async move {
        while let Some(message) = client_receiver.recv().await {
            ws_sender.send(message).await.unwrap_or_else(|e| {
                eprintln!("WebSocket send error: {}", e);
            });
        }
    });

    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(msg) => {
                if let Ok(update) = serde_json::from_str::<Update>(&msg.to_str().unwrap_or_default()) {
                    let mut grid = grid.write().await;
                    grid[update.y * 50 + update.x] = update.color.clone();

                    let update_msg = warp::ws::Message::text(serde_json::to_string(&update).unwrap());
                    let clients = clients.read().await;
                    for client in clients.iter() {
                        let _ = client.send(update_msg.clone());
                    }
                }
            }
            Err(_) => break,
        }
    }

    clients.write().await.retain(|client| client.send(warp::ws::Message::text("close")).is_ok());
}