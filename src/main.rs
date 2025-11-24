mod db; // ← your db.rs with add_paid()

use axum::{extract::State, Json, Router, routing::post};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    pool: Arc<PgPool>,
}

#[tokio::main]
async fn main() {
    // Nhost connection — change only if you want .env later
    let db_url = "postgres://postgres:fThUKQM4EZxh1vxu@rxbsbeppetkggjthchmp.db.ap-southeast-1.nhost.run:5432/rxbsbeppetkggjthchmp";
    let pool = PgPool::connect(db_url)
        .await
        .expect("Failed to connect to Nhost");
    
    println!("Nhost connected — server starting on 0.0.0.0:3000");

    let state = AppState { pool: Arc::new(pool) };

    let app = Router::new()
        .route("/webhook", post(webhook_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening → forward port 3000 and use the public URL in Helius");
    axum::serve(listener, app).await.unwrap();
}

async fn webhook_handler(
    State(state): State<AppState>,
    Json(thedata): Json<Value>,
) -> Json<Value> {
    // YOUR BELOVED PRINTS — NOW THEY WILL ALWAYS SHOW
    println!("WEBHOOK RECEIVED");
    println!("Full data: {:#}", thedata);

    // Helius logic — same as before
    if let Some(events) = thedata["events"].as_array() {
        for event in events {
            let sig = event["signature"].as_str().unwrap_or("");
            let sender = event["source"].as_str().unwrap_or("");
            let lamports = event["lamports"].as_u64().unwrap_or(0);
            let slot = event["slot"].as_i64().unwrap_or(0);
            let timestamp = event["timestamp"].as_i64();
            let dest = event["destination"].as_str().unwrap_or("");

            let my_wallet = "YOUR_WALLET_PUBKEY_HERE"; // CHANGE THIS

            if lamports >= 1_000_000 && dest == my_wallet {
                println!("PAYMENT DETECTED → {} lamports from {}", lamports, sender);

                // INSERT INTO NHOST
                let _ = db::add_paid(&state.pool, sig, sender, lamports, slot, timestamp, my_wallet).await;
            } else {
                println!("Skipped: {} lamports to {}", lamports, dest);
            }
        }
    }

    // Respond fast
    Json(json!({
        "status": "ok bro",
        "received": thedata
    }))
}