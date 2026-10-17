use super::{
    DARWIN_ARM64, LINUX_X64, eof_capability, known_binary, known_contract, known_initialize,
};
use serde_json::json;
use std::fs;

#[test]
fn eof_requires_both_full_physical_image_and_complete_initialize_contract() {
    let initialized = known_contract();
    assert!(known_initialize(&initialized));
    assert!(known_binary(LINUX_X64));
    assert!(known_binary(DARWIN_ARM64));
    for value in [
        json!(null),
        json!({}),
        json!({"serverInfo":{"name":"typescript-go"}}),
        json!({"serverInfo":{"name":"other","version":"7.0.2"}}),
        json!({"serverInfo":{"name":"typescript-go","version":"7.0.3"}}),
        json!({"serverInfo":{"name":"typescript-go","version":7.02}}),
    ] {
        assert!(!known_initialize(&value));
    }
    let mut changed = initialized.clone();
    changed["capabilities"]["positionEncoding"] = json!("utf-8");
    assert!(!known_initialize(&changed));
    let mut changed = initialized.clone();
    changed["capabilities"]
        .as_object_mut()
        .unwrap()
        .remove("diagnosticProvider");
    assert!(!known_initialize(&changed));
    let temp = tempfile::tempdir().unwrap();
    let fake = temp.path().join("native");
    fs::write(&fake, b"Version 7.0.2\n").unwrap();
    assert!(!eof_capability(&fake, &initialized));
    fs::write(&fake, b"Version 7.0.2\n\0\xff").unwrap();
    assert!(!eof_capability(&fake, &initialized));
    assert!(!eof_capability(&temp.path().join("absent"), &initialized));
    for digest in [
        "",
        "4f2de678",
        &"f".repeat(64),
        vize_l0::cstr!("{LINUX_X64}00").as_str(),
    ] {
        assert!(!known_binary(digest));
    }
}

#[cfg(unix)]
#[test]
fn null_ack_precedes_eof_foreign_exit_is_preserved_and_failed_ack_still_closes() {
    use super::close;
    use corsa::runtime::block_on;
    use corsa_lsp::{CorsaError, LspClient, LspSpawnConfig};
    use serde_json::Value;
    use std::os::unix::fs::PermissionsExt;

    for (shutdown_to_eof, mode) in [
        (true, "ok"),
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
            "ok" => {
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
