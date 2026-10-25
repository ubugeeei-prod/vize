use super::{eof_capability, image_digest, known_contract};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Cursor, Read},
};

#[test]
fn streamed_image_digest_matches_whole_bytes_across_block_and_buffer_boundaries() {
    for length in [0, 1, 63, 64, 65, 65_535, 65_536, 65_537, 3 * 65_536 + 17] {
        let bytes: Vec<u8> = (0..length).map(|index| (index % 251) as u8).collect();
        let expected: [u8; 32] = Sha256::digest(&bytes).into();
        assert_eq!(image_digest(bytes.as_slice()).unwrap(), expected);
    }
}

#[test]
fn streamed_image_digest_retries_interrupted_short_reads_until_eof() {
    struct ShortReads {
        image: Cursor<Vec<u8>>,
        calls: usize,
    }
    impl Read for ShortReads {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            assert_eq!(buffer.len(), 64 * 1024);
            self.calls += 1;
            if self.calls % 3 == 1 {
                return Err(io::ErrorKind::Interrupted.into());
            }
            let length = buffer.len().min(97);
            self.image.read(buffer.get_mut(..length).unwrap())
        }
    }
    let bytes: Vec<u8> = (0..131_073).map(|index| (index % 251) as u8).collect();
    let expected: [u8; 32] = Sha256::digest(&bytes).into();
    let mut reader = ShortReads {
        image: Cursor::new(bytes),
        calls: 0,
    };
    assert_eq!(image_digest(&mut reader).unwrap(), expected);
    assert_eq!(reader.image.position(), 131_073);
    assert!(reader.calls > 1_000);
}

#[test]
fn streamed_image_digest_rejects_read_errors_after_a_complete_buffer() {
    struct FailedRead;
    impl Read for FailedRead {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("late image read failure"))
        }
    }
    let image = Cursor::new(vec![42; 65_536]).chain(FailedRead);
    let error = image_digest(image).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "late image read failure");
}

#[test]
fn streamed_image_digest_rejects_invalid_read_lengths() {
    struct InvalidRead;
    impl Read for InvalidRead {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            Ok(buffer.len() + 1)
        }
    }
    assert_eq!(
        image_digest(InvalidRead).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
}

#[test]
fn streamed_image_digest_authenticates_bytes_in_the_final_partial_buffer() {
    let mut bytes = vec![42; 2 * 65_536 + 11];
    let original = image_digest(bytes.as_slice()).unwrap();
    *bytes.get_mut(2 * 65_536 + 5).unwrap() ^= 1;
    assert_ne!(image_digest(bytes.as_slice()).unwrap(), original);
}

#[test]
fn fake_missing_and_changed_physical_images_cannot_borrow_a_provider_contract() {
    let temp = tempfile::tempdir().unwrap();
    let fake = temp.path().join("native");
    for initialized in [known_contract(), super::mapper_contract::initialize()] {
        for bytes in [b"Version 7.0.2\n".as_slice(), b"Version 7.1.0-dev\n\0\xff"] {
            fs::write(&fake, bytes).unwrap();
            assert!(!eof_capability(&fake, &initialized));
        }
        assert!(!eof_capability(&temp.path().join("absent"), &initialized));
    }
}

#[cfg(unix)]
mod native_provider;
mod provider_pairs;

#[cfg(unix)]
#[test]
fn null_ack_precedes_eof_foreign_exit_is_preserved_and_failed_ack_still_closes() {
    use super::close;
    use corsa::runtime::block_on;
    use corsa_lsp::{CorsaError, LspClient, LspSpawnConfig};
    use serde_json::Value;
    use std::os::unix::fs::PermissionsExt;

    for (shutdown_to_eof, mode) in [
        (true, "complete"),
        (false, "ok"),
        (true, "error"),
        (true, "non-null"),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let script = temp.path().join("peer.ts");
        let log = temp.path().join("whole-packets.jsonl");
        let source = include_str!("peer.ts.txt")
            .replace("LOG_PATH", &serde_json::to_string(&log).unwrap())
            .replace("REPLY_MODE", &serde_json::to_string(mode).unwrap());
        fs::write(&script, source).unwrap();
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
        let client = block_on(LspClient::spawn(LspSpawnConfig::new(&script))).unwrap();
        let result = block_on(close(&client, shutdown_to_eof));
        let response = match mode {
            "error" => {
                let expected = json!({"code":-32603,"message":"controlled shutdown refusal","data":{"phase":"shutdown"}});
                let Err(CorsaError::Rpc(error)) = result else {
                    panic!("{result:?}")
                };
                assert_eq!(serde_json::to_value(error).unwrap(), expected);
                json!({"jsonrpc":"2.0","id":1,"error":expected})
            }
            "non-null" => {
                let Err(CorsaError::Json(error)) = result else {
                    panic!("{result:?}")
                };
                assert_eq!(error.classify(), serde_json::error::Category::Data);
                json!({"jsonrpc":"2.0","id":1,"result":{"not":"null"}})
            }
            "ok" | "complete" => {
                result.unwrap();
                json!({"jsonrpc":"2.0","id":1,"result":null})
            }
            _ => unreachable!(),
        };
        let actual: Vec<Value> = fs::read_to_string(&log)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let mut expected = vec![
            json!({"direction":"request","wholePacket":{"jsonrpc":"2.0","id":1,"method":"shutdown"}}),
            json!({"direction":"response","wholePacket":response}),
        ];
        if mode == "complete" {
            expected.push(json!({"direction":"notification","wholePacket":{"jsonrpc":"2.0","method":"window/logMessage","params":{"type":3,"message":"handled method 'shutdown' (1) in 6.071µs"}}}));
        }
        if !shutdown_to_eof {
            expected.push(
                json!({"direction":"request","wholePacket":{"jsonrpc":"2.0","method":"exit"}}),
            );
        }
        expected.push(json!({"event":"stdin-eof"}));
        assert_eq!(
            actual, expected,
            "complete protocol transcript: {mode}, native={shutdown_to_eof}"
        );
    }
}

#[cfg(unix)]
mod completion;
