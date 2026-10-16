use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;
use vize_l0::cstr;

pub(super) fn capture_root(
    workspace: &Path,
    profile: Option<&str>,
) -> Result<Option<PathBuf>, String> {
    match profile {
        None => Ok(None),
        Some(profile @ ("pr" | "full")) => Ok(Some(
            workspace.join(cstr!("target/nextest/{profile}/native-proxy-termination")),
        )),
        Some(profile) => {
            Err(cstr!("unknown native proxy nextest capture profile: {profile:?}").into())
        }
    }
}

#[test]
fn capture_profiles_refuse_foreign_artifact_paths() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for profile in ["pr", "full"] {
        let root = capture_root(&workspace, Some(profile))
            .expect("known artifact profile")
            .expect("capture root");
        assert_eq!(
            root,
            workspace.join(cstr!("target/nextest/{profile}/native-proxy-termination"))
        );
    }
    assert_eq!(
        capture_root(&workspace, None).expect("ordinary Cargo has no nextest artifact"),
        None
    );
    for unknown in ["", "ci", "full/../../foreign", "pr/foreign", "../full"] {
        assert_eq!(
            capture_root(&workspace, Some(unknown)),
            Err(cstr!("unknown native proxy nextest capture profile: {unknown:?}").into())
        );
    }
}

pub(super) fn assert_native_termination(trace_dir: &Path, pid: &str) -> Result<(), String> {
    let path = trace_dir.join(cstr!("server-{pid}.stderr.lifecycle.json"));
    let whole = fs::read(&path).map_err(|error| cstr!("read {}: {error}", path.display()))?;
    let packet: Value = serde_json::from_slice(&whole)
        .map_err(|error| cstr!("parse whole native lifecycle {}: {error}", path.display()))?;
    let refused = || -> String {
        cstr!(
            "incomplete native EOF lifecycle {}: {packet}",
            path.display()
        )
        .into()
    };
    for field in [
        "candidate",
        "initialized",
        "acknowledged",
        "translated",
        "nativeInputEofAfterExit",
        "serverEof",
    ] {
        if packet[field].as_u64() != Some(1) {
            return Err(refused());
        }
    }
    let native_pid = packet["nativePid"]
        .as_u64()
        .filter(|pid| *pid > 0)
        .ok_or_else(refused)?;
    if !(packet["clientEof"].as_u64() == Some(1)
        || packet["clientReaderClosedAfterExit"].as_u64() == Some(1))
        || packet["nativeWaitStatus"].as_u64() != Some(0)
        || packet["reapedNativePid"].as_u64() != Some(native_pid)
        || packet["native"]["pid"].as_u64() != Some(native_pid)
        || packet["nativeBeforeExit"]["pid"].as_u64() != Some(native_pid)
        || packet["errors"]
            .as_array()
            .is_none_or(|errors| !errors.is_empty())
        || packet["native"]["image"] != packet["nativeBeforeExit"]["image"]
        || packet["native"]["birth"] != packet["nativeBeforeExit"]["birth"]
    {
        return Err(refused());
    }
    #[cfg(target_os = "linux")]
    if packet["native"]["birth"]
        .as_u64()
        .is_none_or(|birth| birth == 0)
        || packet["native"]["parent"]
            .as_u64()
            .is_none_or(|parent| parent == 0)
        || packet["native"]["parent"] != packet["nativeBeforeExit"]["parent"]
    {
        return Err(refused());
    }
    let expected: Value = serde_json::from_str(include_str!(
        "../../../../../../tests/fixtures/native-proxy-initialize.json"
    ))
    .map_err(|error| error.to_string())?;
    if packet["wholeInitializeResponse"]["result"] != expected
        || packet["wholeInitializeResponse"]["id"] != packet["wholeInitializeRequest"]["id"]
        || packet["wholeShutdownResponse"]["id"] != packet["wholeShutdownRequest"]["id"]
        || !packet["wholeShutdownResponse"]["result"].is_null()
        || packet["wholeShutdownResponse"].get("error").is_some()
        || packet["wholeExitNotification"]
            != serde_json::json!({"jsonrpc": "2.0", "method": "exit"})
    {
        return Err(refused());
    }
    for (side, name) in [
        ("client", cstr!("client-{pid}.raw")),
        ("server", cstr!("server-{pid}.raw")),
    ] {
        let frames = packet["frames"][side].as_array().ok_or_else(refused)?;
        let mut observed = Vec::new();
        for frame in frames {
            let hex = frame["wholeFrameHex"].as_str().ok_or_else(refused)?;
            if hex.len() % 2 != 0 {
                return Err(refused());
            }
            for pair in hex.as_bytes().as_chunks::<2>().0 {
                let pair = std::str::from_utf8(pair).map_err(|_| refused())?;
                observed.push(u8::from_str_radix(pair, 16).map_err(|_| refused())?);
            }
        }
        let raw = fs::read(trace_dir.join(name)).map_err(|error| error.to_string())?;
        if observed != raw {
            return Err(refused());
        }
    }
    Ok(())
}

#[test]
fn native_proxy_preserves_whole_frames_and_foreign_process_refusals() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new("perl")
        .arg(root.join("tests/fixtures/native-proxy-lifecycle-laws.pl"))
        .arg(root.join("tests/fixtures"))
        .arg(root.join("src/ide/rename/corsa_session_tests/harness/protocol/proxy.rs"))
        .output()
        .expect("execute complete proxy fixture controls");
    assert!(
        output.status.success(),
        "whole framing/foreign native controls failed: {output:?}"
    );
}
