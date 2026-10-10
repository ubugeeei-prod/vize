//! Actual Linux exec and inherited-descriptor custody for test fixtures.
#![expect(
    clippy::disallowed_types,
    reason = "owned test threads share release state and std::io requires std String"
)]
use super::publish_executable;
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvTimeoutError, TryRecvError},
    },
    time::{Duration, Instant},
};

const SCRIPT: &[u8] = b"#!/bin/sh\nexit 0\n";

struct RetainedWriter(Child);
impl RetainedWriter {
    fn start(path: &Path, writer: &File) -> Self {
        let mut child = Command::new("/bin/sh")
            .args(["-c", "printf 'holder-ready\\n'; IFS= read -r release"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::from(writer.try_clone().unwrap()))
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (ack_tx, ack_rx) = mpsc::channel();
        let ack = std::thread::spawn(move || {
            let mut line = String::new();
            BufReader::new(stdout).read_line(&mut line).unwrap();
            ack_tx.send(line).unwrap();
        });
        let holder = Self(child);
        assert_eq!(
            ack_rx.recv_timeout(Duration::from_secs(10)).unwrap(),
            "holder-ready\n"
        );
        ack.join().unwrap();
        let fixture = std::fs::metadata(path).unwrap();
        let descriptor = std::fs::metadata(format!("/proc/{}/fd/2", holder.0.id())).unwrap();
        assert_eq!(descriptor.dev(), fixture.dev());
        assert_eq!(descriptor.ino(), fixture.ino());
        writeln!(
            std::io::stderr(),
            "owned retained writer pid={} fd=2 inode={}:{} ready",
            holder.0.id(),
            fixture.dev(),
            fixture.ino()
        )
        .unwrap();
        holder
    }

    fn release_and_reap(&mut self) {
        let mut stdin = self.0.stdin.take().unwrap();
        stdin.write_all(b"release\n").unwrap();
        drop(stdin);
        assert_eq!(self.0.wait().unwrap().code(), Some(0));
        assert!(self.0.try_wait().unwrap().is_some());
        writeln!(
            std::io::stderr(),
            "owned retained writer pid={} released and reaped",
            self.0.id()
        )
        .unwrap();
    }
}
impl Drop for RetainedWriter {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

fn write_fixture(path: &Path) -> File {
    let mut writer = File::create(path).unwrap();
    writer.write_all(SCRIPT).unwrap();
    writer
        .set_permissions(std::fs::Permissions::from_mode(0o755))
        .unwrap();
    writer
}

fn assert_single_exec(path: &Path) {
    assert_eq!(std::fs::read(path).unwrap(), SCRIPT);
    assert_eq!(
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o755
    );
    assert_eq!(Command::new(path).status().unwrap().code(), Some(0));
}

#[test]
fn local_writer_close_does_not_release_an_owned_child_writer() {
    let root = tempfile::TempDir::new().unwrap();
    let wrapper = root.path().join("tsc");
    let writer = write_fixture(&wrapper);
    writer.lock().unwrap();
    let mut holder = RetainedWriter::start(&wrapper, &writer);
    drop(writer);
    let error = Command::new(&wrapper).status().unwrap_err();
    assert_eq!(error.raw_os_error(), Some(libc::ETXTBSY));
    writeln!(
        std::io::stderr(),
        "original close-only exec failed with exact os{}",
        libc::ETXTBSY
    )
    .unwrap();
    holder.release_and_reap();
    File::open(&wrapper).unwrap().lock_shared().unwrap();
    assert_single_exec(&wrapper);
}

#[test]
fn publication_waits_for_the_last_owned_writer_then_executes_once() {
    let root = tempfile::TempDir::new().unwrap();
    let wrapper = root.path().join("tsc");
    let writer = write_fixture(&wrapper);
    let mut holder = RetainedWriter::start(&wrapper, &writer);
    let probe = File::open(&wrapper).unwrap();
    let release_allowed = Arc::new(AtomicBool::new(false));
    let released = Arc::clone(&release_allowed);
    let published_path = wrapper.clone();
    let (ready_tx, ready_rx) = mpsc::channel();
    let publisher = std::thread::spawn(move || {
        let result = publish_executable(&published_path, writer);
        let after_release = released.load(Ordering::SeqCst);
        ready_tx.send((result, after_release)).unwrap();
    });
    let started = Instant::now();
    loop {
        match probe.try_lock_shared() {
            Err(std::fs::TryLockError::WouldBlock) => break,
            Err(std::fs::TryLockError::Error(error)) => panic!("lock probe failed: {error}"),
            Ok(()) => probe.unlock().unwrap(),
        }
        assert!(
            matches!(ready_rx.try_recv(), Err(TryRecvError::Empty)),
            "fixture became ready while its owned child retained the writer"
        );
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "publisher never locked its writer"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(
        matches!(
            ready_rx.recv_timeout(Duration::from_millis(50)),
            Err(RecvTimeoutError::Timeout)
        ),
        "publication returned before the inherited writer was released"
    );
    writeln!(
        std::io::stderr(),
        "publication pending with exact owned child writer still retained"
    )
    .unwrap();
    release_allowed.store(true, Ordering::SeqCst);
    holder.release_and_reap();
    let (result, after_release) = ready_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    result.unwrap();
    assert!(
        after_release,
        "publisher certified readiness before release was allowed"
    );
    publisher.join().unwrap();
    assert_single_exec(&wrapper);
    writeln!(
        std::io::stderr(),
        "guard ready after writer release; single exec succeeded"
    )
    .unwrap();
}
