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
        if self.latencies.is_empty() {
            return Duration::ZERO;
        }
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

/// Rejects inputs that would otherwise crash `run_scenario`: a concurrency of 0
/// divides by zero computing `ops_per_task`, and `ops < concurrency` truncates
/// `ops_per_task` to 0, leaving an empty latency set for `percentile`.
fn validate_flags(concurrency: usize, ops: usize) -> Result<(), String> {
    if concurrency == 0 {
        return Err("concurrency (-c) must be greater than 0".to_string());
    }
    if ops < concurrency {
        return Err(format!(
            "ops (-n {ops}) must be >= concurrency (-c {concurrency})"
        ));
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(err) = validate_flags(cli.concurrency, cli.ops) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    fn result_from(millis: &[u64]) -> BenchResult {
        BenchResult {
            total_ops: millis.len(),
            elapsed: Duration::from_secs(1),
            latencies: millis.iter().map(|m| Duration::from_millis(*m)).collect(),
        }
    }

    // Regression: an empty latency set used to underflow `len() - 1` on usize
    // and panic. Reachable whenever ops < concurrency truncated ops_per_task to 0.
    #[test]
    fn percentile_of_empty_result_is_zero() {
        let mut result = result_from(&[]);
        assert_eq!(result.percentile(50.0), Duration::ZERO);
        assert_eq!(result.percentile(99.0), Duration::ZERO);
    }

    #[test]
    fn percentile_of_single_sample_is_that_sample() {
        let mut result = result_from(&[7]);
        assert_eq!(result.percentile(50.0), Duration::from_millis(7));
        assert_eq!(result.percentile(99.0), Duration::from_millis(7));
    }

    #[test]
    fn percentile_sorts_before_indexing() {
        let mut result = result_from(&[50, 10, 40, 20, 30]);
        assert_eq!(result.percentile(0.0), Duration::from_millis(10));
        assert_eq!(result.percentile(50.0), Duration::from_millis(30));
    }

    #[test]
    fn percentile_never_indexes_past_the_end() {
        let mut result = result_from(&[1, 2, 3, 4]);
        assert_eq!(result.percentile(100.0), Duration::from_millis(4));
    }

    #[test]
    fn validate_flags_rejects_zero_concurrency() {
        let err = validate_flags(0, 10_000).unwrap_err();
        assert!(err.contains("concurrency"), "unexpected message: {err}");
    }

    #[test]
    fn validate_flags_rejects_ops_below_concurrency() {
        let err = validate_flags(50, 10).unwrap_err();
        assert!(err.contains("ops"), "unexpected message: {err}");
    }

    #[test]
    fn validate_flags_accepts_ops_equal_to_concurrency() {
        assert!(validate_flags(50, 50).is_ok());
    }

    #[test]
    fn validate_flags_accepts_defaults() {
        assert!(validate_flags(50, 10_000).is_ok());
    }
}
