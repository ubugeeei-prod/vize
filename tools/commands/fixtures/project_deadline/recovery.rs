use serde_json::{Value, json};
use sha2::{Digest, Sha256};
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

fn identity(meta: &fs::Metadata) -> Result<(u64, u64), String> {
    #[cfg(unix)]
    {
        Ok((meta.dev(), meta.ino()))
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        Err("mutation identity recovery currently requires Unix".into())
    }
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn atomic_json(path: &Path, value: &Value) -> Result<(), String> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let pending = path.with_extension(format!("pending-{}-{nonce}", std::process::id()));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&pending)
            .map_err(|e| e.to_string())?;
        serde_json::to_writer(&mut file, value).map_err(|e| e.to_string())?;
        file.write_all(b"\n")
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
        fs::rename(&pending, path).map_err(|e| e.to_string())?;
        fs::File::open(path.parent().ok_or("missing evidence directory")?)
            .and_then(|d| d.sync_all())
            .map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(pending);
    }
    result
}

pub struct MutationLease {
    directory: PathBuf,
}
impl MutationLease {
    pub fn begin(
        directory: &Path,
        fixture: &Path,
        source: &Path,
        original: &[u8],
        broken: &[u8],
    ) -> Result<Self, String> {
        let active = directory.join("active-mutation.json");
        match fs::symlink_metadata(&active) {
            Ok(_) => return Err("previous mutation lease is still active".into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
        let physical = source.canonicalize().map_err(|e| e.to_string())?;
        let fixture = fixture.canonicalize().map_err(|e| e.to_string())?;
        let meta = fs::symlink_metadata(source).map_err(|e| e.to_string())?;
        if physical != source || !physical.starts_with(&fixture) || !meta.is_file() {
            return Err("mutation source must be an owned physical fixture file".into());
        }
        if fs::read(source).map_err(|e| e.to_string())? != original {
            return Err("mutation source changed before acquiring its lease".into());
        }
        let (device, inode) = identity(&meta)?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let original_file = format!("mutation-{nonce}-original.bin");
        let broken_file = format!("mutation-{nonce}-broken.bin");
        for (name, bytes) in [(&original_file, original), (&broken_file, broken)] {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(directory.join(name))
                .map_err(|e| e.to_string())?;
            file.write_all(bytes)
                .and_then(|_| file.sync_all())
                .map_err(|e| e.to_string())?;
        }
        atomic_json(
            &active,
            &json!({"schema":"vize.typecheckMutationLease", "version":1,
            "fixture":fixture, "source":physical, "device":device, "inode":inode,
            "originalFile":original_file,"brokenFile":broken_file,"originalSha256":digest(original), "brokenSha256":digest(broken)}),
        )?;
        Ok(Self {
            directory: directory.to_path_buf(),
        })
    }
    pub fn restore(self) -> Result<(), String> {
        recover(&self.directory).map(|_| ())
    }
}

/// Called only after every supervised writer is stopped, or by the sole worker.
pub fn recover(directory: &Path) -> Result<bool, String> {
    let active = directory.join("active-mutation.json");
    match fs::symlink_metadata(&active) {
        Ok(meta) if meta.is_file() => {}
        Ok(_) => return Err("active mutation journal is not a physical file".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.to_string()),
    }
    let value: Value = serde_json::from_slice(&fs::read(&active).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let field = |name: &str| {
        value[name]
            .as_str()
            .ok_or_else(|| format!("invalid mutation lease {name}"))
    };
    let source = PathBuf::from(field("source")?);
    let fixture = PathBuf::from(field("fixture")?);
    let meta = fs::symlink_metadata(&source).map_err(|e| e.to_string())?;
    let (device, inode) = identity(&meta)?;
    if !meta.is_file()
        || source.canonicalize().map_err(|e| e.to_string())? != source
        || !source.starts_with(&fixture)
        || value["device"].as_u64() != Some(device)
        || value["inode"].as_u64() != Some(inode)
    {
        return Err("mutation recovery refused a replaced or unowned source identity".into());
    }
    let read_snapshot = |name: &str| -> Result<Vec<u8>, String> {
        let name = field(name)?;
        if !name.starts_with("mutation-") || name.contains('/') || name.contains('\\') {
            return Err("invalid mutation snapshot name".into());
        }
        let path = directory.join(name);
        if !fs::symlink_metadata(&path)
            .map_err(|e| e.to_string())?
            .is_file()
            || path.canonicalize().map_err(|e| e.to_string())? != path
        {
            return Err("mutation snapshot is not an owned physical file".into());
        }
        fs::read(path).map_err(|e| e.to_string())
    };
    let original = read_snapshot("originalFile")?;
    let broken = read_snapshot("brokenFile")?;
    if digest(&original) != field("originalSha256")? || digest(&broken) != field("brokenSha256")? {
        return Err("mutation recovery bytes disagree with the active lease".into());
    }
    // Open without truncation, then authenticate that exact descriptor before writing.
    // A path replacement must never redirect restoration into a foreign file.
    let mut options = fs::OpenOptions::new();
    options.read(true).write(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NOFOLLOW);
    let mut file = options.open(&source).map_err(|e| e.to_string())?;
    if identity(&file.metadata().map_err(|e| e.to_string())?)? != (device, inode) {
        return Err("mutation recovery refused a replaced open source identity".into());
    }
    let mut current = Vec::new();
    file.read_to_end(&mut current).map_err(|e| e.to_string())?;
    if current != original
        && current != broken
        && !original.starts_with(&current)
        && !broken.starts_with(&current)
    {
        return Err("mutation recovery refused unexpected authored bytes".into());
    }
    file.seek(SeekFrom::Start(0))
        .and_then(|_| file.set_len(0))
        .map_err(|e| e.to_string())?;
    file.write_all(&original)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())?;
    if identity(&fs::symlink_metadata(&source).map_err(|e| e.to_string())?)? != (device, inode)
        || source.canonicalize().map_err(|e| e.to_string())? != source
        || fs::read(&source).map_err(|e| e.to_string())? != original
    {
        return Err("mutation restoration did not retain every original byte".into());
    }
    atomic_json(
        &directory.join("mutation-recovery.json"),
        &json!({"source":source,"originalSha256":digest(&original),"restored":true}),
    )?;
    fs::remove_file(active).map_err(|e| e.to_string())?;
    fs::File::open(directory)
        .and_then(|d| d.sync_all())
        .map_err(|e| e.to_string())?;
    Ok(true)
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::super::{state_directory, supervise};
    use super::*;
    use std::{
        process::{Child, Command},
        time::{Duration, Instant},
    };
    struct Sandbox(PathBuf);
    impl Sandbox {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "vize-deadline-test-{}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir(&root).unwrap();
            Self(root)
        }
        fn state(&self) -> PathBuf {
            state_directory(
                &self.0,
                &json!({"parentPid":std::process::id(),"test":true}),
            )
            .unwrap()
        }
    }
    impl Drop for Sandbox {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    struct Sentinel(Child);
    impl Drop for Sentinel {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    fn node(source: &str) -> Command {
        let mut c = Command::new("node");
        c.args(["-e", source]);
        c
    }
    fn supervise_ms(command: &mut Command, state: &Path, ms: u64) -> bool {
        let started = Instant::now();
        let result = supervise(command, state, started, Duration::from_millis(ms)).unwrap();
        assert!(
            started.elapsed() < Duration::from_secs(4),
            "real supervisor exceeded cleanup allowance"
        );
        result
    }
    #[test]
    fn real_cpu_worker_is_stopped_and_both_raw_streams_survive() {
        let sandbox = Sandbox::new();
        let state = sandbox.state();
        let mut command = node(
            "process.stdout.write('CPU phase started\\n');process.stderr.write('raw diagnostic\\n');while(true){}",
        );
        assert!(supervise_ms(&mut command, &state, 400));
        assert_eq!(
            fs::read(state.join("producer-stdout.bin")).unwrap(),
            b"CPU phase started\n"
        );
        assert_eq!(
            fs::read(state.join("producer-stderr.bin")).unwrap(),
            b"raw diagnostic\n"
        );
        let evidence: Value =
            serde_json::from_slice(&fs::read(state.join("supervisor.json")).unwrap()).unwrap();
        assert_eq!(evidence["timedOut"], true);
        assert_eq!(evidence["cleanup"]["allWritersStopped"], true);
    }
    #[test]
    fn sequential_real_children_share_one_parent_deadline() {
        let sandbox = Sandbox::new();
        let state = sandbox.state();
        let mut command = node(
            "const {spawnSync}=require('node:child_process');for(const [phase,delay] of [['first',200],['second',600]]){console.log(phase+' start');spawnSync(process.execPath,['-e','setTimeout(()=>{},'+delay+')'],{stdio:'inherit'});console.log(phase+' done');}",
        );
        assert!(supervise_ms(&mut command, &state, 600));
        let output = fs::read_to_string(state.join("producer-stdout.bin")).unwrap();
        assert_eq!(output, "first start\nfirst done\nsecond start\n");
    }
    #[test]
    fn separate_owned_process_groups_die_and_unrelated_sentinel_survives() {
        let sandbox = Sandbox::new();
        let state = sandbox.state();
        let mut sentinel = Sentinel(node("setInterval(()=>{},1000)").spawn().unwrap());
        let mut command = Command::new("bash");
        command.args([
            "-c",
            "set -m; node -e 'process.on(\"SIGTERM\",()=>{});setInterval(()=>{},1000)' & wait",
        ]);
        assert!(supervise_ms(&mut command, &state, 400));
        assert!(sentinel.0.try_wait().unwrap().is_none());
        let evidence: Value =
            serde_json::from_slice(&fs::read(state.join("supervisor.json")).unwrap()).unwrap();
        let groups = evidence["cleanup"]["groups"].as_array().unwrap();
        assert!(
            groups.len() >= 2,
            "fixture must really create a separate process group"
        );
        assert_eq!(evidence["cleanup"]["allWritersStopped"], true);
    }
    #[test]
    fn deadline_recovers_every_original_authored_byte_after_stopping_writers() {
        let sandbox = Sandbox::new();
        let state = sandbox.state();
        let source = sandbox.0.join("Edited.vue");
        let original = b"<template>\r\n\xe4\xb8\x96\xe7\x95\x8c\r\n</template>\r\n";
        let broken = b"<template>broken</template>\n";
        fs::write(&source, original).unwrap();
        MutationLease::begin(&state, &sandbox.0, &source, original, broken).unwrap();
        let mut command = node(
            "require('node:fs').writeFileSync(process.argv[1],'<template>broken</template>\\n');while(true){}",
        );
        command.arg(&source);
        assert!(supervise_ms(&mut command, &state, 400));
        assert_eq!(fs::read(&source).unwrap(), original);
        assert!(!state.join("active-mutation.json").exists());
        let result: Value =
            serde_json::from_slice(&fs::read(state.join("mutation-recovery.json")).unwrap())
                .unwrap();
        assert_eq!(result["originalSha256"], digest(original));
    }
    #[test]
    fn worker_refuses_restoration_while_a_separate_group_can_write() {
        if let Some(directory) = std::env::var_os("VIZE_DEADLINE_RESTORE_TEST_STATE") {
            let directory = PathBuf::from(directory);
            let source =
                PathBuf::from(std::env::var_os("VIZE_DEADLINE_RESTORE_TEST_SOURCE").unwrap());
            let fixture = source.parent().unwrap();
            let context = super::super::ProjectContext::worker(directory.clone(), 3000).unwrap();
            let lease = context
                .mutation(fixture, &source, b"original", b"broken")
                .unwrap();
            fs::write(&source, b"broken").unwrap();
            let ready = directory.join("separate-writer.pid");
            let mut writer = Command::new("bash");
            writer.args(["-c", "set -m; node -e 'const fs=require(\"node:fs\");fs.writeFileSync(process.argv[2],String(process.pid));setInterval(()=>fs.writeFileSync(process.argv[1],\"broken\"),10)' \"$1\" \"$2\" & wait", "owned-writer"]);
            writer.arg(&source).arg(&ready).spawn().unwrap();
            let started = Instant::now();
            while !ready.exists() {
                assert!(started.elapsed() < Duration::from_secs(2));
                std::thread::sleep(Duration::from_millis(5));
            }
            let error = context.restore(lease).unwrap_err();
            assert!(error.contains("owned checker descendants remain"));
            assert!(directory.join("active-mutation.json").is_file());
            atomic_json(
                &directory.join("worker-refusal.json"),
                &json!({"error":error}),
            )
            .unwrap();
            loop {
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        let sandbox = Sandbox::new();
        let state = sandbox.state();
        let source = sandbox.0.join("Edited.vue");
        fs::write(&source, b"original").unwrap();
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .arg("worker_refuses_restoration_while_a_separate_group_can_write")
            .arg("--nocapture")
            .env("VIZE_DEADLINE_RESTORE_TEST_STATE", &state)
            .env("VIZE_DEADLINE_RESTORE_TEST_SOURCE", &source);
        assert!(supervise_ms(&mut command, &state, 1500));
        assert!(state.join("worker-refusal.json").is_file());
        assert!(state.join("separate-writer.pid").is_file());
        assert_eq!(fs::read(source).unwrap(), b"original");
        assert!(!state.join("active-mutation.json").exists());
        let result: Value =
            serde_json::from_slice(&fs::read(state.join("supervisor.json")).unwrap()).unwrap();
        assert_eq!(result["cleanup"]["allWritersStopped"], true);
        assert!(result["cleanup"]["groups"].as_array().unwrap().len() >= 2);
    }
    #[test]
    fn replaced_source_identity_refuses_recovery_without_overwriting_foreign_bytes() {
        let sandbox = Sandbox::new();
        let state = sandbox.state();
        let source = sandbox.0.join("Edited.vue");
        fs::write(&source, b"original").unwrap();
        MutationLease::begin(&state, &sandbox.0, &source, b"original", b"broken").unwrap();
        let replacement = sandbox.0.join("replacement");
        fs::write(&replacement, b"foreign").unwrap();
        fs::rename(replacement, &source).unwrap();
        assert!(recover(&state).unwrap_err().contains("replaced or unowned"));
        assert_eq!(fs::read(source).unwrap(), b"foreign");
        assert!(state.join("active-mutation.json").exists());
    }
    #[test]
    fn normal_worker_keeps_complete_raw_bytes_and_does_not_become_a_timeout() {
        let sandbox = Sandbox::new();
        let state = sandbox.state();
        let mut command = node(
            "process.stdout.write('whole\\r\\noutput');process.stderr.write('whole error\\n')",
        );
        assert!(!supervise_ms(&mut command, &state, 3000));
        assert_eq!(
            fs::read(state.join("producer-stdout.bin")).unwrap(),
            b"whole\r\noutput"
        );
        assert_eq!(
            fs::read(state.join("producer-stderr.bin")).unwrap(),
            b"whole error\n"
        );
    }
    #[test]
    fn failed_supervisor_receipt_still_restores_active_mutation() {
        let sandbox = Sandbox::new();
        let state = sandbox.state();
        let source = sandbox.0.join("Edited.vue");
        fs::write(&source, b"original").unwrap();
        MutationLease::begin(&state, &sandbox.0, &source, b"original", b"broken").unwrap();
        fs::write(&source, b"broken").unwrap();
        fs::create_dir(state.join("supervisor.json")).unwrap();
        let mut command = node("while(true){}");
        let start = Instant::now();
        assert!(supervise(&mut command, &state, start, Duration::from_millis(400)).is_err());
        assert_eq!(fs::read(source).unwrap(), b"original");
        assert!(!state.join("active-mutation.json").exists());
    }
    #[test]
    fn unknown_authored_bytes_are_not_overwritten_by_recovery() {
        let sandbox = Sandbox::new();
        let state = sandbox.state();
        let source = sandbox.0.join("Edited.vue");
        fs::write(&source, b"original").unwrap();
        MutationLease::begin(&state, &sandbox.0, &source, b"original", b"broken").unwrap();
        fs::write(&source, b"foreign edit").unwrap();
        assert!(
            recover(&state)
                .unwrap_err()
                .contains("unexpected authored bytes")
        );
        assert_eq!(fs::read(source).unwrap(), b"foreign edit");
    }
    #[test]
    fn bounded_stream_retains_an_explicit_incomplete_prefix_and_refuses_success() {
        use std::{
            io::Read,
            sync::{
                Arc,
                atomic::{AtomicBool, Ordering},
            },
        };
        let sandbox = Sandbox::new();
        let output = sandbox.0.join("bounded.bin");
        let failed = Arc::new(AtomicBool::new(false));
        let error = super::super::retain_stream(
            std::io::repeat(0).take(64 * 1024 * 1024 + 1),
            &output,
            None,
            failed.clone(),
        )
        .unwrap_err();
        assert!(error.contains("incomplete"));
        assert!(failed.load(Ordering::Acquire));
        assert_eq!(fs::metadata(output).unwrap().len(), 64 * 1024 * 1024);
    }
}
