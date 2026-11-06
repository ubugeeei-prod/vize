use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use tempfile::NamedTempFile;

use super::{AtomicWriteOperations, OsOperations, atomic_write, atomic_write_with};

const ORIGINAL: &[u8] = b"original source\n";
const SIBLING: &[u8] = b"sibling replacement\n";

struct Hold {
    stage: &'static str,
    entered: Sender<&'static str>,
    release: Receiver<()>,
}

impl AtomicWriteOperations for Hold {
    fn write_all(&mut self, file: &mut File, contents: &[u8]) -> io::Result<()> {
        if self.stage == "write" {
            self.entered.send("write").unwrap();
            self.release.recv().unwrap();
        }
        OsOperations.write_all(file, contents)
    }

    fn flush(&mut self, file: &mut File) -> io::Result<()> {
        OsOperations.flush(file)
    }

    fn replace(&mut self, temporary: NamedTempFile, path: &Path) -> io::Result<()> {
        if self.stage == "replace" {
            self.entered.send("replace").unwrap();
            self.release.recv().unwrap();
        }
        OsOperations.replace(temporary, path)
    }
}

struct NotifyWrite {
    entered: Sender<()>,
}

impl AtomicWriteOperations for NotifyWrite {
    fn write_all(&mut self, file: &mut File, contents: &[u8]) -> io::Result<()> {
        self.entered.send(()).unwrap();
        OsOperations.write_all(file, contents)
    }

    fn flush(&mut self, file: &mut File) -> io::Result<()> {
        OsOperations.flush(file)
    }

    fn replace(&mut self, temporary: NamedTempFile, path: &Path) -> io::Result<()> {
        OsOperations.replace(temporary, path)
    }
}

fn prepared(directory: &Path, name: &str) -> PathBuf {
    let path = directory.join(name);
    fs::write(&path, ORIGINAL).unwrap();
    path
}

fn assert_no_temporary_files(directory: &Path) {
    let temporary_count = fs::read_dir(directory)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".vize-write-")
        })
        .count();
    assert_eq!(temporary_count, 0);
}

fn finish_within(receiver: &Receiver<io::Result<()>>, what: &str) -> io::Result<()> {
    match receiver.recv_timeout(Duration::from_secs(5)) {
        Ok(result) => result,
        Err(error) => panic!("{what} did not finish: {error}"),
    }
}

#[test]
fn concurrent_replacements_keep_each_files_bytes() {
    let project = tempfile::tempdir().unwrap();
    let names = [
        "F0.vue", "F1.vue", "F2.vue", "F3.vue", "F4.vue", "F5.vue", "F6.vue", "F7.vue",
    ];
    let mut handles = Vec::new();
    for (index, name) in names.iter().enumerate() {
        let path = prepared(project.path(), name);
        handles.push(thread::spawn(move || {
            let mut contents = *b"replacement 0\n";
            contents[12] = b'0' + u8::try_from(index).unwrap();
            atomic_write(&path, &contents).unwrap();
            (path, contents)
        }));
    }
    for handle in handles {
        let (path, contents) = handle.join().unwrap();
        assert_eq!(fs::read(&path).unwrap(), contents);
    }
    assert_no_temporary_files(project.path());
}

#[test]
fn payload_writes_in_one_directory_are_not_serialized() {
    let project = tempfile::tempdir().unwrap();
    let held = prepared(project.path(), "Held.vue");
    let other = prepared(project.path(), "Other.vue");
    let held_task = held.clone();
    let other_task = other.clone();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    thread::spawn(move || {
        let result = atomic_write_with(
            &held_task,
            SIBLING,
            &mut Hold {
                stage: "write",
                entered: entered_tx,
                release: release_rx,
            },
        );
        let _ = done_tx.send(result);
    });
    assert_eq!(
        entered_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
        "write"
    );

    let (other_tx, other_rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = other_tx.send(atomic_write(&other_task, b"other file\n"));
    });
    finish_within(&other_rx, "same-directory payload write").unwrap();
    assert_eq!(fs::read(&other).unwrap(), b"other file\n");

    release_tx.send(()).unwrap();
    finish_within(&done_rx, "held payload write").unwrap();
    assert_eq!(fs::read(&held).unwrap(), SIBLING);
    assert_no_temporary_files(project.path());
}

#[test]
fn replace_waits_for_the_in_flight_sibling_in_the_same_directory() {
    let project = tempfile::tempdir().unwrap();
    let held = prepared(project.path(), "Held.vue");
    let other = prepared(project.path(), "Other.vue");
    let held_task = held.clone();
    let other_task = other.clone();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    thread::spawn(move || {
        let result = atomic_write_with(
            &held_task,
            SIBLING,
            &mut Hold {
                stage: "replace",
                entered: entered_tx,
                release: release_rx,
            },
        );
        let _ = done_tx.send(result);
    });
    assert_eq!(
        entered_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
        "replace"
    );

    let (write_tx, write_rx) = mpsc::channel();
    let (other_tx, other_rx) = mpsc::channel();
    thread::spawn(move || {
        let result = atomic_write_with(
            &other_task,
            b"other file\n",
            &mut NotifyWrite { entered: write_tx },
        );
        let _ = other_tx.send(result);
    });
    match write_rx.recv_timeout(Duration::from_secs(1)) {
        Ok(()) => panic!("same-directory create did not wait for the in-flight rename"),
        Err(mpsc::RecvTimeoutError::Timeout) => {}
        Err(error) => panic!("same-directory writer disconnected: {error}"),
    }

    release_tx.send(()).unwrap();
    finish_within(&done_rx, "held rename").unwrap();
    finish_within(&other_rx, "queued sibling write").unwrap();
    assert_eq!(fs::read(&held).unwrap(), SIBLING);
    assert_eq!(fs::read(&other).unwrap(), b"other file\n");
    assert_no_temporary_files(project.path());
}

#[test]
fn a_blocked_replace_does_not_stall_another_directory() {
    let project = tempfile::tempdir().unwrap();
    let first_dir = project.path().join("first");
    let second_dir = project.path().join("second");
    fs::create_dir(&first_dir).unwrap();
    fs::create_dir(&second_dir).unwrap();
    let first = prepared(&first_dir, "App.vue");
    let second = prepared(&second_dir, "App.vue");
    let first_task = first.clone();
    let second_task = second.clone();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    thread::spawn(move || {
        let result = atomic_write_with(
            &first_task,
            SIBLING,
            &mut Hold {
                stage: "replace",
                entered: entered_tx,
                release: release_rx,
            },
        );
        let _ = done_tx.send(result);
    });
    assert_eq!(
        entered_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
        "replace"
    );

    let (other_tx, other_rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = other_tx.send(atomic_write(&second_task, b"other directory\n"));
    });
    finish_within(&other_rx, "other directory").unwrap();
    assert_eq!(fs::read(&second).unwrap(), b"other directory\n");

    release_tx.send(()).unwrap();
    finish_within(&done_rx, "blocked directory").unwrap();
    assert_eq!(fs::read(&first).unwrap(), SIBLING);
    assert_no_temporary_files(&first_dir);
    assert_no_temporary_files(&second_dir);
}
