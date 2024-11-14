mod persistence;
mod transformer;
mod web;

extern crate dotenv;

use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use log::info;

use axum::{
    routing::{get, post},
    Router,
};

#[tokio::main]
async fn main() {
    dotenv().ok();

    // initialize tracing
    tracing_subscriber::fmt::init();
    let ctx = web::Context {
        db_client: Arc::new(persistence::setup().await.unwrap()),
    };

    // build our application with a route
    let app = Router::new()
        .route("/guilds", get(web::get_guilds))
        .route("/users", get(web::get_users))
        .route("/messages", get(web::get_messages))
        .route("/message", post(web::post_message))
        .with_state(ctx);

    let host = env::var("HTTP_HOST").expect("Missing Env var: HTTP_HOST");
    let port = env::var("HTTP_PORT").expect("Missing Env var: HTTP_PORT");
    info!("Starting webserver on {host}:{port}");
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}"))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
