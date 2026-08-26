mod connection;

use std::io;
use std::time::Instant;
use tokio::net::TcpListener;
use tracing::{info, warn};

use agni::config::Config;
use agni::store::Store;

pub struct Server {
    listener: TcpListener,
    store: Store,
    host: String,
    port: u16,
    started_at: Instant,
}

impl Server {
    pub async fn new(config: &Config) -> io::Result<Self> {
        let listener = TcpListener::bind(config.addr()).await?;
        let started_at = Instant::now();
        warn!(
            service = "agni",
            host = %config.host,
            port = config.port,
            version = env!("CARGO_PKG_VERSION"),
            "server started"
        );
        Ok(Self {
            listener,
            store: Store::new(),
            host: config.host.clone(),
            port: config.port,
            started_at,
        })
    }

    pub async fn run(&self) -> io::Result<()> {
        loop {
            let (socket, addr) = self.listener.accept().await?;
            info!(peer = %addr, "new connection");
            let store = self.store.clone();
            let host = self.host.clone();
            let port = self.port;
            let started_at = self.started_at;
            tokio::spawn(async move {
                connection::handle(socket, store, host, port, started_at).await;
            });
        }
    }
}
