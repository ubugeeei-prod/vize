//! A spent transaction budget must not become a new per-receive timeout.

use super::*;

#[test]
fn an_expired_absolute_deadline_refuses_even_an_already_buffered_matching_reply() {
    let (sender, messages) = mpsc::channel();
    let reply = serde_json::json!({"jsonrpc":"2.0","id":1,"result":null});
    sender.send(Ok(reply.clone())).unwrap();
    let mut process = LspProcess {
        child: None,
        stdin: None,
        messages,
        published_diagnostics: Arc::new(Mutex::new(Vec::new())),
        stdout_reader: None,
        stderr_reader: None,
        stderr: Arc::new(Mutex::new(Vec::new())),
        status: None,
        capture: None,
        passive: None,
        evidence: Evidence::default(),
        stdout_joined: None,
        stderr_joined: None,
    };
    let deadline = Instant::now();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        process.recv_matching_until(deadline, |_| true)
    }))
    .unwrap_err();
    let message = panic.downcast_ref::<String>().unwrap();
    assert_eq!(
        message,
        "timed out waiting for LSP message; seen: []\nLSP process status: None\nLSP stderr:\n"
    );
    assert_eq!(
        process.messages.try_recv().unwrap().unwrap(),
        reply,
        "a renewed timeout would have consumed and accepted this buffered reply"
    );
}
