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
        let mut parts = bytes.splitn(3, |b| *b == b' ');
        let verb = parts.next().unwrap_or_default().to_ascii_uppercase();

        match verb.as_slice() {
            b"PING" => Command::Ping,
            b"HEALTHCHECK" => Command::Healthcheck,
            b"GET" => match parts.next() {
                Some(key) => match parse_key(key) {
                    Some(key) => Command::Get { key },
                    None => Command::Unknown("GET key must be valid UTF-8".to_string()),
                },
                None => Command::Unknown("GET requires a key".to_string()),
            },
            b"SET" => match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => match parse_key(key) {
                    // The value is kept as raw bytes: the store holds Vec<u8>
                    // and the framing is length-delimited, so nothing on the
                    // path requires it to be text.
                    Some(key) => Command::Set {
                        key,
                        value: value.trim_ascii().to_vec(),
                    },
                    None => Command::Unknown("SET key must be valid UTF-8".to_string()),
                },
                _ => Command::Unknown("SET requires a key and value".to_string()),
            },
            _ => Command::Unknown(String::from_utf8_lossy(&verb).into_owned()),
        }
    }
}

/// Keys must be UTF-8 because both `Command` and the store type them as
/// `String`. Returning None lets the caller report that explicitly instead of
/// silently substituting replacement characters.
fn parse_key(bytes: &[u8]) -> Option<String> {
    std::str::from_utf8(bytes.trim_ascii())
        .ok()
        .map(str::to_string)
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
    fn non_utf8_set_values_round_trip_unchanged() {
        // 0xff/0xfe/0x80 are not valid UTF-8. These used to be replaced with
        // U+FFFD, turning 5 bytes into 11 and silently corrupting the write.
        let raw = b"\xff\xfe\x00A\x80";
        let cmd = Command::from_bytes(b"SET k \xff\xfe\x00A\x80");
        let Command::Set { key, value } = cmd else {
            panic!("expected a SET");
        };
        assert_eq!(key, "k");
        assert_eq!(value, raw.to_vec());
    }

    #[test]
    fn non_utf8_keys_are_rejected_rather_than_corrupted() {
        assert_eq!(
            Command::from_bytes(b"GET \xff\xfe"),
            Command::Unknown("GET key must be valid UTF-8".to_string())
        );
        assert_eq!(
            Command::from_bytes(b"SET \xff\xfe v"),
            Command::Unknown("SET key must be valid UTF-8".to_string())
        );
    }

    #[test]
    fn set_value_preserves_interior_bytes_exactly() {
        let cmd = Command::from_bytes(b"SET k a\x00b");
        let Command::Set { value, .. } = cmd else {
            panic!("expected a SET");
        };
        assert_eq!(value, b"a\x00b".to_vec());
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
