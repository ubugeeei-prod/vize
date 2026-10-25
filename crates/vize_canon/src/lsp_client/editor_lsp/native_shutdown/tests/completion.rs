use corsa::runtime::block_on;
use corsa_lsp::{LspClient, LspSpawnConfig};
use serde_json::Value;
use serde_json::json;
use std::fs;
use std::os::unix::fs::PermissionsExt;

#[test]
fn buffered_completion_and_unrelated_notifications_preserve_the_whole_transcript() {
    let temp = tempfile::tempdir().unwrap();
    let log = temp.path().join("whole-packets.jsonl");
    let client = spawn(temp.path(), &log, "unrelated");
    block_on(super::super::close(&client, true)).unwrap();
    let completion = "handled method 'shutdown' (1) in 6.071µs";
    let expected = vec![
        json!({"direction":"request","wholePacket":{"jsonrpc":"2.0","id":1,"method":"shutdown"}}),
        json!({"direction":"response","wholePacket":{"jsonrpc":"2.0","id":1,"result":null}}),
        notification(json!({"type":3,"message":"handled method 'textDocument/hover' (1) in 1ns"})),
        notification(json!({"type":2,"message":completion})),
        notification(json!({"type":"3","message":completion})),
        notification(json!({"type":3,"message":completion})),
        json!({"event":"stdin-eof"}),
    ];
    assert_eq!(packets(&log), expected);
}

#[test]
fn malformed_missing_stale_and_eof_completion_still_close_the_owned_process() {
    struct Initialize;
    impl lsp_types::request::Request for Initialize {
        type Params = Value;
        type Result = Value;
        const METHOD: &'static str = "initialize";
    }
    for mode in ["malformed", "missing", "stale", "eof"] {
        let temp = tempfile::tempdir().unwrap();
        let log = temp.path().join("whole-packets.jsonl");
        let client = spawn(temp.path(), &log, mode);
        let mut expected = Vec::new();
        let shutdown_id = if mode == "stale" {
            // This notification is fully received before close creates its
            // fresh subscription. It cannot authorize the later Shutdown.
            let historical = client.subscribe();
            assert_eq!(
                block_on(client.request::<Initialize>(json!({}))).unwrap(),
                json!({"capabilities":{}})
            );
            let corsa_lsp::jsonrpc::InboundEvent::Notification { method, params } =
                historical.recv().unwrap()
            else {
                panic!("stale notification expected")
            };
            let stale = json!({"type":3,"message":"handled method 'shutdown' (99) in 1ns"});
            assert_eq!(
                (method.as_str(), params),
                ("window/logMessage", stale.clone())
            );
            expected.extend([
                json!({"direction":"request","wholePacket":{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}}),
                notification(stale),
                json!({"direction":"response","wholePacket":{"jsonrpc":"2.0","id":1,"result":{"capabilities":{}}}}),
            ]);
            2
        } else {
            1
        };
        let error = block_on(super::super::close(&client, true)).unwrap_err();
        if mode == "malformed" {
            assert_eq!(
                error.to_string(),
                "protocol error: Malformed authenticated native shutdown completion: {\"message\":\"handled method 'shutdown' (1) in invalid\",\"type\":3}"
            );
        } else {
            assert_eq!(
                error.to_string(),
                "timeout: native shutdown completion timed out after 2000 ms"
            );
        }
        expected.extend([
            json!({"direction":"request","wholePacket":{"jsonrpc":"2.0","id":shutdown_id,"method":"shutdown"}}),
            json!({"direction":"response","wholePacket":{"jsonrpc":"2.0","id":shutdown_id,"result":null}}),
        ]);
        if mode == "malformed" {
            expected.push(notification(
                json!({"type":3,"message":"handled method 'shutdown' (1) in invalid"}),
            ));
        }
        expected.push(json!({"event":"stdin-eof"}));
        assert_eq!(packets(&log), expected, "{mode}");
        // SDK close already reaped the process; repeating it is idempotent.
        block_on(client.close()).unwrap();
    }
}

fn spawn(root: &std::path::Path, log: &std::path::Path, mode: &str) -> LspClient {
    let script = root.join("peer.ts");
    let source = include_str!("../peer.ts.txt")
        .replace("LOG_PATH", &serde_json::to_string(log).unwrap())
        .replace("REPLY_MODE", &serde_json::to_string(mode).unwrap());
    fs::write(&script, source).unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    block_on(LspClient::spawn(LspSpawnConfig::new(&script))).unwrap()
}

fn notification(params: Value) -> Value {
    json!({"direction":"notification","wholePacket":{"jsonrpc":"2.0","method":"window/logMessage","params":params}})
}

fn packets(log: &std::path::Path) -> Vec<Value> {
    fs::read_to_string(log)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
