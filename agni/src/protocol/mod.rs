#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ping_and_healthcheck() {
        assert_eq!(Command::from_bytes(b"PING"), Command::Ping);
        assert_eq!(Command::from_bytes(b"HEALTHCHECK"), Command::Healthcheck);
    }

    #[test]
    fn command_verb_is_case_insensitive() {
        assert_eq!(Command::from_bytes(b"ping"), Command::Ping);
        assert_eq!(Command::from_bytes(b"PiNg"), Command::Ping);
    }

    #[test]
    fn parses_get_with_key() {
        assert_eq!(
            Command::from_bytes(b"GET name"),
            Command::Get {
                key: "name".to_string()
            }
        );
    }

    #[test]
    fn parses_set_with_key_and_value() {
        assert_eq!(
            Command::from_bytes(b"SET name agni"),
            Command::Set {
                key: "name".to_string(),
                value: b"agni".to_vec()
            }
        );
    }

    #[test]
    fn set_value_may_contain_spaces() {
        assert_eq!(
            Command::from_bytes(b"SET greeting hello world"),
            Command::Set {
                key: "greeting".to_string(),
                value: b"hello world".to_vec()
            }
        );
    }

    // Characterisation tests: these record what the parser does today so that
    // any change to the behaviour shows up as an explicit diff here.

    #[test]
    fn arity_errors_are_currently_reported_as_unknown_commands() {
        assert_eq!(
            Command::from_bytes(b"GET"),
            Command::Unknown("GET requires a key".to_string())
        );
        assert_eq!(
            Command::from_bytes(b"SET onlykey"),
            Command::Unknown("SET requires a key and value".to_string())
        );
    }

    #[test]
    fn unrecognised_verb_is_unknown_with_the_verb_uppercased() {
        assert_eq!(
            Command::from_bytes(b"flarp"),
            Command::Unknown("FLARP".to_string())
        );
    }

    #[test]
    fn non_utf8_set_values_are_currently_corrupted() {
        // 0xff/0xfe/0x80 are not valid UTF-8; from_utf8_lossy replaces each
        // with U+FFFD (ef bf bd), so 5 bytes in become 11 bytes out.
        let cmd = Command::from_bytes(b"SET k \xff\xfe\x00A\x80");
        let Command::Set { value, .. } = cmd else {
            panic!("expected a SET");
        };
        assert_ne!(value, b"\xff\xfe\x00A\x80".to_vec());
        assert_eq!(value, b"\xef\xbf\xbd\xef\xbf\xbd\x00A\xef\xbf\xbd".to_vec());
    }

    #[test]
    fn responses_encode_to_their_wire_forms() {
        assert_eq!(Response::Pong.to_bytes(), b"PONG".to_vec());
        assert_eq!(Response::Ok.to_bytes(), b"OK".to_vec());
        assert_eq!(Response::Null.to_bytes(), b"NULL".to_vec());
        assert_eq!(Response::Value(b"v".to_vec()).to_bytes(), b"v".to_vec());
        assert_eq!(
            Response::Error("boom".to_string()).to_bytes(),
            b"ERR boom".to_vec()
        );
    }

    #[test]
    fn a_stored_null_literal_is_indistinguishable_from_a_miss() {
        // Known wire-protocol quirk, shared with the snapshot/go and
        // snapshot/kotlin implementations. Recorded, not fixed: the wire
        // format is frozen for benchmark comparability.
        assert_eq!(
            Response::Value(b"NULL".to_vec()).to_bytes(),
            Response::Null.to_bytes()
        );
    }
}
