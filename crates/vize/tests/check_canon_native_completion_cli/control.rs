use super::{EOF, digest};
use serde_json::{Value, json};
use std::{fs, path::Path};
use vize_l0::cstr;

pub(super) fn assert_control(root: &Path, native: &Path, mode: &str) {
    let receipt: Value =
        serde_json::from_slice(&fs::read(root.join("control.json")).unwrap()).unwrap();
    let wrapper_pid = receipt["wrapperPid"].as_u64().unwrap();
    let native_pid = receipt["nativePid"].as_u64().unwrap();
    assert_ne!(wrapper_pid, native_pid);
    assert_ne!(wrapper_pid, 0);
    assert_ne!(native_pid, 0);
    let wrapper_exit = match mode {
        "nonzero" => json!(17),
        "sigsegv" | "timeout" => Value::Null,
        _ => json!(0),
    };
    assert_eq!(receipt["nativeExit"], 0);
    assert_eq!(receipt["wrapperExit"], wrapper_exit);
    assert_eq!(receipt["errors"], json!([]));
    for flag in [
        "nativeReaped",
        "stdinComplete",
        "stdoutComplete",
        "stderrComplete",
        "shutdownAcknowledged",
        "exitTranslatedToNativeEof",
    ] {
        assert_eq!(receipt[flag], true, "{flag}");
    }
    for flag in ["timedOut", "killRequested"] {
        assert_eq!(receipt[flag], false, "{flag}");
    }
    assert_eq!(receipt["native"], json!(native));
    assert_eq!(
        receipt["nativeSha256"],
        json!(digest(&fs::read(native).unwrap()))
    );
    assert_eq!(
        receipt["wrapperSourceSha256"],
        json!(digest(&fs::read(root.join("native-control.py")).unwrap()))
    );
    let exit_text = if wrapper_exit.is_null() {
        "None".into()
    } else {
        cstr!("{wrapper_exit}")
    };
    assert_eq!(fs::read_to_string(root.join("native-child.txt")).unwrap(), cstr!("wrapper-pid={wrapper_pid}\nnative-pid={native_pid}\nnative-exit=0\nwrapper-exit={exit_text}\n").as_str());
    assert_eq!(receipt["mode"], mode);
    let wrapper_input = fs::read(root.join("wrapper.stdin.bin")).unwrap();
    let split = wrapper_input
        .windows(4)
        .position(|bytes| bytes == b"\r\n\r\n")
        .unwrap();
    let header = std::str::from_utf8(&wrapper_input[..split]).unwrap();
    let length: usize = header
        .strip_prefix("Content-Length: ")
        .unwrap()
        .parse()
        .unwrap();
    let initialize: Value = serde_json::from_slice(&wrapper_input[split + 4..][..length]).unwrap();
    assert_eq!(initialize["method"], "initialize");
    assert_eq!(receipt["cwd"], initialize["params"]["rootPath"]);
    assert_eq!(receipt["parentPid"], initialize["params"]["processId"]);
    assert_eq!(receipt["argv"], json!(["--lsp", "--stdio"]));
    assert_eq!(receipt["nativeSignal"], Value::Null);
    let files: Vec<_> = receipt["rawFiles"]
        .as_object()
        .unwrap()
        .keys()
        .map(|key| key.as_str())
        .collect();
    assert_eq!(
        files,
        [
            "native.stderr.bin",
            "native.stdin.bin",
            "native.stdout.bin",
            "translated-exit.frames.bin",
            "wrapper.injected-stderr.bin",
            "wrapper.stderr.bin",
            "wrapper.stdin.bin",
            "wrapper.stdout.bin"
        ]
    );
    for (file, metadata) in receipt["rawFiles"].as_object().unwrap() {
        let bytes = fs::read(root.join(file)).unwrap();
        assert_eq!(metadata["bytes"], json!(bytes.len()));
        assert_eq!(metadata["sha256"], json!(digest(&bytes)));
    }
    assert_eq!(fs::read(root.join("native.stderr.bin")).unwrap(), b"");
    assert_eq!(
        fs::read(root.join("native.stdout.bin")).unwrap(),
        fs::read(root.join("wrapper.stdout.bin")).unwrap()
    );
    let prefix: &[u8] = match mode {
        "nonutf8" => b"wrapper\0\xffcontrol\n",
        "unexpected" => b"unexpected native stderr\n",
        "nonzero" | "sigsegv" | "timeout" => b"",
        _ => panic!("unknown wrapper mode"),
    };
    let expected_stderr = [prefix, EOF].concat();
    assert_eq!(
        fs::read(root.join("wrapper.stderr.bin")).unwrap(),
        expected_stderr
    );
    assert_eq!(
        fs::read(root.join("wrapper.injected-stderr.bin")).unwrap(),
        expected_stderr
    );
    let translated = fs::read(root.join("translated-exit.frames.bin")).unwrap();
    let frame = std::str::from_utf8(&translated).unwrap();
    let (header, body) = frame.split_once("\r\n\r\n").unwrap();
    assert_eq!(header, cstr!("Content-Length: {}", body.len()).as_str());
    assert_eq!(
        serde_json::from_str::<Value>(body).unwrap(),
        json!({"jsonrpc":"2.0", "method":"exit"})
    );
    let mut stdin = fs::read(root.join("native.stdin.bin")).unwrap();
    stdin.extend(translated);
    assert_eq!(stdin, fs::read(root.join("wrapper.stdin.bin")).unwrap());
}
