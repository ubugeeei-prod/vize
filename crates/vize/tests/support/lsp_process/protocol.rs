//! Content-Length decoding shared by the real integration process clients.

use serde_json::Value;
use std::io::BufRead;

/// `None` means the stream ended before any byte of a new frame. Once a
/// header begins, a missing delimiter or body is still a framing error.
pub(crate) fn read_message(reader: &mut impl BufRead) -> std::io::Result<Option<Value>> {
    let mut content_length = None;
    let mut header_started = false;
    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            return if header_started {
                Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "LSP stdout closed inside frame headers",
                ))
            } else {
                Ok(None)
            };
        }
        header_started = true;
        if !line.ends_with('\n') {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "incomplete LSP header line",
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
        .map(Some)
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
            Some(serde_json::json!({ "x": 1 }))
        );
    }

    #[test]
    fn read_message_accepts_eof_only_before_a_new_frame() {
        let frame = b"Content-Length: 7\r\n\r\n{\"x\":1}";
        let mut reader = BufReader::new(Cursor::new(frame));
        assert_eq!(
            read_message(&mut reader).unwrap(),
            Some(serde_json::json!({ "x": 1 }))
        );
        assert_eq!(read_message(&mut reader).unwrap(), None);
        let mut empty = Cursor::new(b"");
        assert_eq!(read_message(&mut empty).unwrap(), None);
    }

    #[test]
    fn read_message_rejects_every_nonempty_truncation_of_a_frame() {
        let frame = b"Content-Length: 7\r\n\r\n{\"x\":1}";
        for end in 1..frame.len() {
            let mut reader = BufReader::new(Cursor::new(frame.get(..end).unwrap()));
            assert_eq!(
                read_message(&mut reader).unwrap_err().kind(),
                ErrorKind::UnexpectedEof,
                "truncated frame at byte {end}"
            );
        }
    }

    #[test]
    fn read_message_rejects_truncated_following_frames_after_a_complete_message() {
        let first = b"Content-Length: 7\r\n\r\n{\"x\":1}";
        for suffix in [
            b"Content-Length: 7".as_slice(),
            b"Content-Length: 7\r\n".as_slice(),
            b"Content-Length: 7\r\n\r\n{\"x\":".as_slice(),
            b"Content-Type: application/json\r\n".as_slice(),
        ] {
            let mut bytes = first.to_vec();
            bytes.extend_from_slice(suffix);
            let mut reader = BufReader::new(Cursor::new(bytes));
            assert_eq!(
                read_message(&mut reader).unwrap(),
                Some(serde_json::json!({ "x": 1 }))
            );
            assert_eq!(
                read_message(&mut reader).unwrap_err().kind(),
                ErrorKind::UnexpectedEof
            );
        }
    }

    #[test]
    fn read_message_accepts_two_complete_frames_before_boundary_eof() {
        let frame = b"Content-Length: 7\r\n\r\n{\"x\":1}";
        let mut bytes = frame.to_vec();
        bytes.extend_from_slice(frame);
        let mut reader = BufReader::new(Cursor::new(bytes));
        for _ in 0..2 {
            assert_eq!(
                read_message(&mut reader).unwrap(),
                Some(serde_json::json!({ "x": 1 }))
            );
        }
        assert_eq!(read_message(&mut reader).unwrap(), None);
    }

    #[test]
    fn read_message_keeps_complete_malformed_frames_as_errors() {
        for frame in [
            b"Content-Length: nope\r\n\r\n".as_slice(),
            b"Content-Length: 7\r\n\r\n{bad:1}".as_slice(),
        ] {
            let mut reader = BufReader::new(Cursor::new(frame));
            assert_eq!(
                read_message(&mut reader).unwrap_err().kind(),
                ErrorKind::InvalidData
            );
        }
    }

    #[test]
    fn read_message_rejects_a_missing_content_length() {
        let mut reader = BufReader::new(Cursor::new(b"Content-Type: application/json\r\n\r\n{}"));

        let error = read_message(&mut reader).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert_eq!(vize_l0::cstr!("{error}"), "missing Content-Length header");
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
