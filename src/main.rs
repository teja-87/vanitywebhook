mod db;  // Your db module

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use serde_json::Value;
use serde_json::json;  // Fixed: Import json! macro
use std::sync::Arc;
use std::env;
use sqlx::PgPool;
use dotenvy;

#[derive(Clone)]
struct AppState {
    db_pool: Arc<PgPool>,
}

#[tokio::main]
async fn main() {
    println!("Hello, world! Starting server...");    

    dotenvy::dotenv().ok();  // Load .env
    let db_url = "postgres://postgres:fThUKQM4EZxh1vxu@rxbsbeppetkggjthchmp.db.ap-southeast-1.nhost.run:5432/rxbsbeppetkggjthchmp".to_string();
    let pool = PgPool::connect(&db_url).await.expect("Failed to connect to Supabase");
    let state = AppState { db_pool: Arc::new(pool) };

    let app = Router::new()
        .route("/webhook", post(webhook_handler))
        .with_state(state);  // Share pool with handlers

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn webhook_handler(
    State(state): State<AppState>,  // Get shared pool
    Json(thedata): axum::extract::Json<Value>,
) -> impl IntoResponse {
    println!("Received data: {}", thedata);

    // Parse example Helius payload (adjust paths for your JSON)
    if let Some(events) = thedata["events"].as_array() {
        for event in events {
            let sig = event["signature"].as_str().unwrap_or("");
            let sender = event["source"].as_str().unwrap_or("");  // Sender wallet
            let lamports = event["lamports"].as_u64().unwrap_or(0);
            let slot = event["slot"].as_i64().unwrap_or(0);
            let timestamp = event["timestamp"].as_i64();

            // Filter: Min SOL to your wallet
            let dest = event["destination"].as_str().unwrap_or("");
            let my_wallet = "YOUR_WALLET_PUBKEY_HERE";  // Replace
            if lamports >= 1_000_000 && dest == my_wallet {  // 0.001 SOL min
                db::add_paid(&state.db_pool, sig, sender, lamports, slot, timestamp, my_wallet).await.unwrap_or(());
            } else {
                println!("Skipped non-payment: {}", sig);
            }
        }
    }

    (StatusCode::OK, Json(json!({"status": "ok", "received": thedata})))
}