mod handlers;
mod models;
mod routes;

use routes::app_routes;

#[tokio::main]
async fn main() {
    let app = app_routes();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind server");

    println!("Fuel ingestion service running on http://127.0.0.1:8080");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}