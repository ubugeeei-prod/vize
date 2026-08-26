use std::fs;

use vize_s0::cstr;

use super::RealCorsaRenameSession;

pub(in crate::ide::rename::corsa_session_tests) fn describe_server_frames(
    session: &RealCorsaRenameSession,
) -> String {
    describe_server_frames_in(&session.protocol_trace_dir)
}

pub(in crate::ide) fn describe_server_frames_in(trace_dir: &std::path::Path) -> String {
    let mut paths = match fs::read_dir(trace_dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("server-") && name.ends_with(".raw"))
            })
            .collect::<Vec<_>>(),
        Err(error) => return cstr!("read server traces: {error}").into(),
    };
    paths.sort();

    let mut valid = 0_usize;
    let mut incomplete = Vec::new();
    for path in paths {
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => return cstr!("read {}: {error}", path.display()).into(),
        };
        match inspect_stream(&bytes) {
            Ok(frame_count) => valid += frame_count,
            Err(StreamError::InvalidJson {
                frame,
                declared_length,
                error,
                body,
            }) => {
                return cstr!(
                    "invalid server frame {}#{frame} declared={declared_length}: {error}; body={body:?}",
                    path.display()
                ).into();
            }
            Err(error) => incomplete.push(cstr!("{}: {error}", path.display())),
        }
    }
    cstr!(
        "server frames valid={valid}, incomplete=[{}]",
        incomplete.join(", ")
    )
    .into()
}

fn inspect_stream(bytes: &[u8]) -> Result<usize, StreamError> {
    let mut offset = 0_usize;
    let mut frame = 0_usize;
    while offset < bytes.len() {
        let header_end = find_bytes(&bytes[offset..], b"\r\n\r\n")
            .ok_or(StreamError::IncompleteHeader { offset })?;
        let body_start = offset + header_end + 4;
        let declared_length = content_length(&bytes[offset..offset + header_end])?;
        let body_end = body_start + declared_length;
        if body_end > bytes.len() {
            return Err(StreamError::IncompleteBody {
                frame,
                declared_length,
                available: bytes.len() - body_start,
            });
        }
        let body = &bytes[body_start..body_end];
        if let Err(error) = serde_json::from_slice::<serde_json::Value>(body) {
            let preview = String::from_utf8_lossy(&body[..body.len().min(512)]).into_owned();
            return Err(StreamError::InvalidJson {
                frame,
                declared_length,
                error,
                body: preview,
            });
        }
        offset = body_end;
        frame += 1;
    }
    Ok(frame)
}

fn content_length(header: &[u8]) -> Result<usize, StreamError> {
    for line in header.split(|byte| *byte == b'\n') {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        let Some(index) = line.iter().position(|byte| *byte == b':') else {
            continue;
        };
        let key = &line[..index];
        let value = &line[index + 1..];
        if key.eq_ignore_ascii_case(b"Content-Length") {
            let value = std::str::from_utf8(value)
                .map_err(|_| StreamError::InvalidContentLength)?
                .trim();
            return value.parse().map_err(|_| StreamError::InvalidContentLength);
        }
    }
    Err(StreamError::MissingContentLength)
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[derive(Debug)]
enum StreamError {
    IncompleteHeader {
        offset: usize,
    },
    InvalidContentLength,
    MissingContentLength,
    IncompleteBody {
        frame: usize,
        declared_length: usize,
        available: usize,
    },
    InvalidJson {
        frame: usize,
        declared_length: usize,
        error: serde_json::Error,
        body: String,
    },
}

impl std::fmt::Display for StreamError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IncompleteHeader { offset } => write!(formatter, "incomplete header at {offset}"),
            Self::InvalidContentLength => formatter.write_str("invalid Content-Length"),
            Self::MissingContentLength => formatter.write_str("missing Content-Length"),
            Self::IncompleteBody {
                frame,
                declared_length,
                available,
            } => write!(
                formatter,
                "incomplete frame {frame}: declared={declared_length}, available={available}"
            ),
            Self::InvalidJson { .. } => unreachable!("formatted at the evidence boundary"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_frame_evidence_rejects_trailing_json_inside_one_body() {
        let body = br#"{"jsonrpc":"2.0","id":1,"result":null}{}"#;
        let stream = cstr!("Content-Length: {}\r\n\r\n", body.len())
            .into_string()
            .into_bytes()
            .into_iter()
            .chain(body.iter().copied())
            .collect::<Vec<_>>();
        let StreamError::InvalidJson {
            frame,
            declared_length,
            body: actual,
            ..
        } = inspect_stream(&stream).expect_err("trailing JSON must fail closed")
        else {
            panic!("unexpected framing error")
        };
        assert_eq!(frame, 0);
        assert_eq!(declared_length, body.len());
        assert_eq!(actual.as_bytes(), body);
    }

    #[test]
    fn server_frame_evidence_accepts_adjacent_content_length_frames() {
        let bodies = [
            br#"{"jsonrpc":"2.0","id":1}"#.as_slice(),
            br#"{"method":"exit"}"#,
        ];
        let stream = bodies
            .into_iter()
            .flat_map(|body| {
                cstr!("Content-Length: {}\r\n\r\n", body.len())
                    .into_string()
                    .into_bytes()
                    .into_iter()
                    .chain(body.iter().copied())
            })
            .collect::<Vec<_>>();
        assert_eq!(inspect_stream(&stream).expect("valid frames"), 2);
    }
}
