use axum::{Json, Router, routing::post};
use serde_json::{Value, json};

#[tokio::main]


async fn main() {
    let app=Router::new().route("/webhook", post(webhook_handler));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener,app).await.unwrap();
    println!("Hello, world!");
}


async fn webhook_handler(Json(thedata):Json<Value>)->Json<Value>{

    println!("recieved data");
    println!("this is data:{}",thedata);
    Json(json!({
        "status": "ok",
        "received": thedata
    }))


}