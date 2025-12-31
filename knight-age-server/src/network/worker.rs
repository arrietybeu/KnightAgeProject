use crate::network::context::PacketContext;
use crate::network::handler::{PacketHandlerResult, PacketRegistry};
use crate::network::packet::Packet;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Message sent to worker pool
pub struct WorkerTask {
    pub packet: Packet,
    pub ctx: PacketContext,
}

/// Worker pool for processing packets concurrently
/// This allows handling many CCU efficiently
pub struct WorkerPool {
    sender: mpsc::Sender<WorkerTask>,
}

impl WorkerPool {
    /// Create a new worker pool with specified number of workers
    pub fn new(num_workers: usize, registry: Arc<PacketRegistry>) -> Self {
        let (sender, receiver) = mpsc::channel::<WorkerTask>(10000);
        let receiver = Arc::new(tokio::sync::Mutex::new(receiver));

        println!(
            "[WorkerPool] Starting {} workers for packet processing",
            num_workers
        );

        // Spawn worker tasks
        for worker_id in 0..num_workers {
            let registry = registry.clone();
            let receiver = receiver.clone();

            tokio::spawn(async move {
                loop {
                    // Get next task
                    let task = {
                        let mut rx = receiver.lock().await;
                        rx.recv().await
                    };

                    match task {
                        Some(task) => {
                            Self::process_task(worker_id, &registry, task).await;
                        }
                        None => {
                            println!("[Worker {}] Channel closed, shutting down", worker_id);
                            break;
                        }
                    }
                }
            });
        }

        Self { sender }
    }

    /// Process a single task
    async fn process_task(worker_id: usize, registry: &PacketRegistry, task: WorkerTask) {
        let session_id = task.ctx.session_id;
        let cmd = task.packet.cmd;

        match registry.handle_packet(&task.packet, &task.ctx).await {
            Ok(result) => match result {
                PacketHandlerResult::Ok => {
                    // Packet handled successfully
                }
                PacketHandlerResult::CloseConnection => {
                    println!(
                        "[Worker {}] Session {} requested close after cmd {}",
                        worker_id, session_id, cmd
                    );
                    // Connection will be closed by session manager
                }
                PacketHandlerResult::Error(e) => {
                    eprintln!(
                        "[Worker {}] Session {} handler error for cmd {}: {}",
                        worker_id, session_id, cmd, e
                    );
                }
            },
            Err(e) => {
                eprintln!(
                    "[Worker {}] Session {} IO error for cmd {}: {}",
                    worker_id, session_id, cmd, e
                );
            }
        }
    }

    /// Submit a task to the worker pool
    pub async fn submit(&self, task: WorkerTask) -> Result<(), mpsc::error::SendError<WorkerTask>> {
        self.sender.send(task).await
    }

    /// Get sender for submitting tasks
    pub fn get_sender(&self) -> mpsc::Sender<WorkerTask> {
        self.sender.clone()
    }
}

/// Builder for creating the server with proper configuration
pub struct ServerConfig {
    pub num_workers: usize,
    pub max_connections: usize,
    pub bind_address: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            num_workers: num_cpus::get().max(4),
            max_connections: 10_000,
            bind_address: "127.0.0.1:19129".to_string(),
        }
    }
}

impl ServerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_workers(mut self, num: usize) -> Self {
        self.num_workers = num;
        self
    }

    pub fn with_bind_address(mut self, addr: &str) -> Self {
        self.bind_address = addr.to_string();
        self
    }
}

