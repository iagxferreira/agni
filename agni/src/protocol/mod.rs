pub enum Command {
    Ping,
    Healthcheck,
    Get { key: String },
    Set { key: String, value: Vec<u8> },
    Unknown(String),
}

impl Command {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let input = String::from_utf8_lossy(bytes);
        let mut parts = input.splitn(3, ' ');

        let command = parts.next().unwrap_or("").to_uppercase();

        match command.as_str() {
            "PING" => Command::Ping,
            "HEALTHCHECK" => Command::Healthcheck,
            "GET" => match parts.next() {
                Some(key) => Command::Get {
                    key: key.trim().to_string(),
                },
                None => Command::Unknown("GET requires a key".to_string()),
            },
            "SET" => match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => Command::Set {
                    key: key.trim().to_string(),
                    value: value.trim().as_bytes().to_vec(),
                },
                _ => Command::Unknown("SET requires a key and value".to_string()),
            },
            _ => Command::Unknown(command),
        }
    }
}

pub enum Response {
    Pong,
    Ok,
    Value(Vec<u8>),
    Null,
    Error(String),
}

impl Response {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Response::Pong => b"PONG".to_vec(),
            Response::Ok => b"OK".to_vec(),
            Response::Value(v) => v.clone(),
            Response::Null => b"NULL".to_vec(),
            Response::Error(msg) => format!("ERR {}", msg).into_bytes(),
        }
    }
}
