//! Content-Length decoding shared by the real integration process clients.

use serde_json::Value;
use std::io::BufRead;

pub(super) fn read_message(reader: &mut impl BufRead) -> std::io::Result<Value> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "LSP stdout reached EOF",
            ));
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }
        if let Some(value) = line.strip_prefix("Content-Length:") {
            content_length =
                Some(value.trim().parse::<usize>().map_err(|error| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, error)
                })?);
        }
    }

    let Some(content_length) = content_length else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "missing Content-Length header",
        ));
    };
    let mut body = vec![0; content_length];
    reader.read_exact(&mut body)?;
    serde_json::from_slice(&body)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
}

#[cfg(test)]
mod tests {
    use super::read_message;
    use std::io::{BufReader, Cursor, ErrorKind};

    #[test]
    fn read_message_accepts_an_exact_content_length() {
        let mut reader = BufReader::new(Cursor::new(b"Content-Length: 7\r\n\r\n{\"x\":1}"));

        assert_eq!(
            read_message(&mut reader).unwrap(),
            serde_json::json!({ "x": 1 })
        );
    }

    #[test]
    fn read_message_rejects_a_missing_content_length() {
        let mut reader = BufReader::new(Cursor::new(b"Content-Type: application/json\r\n\r\n{}"));

        let error = read_message(&mut reader).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert!(vize_l0::cstr!("{error}").contains("missing Content-Length"));
    }

    #[test]
    fn read_message_rejects_a_body_shorter_than_content_length() {
        let mut reader = BufReader::new(Cursor::new(b"Content-Length: 8\r\n\r\n{\"x\":1}"));

        assert_eq!(
            read_message(&mut reader).unwrap_err().kind(),
            ErrorKind::UnexpectedEof
        );
    }
}
