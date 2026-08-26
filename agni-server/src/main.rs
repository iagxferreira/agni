mod server;

use clap::Parser;
use tracing::error;

use agni::config::Config;

#[derive(Parser)]
#[command(name = "agni-server", about = "A Redis-like in-memory cache server")]
struct Cli {
    /// Path to the YAML configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();

    let config = match cli.config {
        Some(path) => Config::from_file(&path).unwrap_or_else(|e| {
            error!("{}", e);
            std::process::exit(1);
        }),
        None => Config::default(),
    };

    let server = server::Server::new(&config)
        .await
        .expect("failed to start server");

    if let Err(e) = server.run().await {
        error!("server error: {}", e);
    }
}
