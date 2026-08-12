use std::time::{Duration, Instant};

use bytes::Bytes;
use clap::Parser;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

#[derive(Parser)]
#[command(name = "agni-bench")]
struct Cli {
    /// Server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Server port
    #[arg(long, default_value_t = 6379)]
    port: u16,

    /// Number of concurrent connections
    #[arg(short = 'c', long, default_value_t = 50)]
    concurrency: usize,

    /// Total number of operations per scenario
    #[arg(short = 'n', long, default_value_t = 10000)]
    ops: usize,
}

struct BenchResult {
    total_ops: usize,
    elapsed: Duration,
    latencies: Vec<Duration>,
}

impl BenchResult {
    fn ops_per_sec(&self) -> f64 {
        self.total_ops as f64 / self.elapsed.as_secs_f64()
    }

    fn percentile(&mut self, p: f64) -> Duration {
        self.latencies.sort();
        let idx =
            ((self.latencies.len() as f64 * p / 100.0) as usize).min(self.latencies.len() - 1);
        self.latencies[idx]
    }
}

async fn send_recv(
    writer: &mut FramedWrite<tokio::net::tcp::OwnedWriteHalf, LengthDelimitedCodec>,
    reader: &mut FramedRead<tokio::net::tcp::OwnedReadHalf, LengthDelimitedCodec>,
    cmd: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    writer.send(Bytes::from(cmd.to_string())).await?;
    reader.next().await.ok_or("connection closed")??;
    Ok(())
}

async fn run_scenario(
    addr: String,
    concurrency: usize,
    ops: usize,
    build_cmd: impl Fn(usize) -> String + Send + Sync + 'static,
) -> BenchResult {
    let build_cmd = std::sync::Arc::new(build_cmd);
    let ops_per_task = ops / concurrency;

    let start = Instant::now();

    let mut handles = Vec::with_capacity(concurrency);
    for _ in 0..concurrency {
        let addr = addr.clone();
        let build_cmd = build_cmd.clone();

        handles.push(tokio::spawn(async move {
            let stream = TcpStream::connect(&addr).await.expect("connect failed");
            let (reader, writer) = stream.into_split();
            let mut framed_read = FramedRead::new(reader, LengthDelimitedCodec::new());
            let mut framed_write = FramedWrite::new(writer, LengthDelimitedCodec::new());

            let mut latencies = Vec::with_capacity(ops_per_task);

            for i in 0..ops_per_task {
                let cmd = build_cmd(i);
                let op_start = Instant::now();
                send_recv(&mut framed_write, &mut framed_read, &cmd)
                    .await
                    .expect("send/recv failed");
                latencies.push(op_start.elapsed());
            }

            latencies
        }));
    }

    let mut all_latencies = Vec::with_capacity(ops);
    for handle in handles {
        all_latencies.extend(handle.await.expect("task panicked"));
    }

    let elapsed = start.elapsed();
    let total_ops = all_latencies.len();

    BenchResult {
        total_ops,
        elapsed,
        latencies: all_latencies,
    }
}

fn print_result(label: &str, mut result: BenchResult) {
    println!("\n=== {} ===", label);
    println!("  Ops:          {}", result.total_ops);
    println!("  Total time:   {:.2}s", result.elapsed.as_secs_f64());
    println!("  Throughput:   {:.0} ops/sec", result.ops_per_sec());
    println!("  Latency p50:  {:?}", result.percentile(50.0));
    println!("  Latency p95:  {:?}", result.percentile(95.0));
    println!("  Latency p99:  {:?}", result.percentile(99.0));
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let addr = format!("{}:{}", cli.host, cli.port);
    let c = cli.concurrency;
    let n = cli.ops;

    println!("=== Agni Bench ===");
    println!("  Target:       {}", addr);
    println!("  Concurrency:  {} connections", c);
    println!("  Ops/scenario: {}", n);

    // Warm up
    let _ = run_scenario(addr.clone(), c, c, |_| "PING".to_string()).await;

    let result = run_scenario(addr.clone(), c, n, |_| "PING".to_string()).await;
    print_result("PING", result);

    let result = run_scenario(addr.clone(), c, n, |i| {
        format!("SET key:{} value:{}", i % 1000, i)
    })
    .await;
    print_result("SET (1000 unique keys)", result);

    let result = run_scenario(addr.clone(), c, n, |i| format!("GET key:{}", i % 1000)).await;
    print_result("GET (hit)", result);

    let result = run_scenario(addr.clone(), c, n, |i| format!("GET missing:{}", i)).await;
    print_result("GET (miss)", result);

    // Mixed: interleaved SET and GET on same connection
    println!("\n=== Mixed SET+GET ===");
    let build_cmd = std::sync::Arc::new(|i: usize| {
        if i.is_multiple_of(2) {
            format!("SET key:{} value:{}", i % 500, i)
        } else {
            format!("GET key:{}", i % 500)
        }
    });
    let result = run_scenario(addr.clone(), c, n, move |i| build_cmd(i)).await;
    println!("  Ops:          {}", result.total_ops);
    println!("  Total time:   {:.2}s", result.elapsed.as_secs_f64());
    println!("  Throughput:   {:.0} ops/sec", result.ops_per_sec());

    println!("\nDone.");
}
