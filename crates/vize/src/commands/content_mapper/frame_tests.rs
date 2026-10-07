//! Framing laws independent of mapper projection and project semantics.

use std::io::{self, BufReader, Cursor, ErrorKind, Read};
use vize_l0::cstr;

use super::{MAX_MESSAGE_BYTES, read_frame};

const HEADER_LIMIT: usize = 8 * 1024;

fn frame(header: &[u8], body: &[u8]) -> Vec<u8> {
    [header, b"\r\n", body].concat()
}

#[test]
fn fragmented_and_concatenated_frames_retain_whole_bodies() {
    let first = b"{\"text\":\"\xe6\x97\xa5\xe6\x9c\xac\xe8\xaa\x9e\"}";
    let mut input = frame(
        cstr!(
            "Content-Type: application/json\r\ncOnTeNt-LeNgTh: {}\r\n",
            first.len()
        )
        .as_bytes(),
        first,
    );
    input.extend_from_slice(b"Content-Length: 2\n\n{}");
    for capacity in [1, 2, 7, 128] {
        let mut reader = BufReader::with_capacity(capacity, Cursor::new(&input));
        assert_eq!(read_frame(&mut reader).unwrap(), Some(first.to_vec()));
        assert_eq!(read_frame(&mut reader).unwrap(), Some(b"{}".to_vec()));
        assert_eq!(read_frame(&mut reader).unwrap(), None);
    }
}

#[test]
fn interrupted_header_reads_retry_without_losing_fragmented_bytes() {
    struct InterruptedOnce {
        input: Cursor<Vec<u8>>,
        interrupted: bool,
    }
    impl Read for InterruptedOnce {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(ErrorKind::Interrupted.into());
            }
            self.input.read(output)
        }
    }
    let mut reader = BufReader::with_capacity(
        1,
        InterruptedOnce {
            input: Cursor::new(b"Content-Length: 2\r\n\r\n{}".to_vec()),
            interrupted: false,
        },
    );
    assert_eq!(read_frame(&mut reader).unwrap(), Some(b"{}".to_vec()));
    assert_eq!(read_frame(&mut reader).unwrap(), None);
}

#[test]
fn duplicate_content_lengths_are_rejected_without_consuming_the_body() {
    for second in [
        b"Content-Length: 2".as_slice(),
        b"content-length: 3",
        b"CONTENT-LENGTH: invalid",
    ] {
        let input = [
            b"Content-Length: 2\r\n".as_slice(),
            second,
            b"\r\n\r\n{}tail",
        ]
        .concat();
        let mut reader = Cursor::new(&input);
        let error = read_frame(&mut reader).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert_eq!(cstr!("{error}"), "duplicate Content-Length header");
        assert_eq!(
            reader.position() as usize,
            b"Content-Length: 2\r\n".len() + second.len() + 2
        );
    }
}

#[test]
fn the_complete_header_budget_includes_every_line_and_the_separator() {
    let length = b"Content-Length: 2\r\n";
    let padding = HEADER_LIMIT - length.len() - b"X: \r\n\r\n".len();
    let input = [
        length.as_slice(),
        b"X: ",
        &vec![b'x'; padding],
        b"\r\n\r\n{}",
    ]
    .concat();
    let mut reader = Cursor::new(&input);
    assert_eq!(read_frame(&mut reader).unwrap(), Some(b"{}".to_vec()));
    assert_eq!(reader.position() as usize, HEADER_LIMIT + 2);
    let mut concatenated = input.clone();
    concatenated.extend_from_slice(b"Content-Length: 2\r\n\r\n{}");
    for capacity in [1, 2, 7, 128] {
        let mut reader = BufReader::with_capacity(capacity, Cursor::new(&concatenated));
        assert_eq!(read_frame(&mut reader).unwrap(), Some(b"{}".to_vec()));
        assert_eq!(read_frame(&mut reader).unwrap(), Some(b"{}".to_vec()));
        assert_eq!(read_frame(&mut reader).unwrap(), None);
    }

    for input in [
        vec![b'x'; HEADER_LIMIT + 1],
        [
            b"X: x\n".repeat(HEADER_LIMIT / 5),
            b"Content-Length: 2\r\n\r\n{}".to_vec(),
        ]
        .concat(),
        [
            length.as_slice(),
            b"X: ",
            &vec![b'x'; padding + 1],
            b"\r\n\r\n{}",
        ]
        .concat(),
    ] {
        let mut reader = Cursor::new(&input);
        let error = read_frame(&mut reader).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert_eq!(cstr!("{error}"), "content-mapper headers exceed 8 KiB");
        assert_eq!(reader.position() as usize, HEADER_LIMIT);
        for capacity in [1, 2, 7, 128] {
            let mut reader = BufReader::with_capacity(capacity, Cursor::new(&input));
            let error = read_frame(&mut reader).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::InvalidData);
            assert_eq!(cstr!("{error}"), "content-mapper headers exceed 8 KiB");
            let mut remaining = Vec::new();
            reader.read_to_end(&mut remaining).unwrap();
            assert_eq!(remaining, input[HEADER_LIMIT..]);
        }
    }
}

#[test]
fn eof_missing_invalid_and_oversized_lengths_remain_failures() {
    for (input, kind) in [
        (
            b"Content-Length: 2\r\n".as_slice(),
            ErrorKind::UnexpectedEof,
        ),
        (
            b"Content-Length: 2\r\n\r\n{".as_slice(),
            ErrorKind::UnexpectedEof,
        ),
        (b"X: 1\r\n\r\n".as_slice(), ErrorKind::InvalidData),
        (
            b"Content-Length: nope\r\n\r\n".as_slice(),
            ErrorKind::InvalidData,
        ),
    ] {
        assert_eq!(
            read_frame(&mut Cursor::new(input)).unwrap_err().kind(),
            kind
        );
    }
    let input = cstr!("Content-Length: {}\r\n\r\n", MAX_MESSAGE_BYTES + 1);
    let error = read_frame(&mut Cursor::new(input.as_bytes())).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(cstr!("{error}"), "content-mapper message exceeds 64 MiB");
    assert_eq!(read_frame(&mut Cursor::new(b"" as &[u8])).unwrap(), None);
}
