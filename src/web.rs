use axum::extract::{Json, Query};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::manager::{ManagerCommand, Stats, Symbol};

// AddBatchRequest is a struct that represents the request body for the add_batch endpoint.
#[derive(Deserialize)]
pub struct AddBatchRequest {
    pub symbol: String,
    pub values: Vec<f32>,
}

// AddBatchResponse is a struct that represents the response body for the add_batch endpoint.
#[derive(Serialize)]
pub struct AddBatchResponse {
    pub status: String,
}

// StatsRequest is a struct that represents the request query parameters for the stats endpoint.
#[derive(Deserialize)]
pub struct StatsRequest {
    pub symbol: String,
    pub k: u32,
}

// RouterHandle is a struct that holds the manager_tx sender and forwards requests to the manager.
#[derive(Clone)]
pub struct RouterHandle {
    pub manager_tx: mpsc::Sender<(Symbol, ManagerCommand)>,
}

impl RouterHandle {
    pub fn new(manager_tx: mpsc::Sender<(Symbol, ManagerCommand)>) -> Self {
        RouterHandle { manager_tx }
    }

    pub async fn handle_add_batch(
        self,
        Json(payload): Json<AddBatchRequest>,
    ) -> Json<AddBatchResponse> {
        let (resp_tx, mut resp_rx) = mpsc::channel(1);
        let command = ManagerCommand::AddBatch {
            values: payload.values,
            resp: resp_tx,
        };
        let sym = Symbol(payload.symbol);

        let _ = self.manager_tx.send((sym, command)).await;
        let status = resp_rx
            .recv()
            .await
            .unwrap_or("Failed to add batch".to_string());
        Json(AddBatchResponse { status })
    }

    pub async fn handle_get_stats(self, Query(params): Query<StatsRequest>) -> Json<Stats> {
        let (resp_tx, mut resp_rx) = mpsc::channel(1);
        let command = ManagerCommand::GetStats {
            k: params.k,
            resp: resp_tx,
        };
        let sym = Symbol(params.symbol);
        let _ = self.manager_tx.send((sym, command)).await;
        Json(resp_rx.recv().await.unwrap_or(Stats {
            min: 0.0,
            max: 0.0,
            last: 0.0,
            avg: 0.0,
            var: 0.0,
        }))
    }
}
