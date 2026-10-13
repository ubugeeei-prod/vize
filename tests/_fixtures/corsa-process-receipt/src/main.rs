use corsa_core::{AsyncChildGuard, CorsaCommand, ProcessObserver};
use corsa_runtime::block_on;
use std::{fs, time::Duration};

fn main() {
    fs::create_dir_all("receipts").unwrap();
    for (name, script, timeout) in [
        (
            "raw-non-utf8-exit17",
            "printf 'before\\000\\377after\\n' >&2; exit 17",
            1000,
        ),
        (
            "wrong-stderr-exit0",
            "printf 'unexpected native error\\n' >&2; exit 0",
            1000,
        ),
        (
            "mixed-expected-wrong",
            "printf \"error handling method 'exit': EOF\\nunexpected native error\\n\" >&2; exit 0",
            1000,
        ),
        (
            "timeout-kill-reap",
            "printf 'before timeout\\n' >&2; exec sleep 30",
            20,
        ),
    ] {
        let observer = ProcessObserver::default();
        let command = CorsaCommand::new("/bin/sh").with_process_observer(observer.clone());
        let (mut child, observation) = command.spawn_observed_async(["-c", script]).unwrap();
        let pid = child.id();
        drop(child.stdin.take());
        drop(child.stdout.take());
        let guard = AsyncChildGuard::new_observed(child, observation);
        block_on(guard.shutdown(Duration::from_millis(timeout))).unwrap();
        block_on(guard.shutdown(Duration::from_millis(timeout))).unwrap();
        let receipts = observer.receipts();
        assert_eq!(receipts.len(), 1);
        let receipt = &receipts[0];
        assert_eq!(receipt.pid, pid);
        assert!(receipt.completed);
        assert!(receipt.stderr_complete);
        assert_eq!(receipt.stderr_error, None);
        assert_eq!(receipt.process_error, None);
        assert!(receipt.reaped);
        let status = receipt.exit_status.unwrap();
        match name {
            "raw-non-utf8-exit17" => {
                assert_eq!(receipt.stderr, b"before\0\xffafter\n");
                assert_eq!(status.code(), Some(17));
                assert_eq!((receipt.timed_out, receipt.kill_requested), (false, false));
            }
            "wrong-stderr-exit0" => {
                assert_eq!(receipt.stderr, b"unexpected native error\n");
                assert_eq!(status.code(), Some(0));
            }
            "mixed-expected-wrong" => {
                assert_eq!(
                    receipt.stderr,
                    b"error handling method 'exit': EOF\nunexpected native error\n"
                );
                assert_eq!(status.code(), Some(0));
            }
            "timeout-kill-reap" => {
                assert_eq!(receipt.stderr, b"before timeout\n");
                assert!(!status.success());
                assert_eq!((receipt.timed_out, receipt.kill_requested), (true, true));
            }
            _ => unreachable!(),
        }
        fs::write(format!("receipts/{name}.stderr.bin"), &receipt.stderr).unwrap();
        println!("{name}: {receipt:?}");
    }
    drop_and_default_controls();
}

fn drop_and_default_controls() {
    use std::io::{BufRead, BufReader};
    let observer = ProcessObserver::default();
    let command = CorsaCommand::new("/bin/sh").with_process_observer(observer.clone());
    let (mut child, observation) = command
        .spawn_observed_async([
            "-c",
            "printf 'drop control\\n' >&2; printf 'ready\\n'; exec sleep 30",
        ])
        .unwrap();
    let mut ready = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut ready)
        .unwrap();
    assert_eq!(ready, "ready\n");
    drop(child.stdin.take());
    drop(AsyncChildGuard::new_observed(child, observation));
    let receipts = observer.receipts();
    assert_eq!(receipts.len(), 1);
    let receipt = &receipts[0];
    assert!(
        receipt.completed && receipt.stderr_complete && receipt.reaped && receipt.kill_requested
    );
    assert!(!receipt.timed_out);
    assert!(!receipt.exit_status.unwrap().success());
    assert_eq!(receipt.stderr, b"drop control\n");
    assert_eq!(receipt.stderr_error, None);
    assert_eq!(receipt.process_error, None);
    println!("guard-drop: {receipt:?}");
    let (mut child, observation) = CorsaCommand::new("/bin/sh")
        .spawn_observed_async(["-c", "exit 0"])
        .unwrap();
    assert!(observation.is_none());
    assert!(child.stderr.is_none());
    drop(child.stdin.take());
    drop(child.stdout.take());
    block_on(AsyncChildGuard::new_observed(child, observation).shutdown(Duration::from_secs(1)))
        .unwrap();
}
