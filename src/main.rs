use axum::{
    routing::{get, post},
    Router,
};
use manager::SymbolManager;
use web::RouterHandle;

mod datastore;
mod interval_stat_deque;
mod manager;
mod web;

#[tokio::main]
async fn main() {
    let symbols_manager = SymbolManager::new();
    let add_batch_router = RouterHandle::new(symbols_manager.manager_tx.clone());
    let stats_router = RouterHandle::new(symbols_manager.manager_tx.clone());

    tokio::spawn(symbols_manager.run());

    let app = Router::new()
        .route(
            "/add_batch",
            post(move |req| add_batch_router.handle_add_batch(req)),
        )
        .route("/stats", get(move |req| stats_router.handle_get_stats(req)));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
