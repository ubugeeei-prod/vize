//! Authenticate an exec wrapper through its still-owned Linux process image.

use corsa_lsp::process::{ProcessObserver, ProcessReceipt};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, Metadata},
    io::Read,
    os::unix::fs::MetadataExt,
    path::Path,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Life {
    pid: u32,
    parent: u32,
    birth: u64,
}

fn life(pid: u32, stat: &str) -> Option<Life> {
    let (observed_pid, fields) = stat.split_once('(')?;
    if observed_pid.trim().parse::<u32>().ok()? != pid {
        return None;
    }
    let (_, fields) = fields.rsplit_once(')')?;
    let fields: Vec<_> = fields.split_whitespace().collect();
    if !matches!(fields.first().copied()?, "R" | "S" | "D" | "T" | "t" | "I") {
        return None;
    }
    let parent = fields.get(1)?.parse().ok()?;
    let birth = fields.get(19)?.parse().ok()?;
    if parent != std::process::id() || birth == 0 {
        return None;
    }
    Some(Life { pid, parent, birth })
}

fn live_pid(receipts: &[ProcessReceipt]) -> Option<u32> {
    let [receipt] = receipts else { return None };
    (!receipt.completed
        && !receipt.stderr_complete
        && receipt.stderr_error.is_none()
        && receipt.exit_status.is_none()
        && receipt.process_error.is_none()
        && !receipt.timed_out
        && !receipt.kill_requested
        && !receipt.reaped
        && receipt.pid != 0)
        .then_some(receipt.pid)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ImageIdentity {
    device: u64,
    inode: u64,
    length: u64,
    modified: (i64, i64),
    changed: (i64, i64),
}

fn image_identity(metadata: &Metadata) -> Option<ImageIdentity> {
    metadata.is_file().then_some(ImageIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
        length: metadata.len(),
        modified: (metadata.mtime(), metadata.mtime_nsec()),
        changed: (metadata.ctime(), metadata.ctime_nsec()),
    })
}

fn digest_image(file: &mut File) -> Option<vize_l0::String> {
    let mut digest = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(length) => digest.update(buffer.get(..length)?),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return None,
        }
    }
    Some(
        digest
            .finalize()
            .iter()
            .map(|byte| vize_l0::cstr!("{byte:02x}"))
            .collect(),
    )
}

fn image_digest(pid: u32) -> Option<vize_l0::String> {
    let directory = Path::new("/proc").join(vize_l0::cstr!("{pid}").as_str());
    let before = life(pid, &fs::read_to_string(directory.join("stat")).ok()?)?;
    let executable = directory.join("exe");
    let mut image = File::open(&executable).ok()?;
    let identity = image_identity(&image.metadata().ok()?)?;
    let digest = digest_image(&mut image)?;
    if image_identity(&image.metadata().ok()?)? != identity
        || image_identity(&fs::metadata(&executable).ok()?)? != identity
        || life(pid, &fs::read_to_string(directory.join("stat")).ok()?)? != before
    {
        return None;
    }
    Some(digest)
}

pub(super) fn eof_capability(observer: &ProcessObserver) -> bool {
    let Some(pid) = live_pid(&observer.receipts()) else {
        return false;
    };
    let Some(digest) = image_digest(pid) else {
        return false;
    };
    live_pid(&observer.receipts()) == Some(pid) && super::known_binary(digest.as_str())
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::disallowed_macros,
        reason = "physical process fixtures use std strings"
    )]
    use super::*;
    use std::{os::unix::fs::PermissionsExt, process::ExitStatus, time::Duration};

    fn capture(name: &str, value: serde_json::Value) {
        if let Some(root) = std::env::var_os("VIZE_NATIVE_OWNED_IMAGE_CAPTURE") {
            fs::create_dir_all(&root).unwrap();
            fs::write(
                Path::new(&root).join(name),
                serde_json::to_vec(&value).unwrap(),
            )
            .unwrap();
        }
    }

    fn complete_receipt(row: &ProcessReceipt) -> serde_json::Value {
        use std::os::unix::process::ExitStatusExt;
        serde_json::json!({"pid": row.pid, "completed": row.completed,
            "stderr": row.stderr, "stderr_complete": row.stderr_complete,
            "stderr_error": row.stderr_error, "exit_status": row.exit_status.map(ExitStatus::into_raw),
            "process_error": row.process_error, "timed_out": row.timed_out,
            "kill_requested": row.kill_requested, "reaped": row.reaped,
            "elapsed_nanos": vize_l0::cstr!("{}", row.elapsed.as_nanos())})
    }

    fn receipt() -> ProcessReceipt {
        ProcessReceipt {
            pid: 41,
            completed: false,
            stderr: vec![],
            stderr_complete: false,
            stderr_error: None,
            exit_status: None,
            process_error: None,
            timed_out: false,
            kill_requested: false,
            reaped: false,
            elapsed: Duration::ZERO,
        }
    }

    #[test]
    fn absent_ambiguous_completed_or_failed_ownership_never_authenticates_an_image() {
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(live_pid(&[receipt()]), Some(41));
        assert_eq!(live_pid(&[]), None);
        assert_eq!(live_pid(&[receipt(), receipt()]), None);
        let mut denied = Vec::new();
        for change in 0..9 {
            let mut row = receipt();
            match change {
                0 => row.completed = true,
                1 => row.stderr_complete = true,
                2 => row.stderr_error = Some("reader failed".into()),
                3 => row.exit_status = Some(ExitStatus::from_raw(0)),
                4 => row.process_error = Some("PID mismatch".into()),
                5 => row.timed_out = true,
                6 => row.kill_requested = true,
                7 => row.reaped = true,
                8 => row.pid = 0,
                _ => unreachable!(),
            }
            assert_eq!(
                live_pid(std::slice::from_ref(&row)),
                None,
                "change {change}"
            );
            denied.push(complete_receipt(&row));
        }
        assert!(!eof_capability(&ProcessObserver::default()));
        assert_eq!(image_digest(std::process::id()), None);
        assert_eq!(image_digest(u32::MAX), None);
        capture(
            "ownership.json",
            serde_json::json!({"original": complete_receipt(&receipt()),
                "wholeDeniedReceipts": denied, "absentObserver": false, "selfPid": std::process::id(),
            "selfImage": null, "missingPid": u32::MAX, "missingImage": null}),
        );
    }

    #[test]
    fn full_pid_parent_birth_and_live_state_are_required_even_with_parenthesized_names() {
        let stat = |state: &str, parent: u32, birth: u64| {
            format!(
                "41 (native (exec)) {state} {parent} 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 {birth} 0"
            )
        };
        let expected = Life {
            pid: 41,
            parent: std::process::id(),
            birth: 512,
        };
        assert_eq!(
            life(41, &stat("S", std::process::id(), 512)),
            Some(expected)
        );
        for value in [
            stat("Z", std::process::id(), 512),
            stat("X", std::process::id(), 512),
            stat("?", std::process::id(), 512),
            stat("S", std::process::id() + 1, 512),
            stat("S", std::process::id(), 0),
            "41 (native) S 1".into(),
        ] {
            assert_eq!(life(41, &value), None);
        }
        assert_eq!(life(42, &stat("S", std::process::id(), 512)), None);
        capture(
            "life.json",
            serde_json::json!({"pid": 41, "parent": std::process::id(),
            "birth": 512, "wholePositiveStat": stat("S", std::process::id(), 512),
            "deniedStates": ["Z", "X", "?"], "foreignParent": std::process::id()+1,
            "deniedBirth": 0, "deniedPid": 42}),
        );
    }

    #[test]
    fn an_actual_owned_foreign_image_and_a_reaped_child_do_not_get_native_capability() {
        use std::os::unix::process::ExitStatusExt;
        use std::process::{Command, Stdio};
        let mut child = Command::new("/bin/cat")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let pid = child.id();
        let stat = fs::read_to_string(vize_l0::cstr!("/proc/{pid}/stat").as_str()).unwrap();
        let digest = image_digest(pid).unwrap();
        drop(child.stdin.take());
        let terminal = child.wait().unwrap();
        assert!(!super::super::known_binary(digest.as_str()));
        assert!(terminal.success());
        assert_eq!(image_digest(pid), None);
        capture(
            "foreign.json",
            serde_json::json!({"pid": pid, "wholeLiveStat": stat,
            "imageSHA256": digest, "knownNative": false, "terminalRaw": terminal.into_raw(),
            "reapedImage": null, "killRequested": false}),
        );
    }

    #[test]
    fn actual_exec_wrapper_uses_only_its_owned_initialized_native_image() {
        use super::super::super::EditorLspSession;
        let required = std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_some();
        if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
            assert!(!required, "native image qualification cannot be disabled");
            return;
        }
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let backend = vize_carton::corsa_resolver::resolve_corsa_executable(
            vize_carton::corsa_resolver::CorsaResolveRequest {
                project_root: Some(workspace),
                ..Default::default()
            },
        )
        .ok();
        let Some(backend) = backend else {
            assert!(
                !required,
                "native image qualification requires the actual backend"
            );
            return;
        };
        let project = tempfile::tempdir().unwrap();
        let wrapper = project.path().join("owned-wrapper");
        let quoted = backend.to_str().unwrap().replace('\'', "'\\''");
        let original = include_str!(
            "../../../../../../tests/_fixtures/differential/typechecker/native-owned-exec/exec-wrapper.sh.txt"
        );
        let body = original.replace("__BACKEND__", &quoted);
        fs::write(&wrapper, &body).unwrap();
        fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).unwrap();
        let mut session =
            EditorLspSession::spawn(wrapper.to_str().unwrap(), project.path(), project.path())
                .unwrap();
        assert!(session.shutdown_to_eof);
        assert_eq!(
            session.native_initialize_response,
            super::super::known_contract()
        );
        let pid = live_pid(&session.process_observer.receipts()).unwrap();
        let stat = fs::read_to_string(vize_l0::cstr!("/proc/{pid}/stat").as_str()).unwrap();
        let before = session.process_observer.receipts();
        let digest = image_digest(pid).unwrap();
        assert!(super::super::known_binary(digest.as_str()));
        let mut unknown = session.native_initialize_response.clone();
        unknown["capabilities"]["positionEncoding"] = serde_json::json!("utf-8");
        assert!(!super::super::eof_capability(
            &wrapper,
            &unknown,
            &session.process_observer
        ));
        assert!(!super::super::eof_capability(
            &wrapper,
            &session.native_initialize_response,
            &ProcessObserver::default()
        ));
        session.shutdown().unwrap();
        let rows = session.process_observer.receipts();
        let [row] = rows.as_slice() else {
            panic!("expected one owned native")
        };
        assert_eq!(row.pid, pid);
        assert!(row.completed && row.reaped && row.stderr_complete);
        assert!(row.stderr.is_empty());
        assert!(row.stderr_error.is_none() && row.process_error.is_none());
        assert!(!row.timed_out && !row.kill_requested);
        assert!(row.exit_status.unwrap().success());
        assert!(!eof_capability(&session.process_observer));
        assert_eq!(image_digest(pid), None);
        assert_eq!(fs::read_to_string(wrapper).unwrap(), body);
        capture(
            "native.json",
            serde_json::json!({"pid": pid, "wholeLiveStat": stat,
            "imageSHA256": digest, "nativePath": backend, "wholeWrapper": body,
            "wholeInitialize": session.native_initialize_response,
            "wholeChangedInitialize": unknown, "before": complete_receipt(&before[0]),
            "terminal": complete_receipt(row), "knownNative": true,
            "unknownInitialize": false, "absentObserver": false, "reapedImage": null}),
        );
    }
}
