use axum::{
    extract::{Json, Query},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, f64::NEG_INFINITY, sync::Arc};
use tokio::{sync::mpsc, task};

// Segment Tree for O(log n) operations
struct SegmentTree {
    size: usize,                     // Maximum size (capacity)
    current_size: usize,             // Actual number of elements
    data: Vec<(f64, f64, f64, f64)>, // (min, max, sum, sum_sq)
}

impl SegmentTree {
    fn new() -> Self {
        Self {
            size: 0,
            current_size: 0,
            data: Vec::new(),
        }
    }

    fn resize(&mut self, new_size: usize) {
        let tree_size = 2 * new_size.next_power_of_two();
        self.size = new_size;
        self.data
            .resize(tree_size, (f64::INFINITY, f64::NEG_INFINITY, 0.0, 0.0));
    }

    fn batch_update(&mut self, values: &[f64]) {
        // Resize the segment tree if necessary
        let required_size = self.current_size + values.len();
        if required_size > self.size {
            self.resize(required_size);
        }

        // Update leaf nodes with new values
        for (i, &value) in values.iter().enumerate() {
            let idx = self.current_size + i;
            self.data[idx] = (value, value, value, value * value);
        }

        // Recalculate parent nodes
        for idx in (1..self.current_size).rev() {
            let left = self.data[2 * idx];
            let right = self.data[2 * idx + 1];
            self.data[idx] = (
                left.0.min(right.0),
                left.1.max(right.1),
                left.2 + right.2,
                left.3 + right.3,
            );
        }

        // Update current size
        self.current_size = self.current_size.max(required_size);
    }

    fn query(&self, left: usize, right: usize) -> (f64, f64, f64, f64) {
        let mut l = left + self.size;
        let mut r = right + self.size;
        let mut result = (f64::INFINITY, f64::NEG_INFINITY, 0.0, 0.0);

        while l < r {
            if l % 2 == 1 {
                result = self.merge(result, self.data[l]);
                l += 1;
            }
            if r % 2 == 1 {
                r -= 1;
                result = self.merge(result, self.data[r]);
            }
            l /= 2;
            r /= 2;
        }
        result
    }

    fn merge(&self, a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
        (a.0.min(b.0), a.1.max(b.1), a.2 + b.2, a.3 + b.3)
    }
}

// API request/response structures
#[derive(Deserialize)]
struct AddBatchRequest {
    symbol: String,
    values: Vec<f64>,
}

#[derive(Serialize)]
struct AddBatchResponse {
    status: String,
}

#[derive(Deserialize)]
struct StatsRequest {
    symbol: String,
    k: u32,
}

#[derive(Serialize)]
struct StatsResponse {
    min: f64,
    max: f64,
    last: f64,
    avg: f64,
    var: f64,
}

#[tokio::main]
async fn main() {
    let (router, symbols_manager) = SymbolManager::new();

    let batch_router = router.clone();

    tokio::spawn(symbols_manager.run());

    let app = Router::new()
        .route(
            "/add_batch",
            post(move |req| batch_router.clone().handle_add_batch(req)),
        )
        .route(
            "/stats",
            get(move |req| router.clone().handle_get_stats(req)),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Clone)]
struct RouterHandle {
    manager_tx: mpsc::Sender<(String, Command)>,
}

impl RouterHandle {
    async fn handle_add_batch(
        self,
        Json(payload): Json<AddBatchRequest>,
    ) -> Json<AddBatchResponse> {
        let (resp_tx, mut resp_rx) = mpsc::channel(1);
        let command = Command::AddBatch {
            values: payload.values,
            resp: resp_tx,
        };

        let _ = self.manager_tx.send((payload.symbol, command)).await;
        let status = resp_rx
            .recv()
            .await
            .unwrap_or("Failed to add batch".to_string());
        Json(AddBatchResponse { status })
    }

    async fn handle_get_stats(self, Query(params): Query<StatsRequest>) -> Json<StatsResponse> {
        let (resp_tx, mut resp_rx) = mpsc::channel(1);
        let command = Command::GetStats {
            k: params.k,
            resp: resp_tx,
        };

        let _ = self.manager_tx.send((params.symbol, command)).await;
        Json(resp_rx.recv().await.unwrap_or(StatsResponse {
            min: 0.0,
            max: 0.0,
            last: 0.0,
            avg: 0.0,
            var: 0.0,
        }))
    }
}

struct SymbolManager {
    manager_rx: mpsc::Receiver<(String, Command)>,
    symbol_tasks: HashMap<String, mpsc::Sender<Command>>,
}

impl SymbolManager {
    fn new() -> (RouterHandle, Self) {
        let (manager_tx, manager_rx) = mpsc::channel(100);
        (
            RouterHandle { manager_tx },
            SymbolManager {
                manager_rx,
                symbol_tasks: HashMap::new(),
            },
        )
    }

    async fn run(mut self) {
        while let Some((symbol, command)) = self.manager_rx.recv().await {
            if !self.symbol_tasks.contains_key(&symbol) {
                let (task_tx, task_rx) = mpsc::channel(100);
                self.symbol_tasks.insert(symbol.clone(), task_tx.clone());
                tokio::spawn(SymbolTask::new(symbol.clone(), task_rx).run());
            }

            if let Some(task_tx) = self.symbol_tasks.get(&symbol) {
                let _ = task_tx.send(command).await;
            }
        }
    }
}

struct SymbolTask {
    symbol: String,
    tree: SegmentTree,
    receiver: mpsc::Receiver<Command>,
}

impl SymbolTask {
    fn new(symbol: String, receiver: mpsc::Receiver<Command>) -> Self {
        Self {
            symbol,
            tree: SegmentTree::new(),
            receiver,
        }
    }

    async fn run(mut self) {
        while let Some(command) = self.receiver.recv().await {
            match command {
                Command::AddBatch { values, resp } => {
                    // Perform batch update starting at the current size
                    self.tree.batch_update(&values);
                    let _ = resp.send("Batch added successfully".to_string()).await;
                }
                Command::GetStats { k, resp } => {
                    let k = 10usize.pow(k);
                    let total_elements = self.tree.current_size; // Use dynamic size
                    let end = total_elements;
                    let start = if end > k { end - k } else { 0 };

                    println!("Symbol: {}, Start: {}, End: {}", self.symbol, start, end);

                    let (min, max, sum, sum_sq) = self.tree.query(start, end);
                    let count = (end - start) as f64;

                    if count == 0.0 {
                        let _ = resp
                            .send(StatsResponse {
                                min: 0.0,
                                max: 0.0,
                                last: 0.0,
                                avg: 0.0,
                                var: 0.0,
                            })
                            .await;
                        continue;
                    }

                    let avg = sum / count;
                    let var = (sum_sq / count) - (avg * avg);

                    let last = self.tree.query(end - 1, end).2;

                    let _ = resp
                        .send(StatsResponse {
                            min,
                            max,
                            last,
                            avg,
                            var,
                        })
                        .await;
                }
            }
        }
    }
}

enum Command {
    AddBatch {
        values: Vec<f64>,
        resp: mpsc::Sender<String>,
    },
    GetStats {
        k: u32,
        resp: mpsc::Sender<StatsResponse>,
    },
}
