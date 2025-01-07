use std::collections::HashMap;

use serde::Serialize;
use tokio::sync::mpsc;

use crate::datastore::SymbolDataStore;

// ManagerCommand is an enum that represents the commands that can be sent to the manager.
pub enum ManagerCommand {
    AddBatch {
        values: Vec<f32>,
        resp: mpsc::Sender<String>,
    },
    GetStats {
        k: u32,
        resp: mpsc::Sender<Stats>,
    },
}

// State represents the statistics of a symbol.
#[derive(Serialize)]
pub struct Stats {
    pub min: f32,
    pub max: f32,
    pub last: f32,
    pub avg: f32,
    pub var: f32,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Symbol(pub String);

// SymbolManager is a struct that manages the symbol tasks.
pub struct SymbolManager {
    pub manager_tx: mpsc::Sender<(Symbol, ManagerCommand)>,
    manager_rx: mpsc::Receiver<(Symbol, ManagerCommand)>,
    symbol_tasks: HashMap<Symbol, mpsc::Sender<ManagerCommand>>,
}

impl SymbolManager {
    pub fn new() -> Self {
        let (manager_tx, manager_rx) = mpsc::channel(100);
        SymbolManager {
            manager_tx,
            manager_rx,
            symbol_tasks: HashMap::new(),
        }
    }

    pub async fn run(mut self) {
        while let Some((symbol, command)) = self.manager_rx.recv().await {
            if !self.symbol_tasks.contains_key(&symbol) {
                let (task_tx, task_rx) = mpsc::channel(100);
                self.symbol_tasks.insert(symbol.clone(), task_tx.clone());
                tokio::spawn(SymbolTask::new(task_rx).run());
            }

            if let Some(task_tx) = self.symbol_tasks.get(&symbol) {
                let _ = task_tx.send(command).await;
            }
        }
    }
}

struct SymbolTask {
    store: SymbolDataStore,
    receiver: mpsc::Receiver<ManagerCommand>,
}

impl SymbolTask {
    fn new(receiver: mpsc::Receiver<ManagerCommand>) -> Self {
        Self {
            store: SymbolDataStore::new(10usize.pow(8)),
            receiver,
        }
    }

    async fn run(mut self) {
        while let Some(command) = self.receiver.recv().await {
            match command {
                ManagerCommand::AddBatch { values, resp } => {
                    if values.is_empty() || values.len() > 10_000 {
                        let _ = resp.send("Invalid batch size".to_string()).await;
                        continue;
                    }

                    self.store.add_batch(values.as_slice());
                    let _ = resp.send("Batch added successfully".to_string()).await;
                }

                ManagerCommand::GetStats { k, resp } => {
                    let stats = self.store.get_stats(k);

                    let response: Stats = stats
                        .map(|stats| {
                            let avg = stats.sum / stats.count as f32;
                            let var = (stats.sum_squares / stats.count as f32) - (avg * avg);

                            Stats {
                                min: stats.min,
                                max: stats.max,
                                last: stats.last,
                                avg,
                                var,
                            }
                        })
                        .unwrap_or(Stats {
                            min: 0.0,
                            max: 0.0,
                            last: 0.0,
                            avg: 0.0,
                            var: 0.0,
                        });

                    let _ = resp.send(response).await;
                    continue;
                }
            }
        }
    }
}
