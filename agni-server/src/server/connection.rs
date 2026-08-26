use std::time::Instant;

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use tracing::{error, warn};

use agni::protocol::{Command, Response};
use agni::store::Store;

pub async fn handle(socket: TcpStream, store: Store, host: String, port: u16, started_at: Instant) {
    let (reader, writer) = socket.into_split();
    let mut framed_read = FramedRead::new(reader, LengthDelimitedCodec::new());
    let mut framed_write = FramedWrite::new(writer, LengthDelimitedCodec::new());

    while let Some(result) = framed_read.next().await {
        match result {
            Ok(frame) => {
                let response = match Command::from_bytes(&frame) {
                    Command::Ping => Response::Pong,
                    Command::Healthcheck => {
                        warn!(
                            service = "agni",
                            host = %host,
                            port = port,
                            uptime_secs = started_at.elapsed().as_secs(),
                            "healthcheck ok"
                        );
                        Response::Ok
                    }
                    Command::Get { key } => match store.get(&key) {
                        Some(value) => Response::Value(value),
                        None => Response::Null,
                    },
                    Command::Set { key, value } => {
                        store.set(key, value);
                        Response::Ok
                    }
                    Command::Unknown(msg) => Response::Error(format!("unknown command '{}'", msg)),
                };

                if framed_write
                    .send(Bytes::from(response.to_bytes()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            Err(e) => {
                error!("connection error: {}", e);
                break;
            }
        }
    }
}
