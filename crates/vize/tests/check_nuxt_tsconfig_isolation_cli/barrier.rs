use std::{
    ffi::CString,
    fs,
    io::{Read, Write},
    os::unix::{ffi::OsStrExt, io::FromRawFd},
    path::Path,
    process::{Child, Output},
    time::{Duration, Instant},
};

use super::nuxt_fifo::create_fifo;

pub(super) fn create_phase_barrier(path: &Path) {
    fs::create_dir_all(path).unwrap();
    create_fifo(&path.join("ready"));
    create_fifo(&path.join("release-alpha"));
    create_fifo(&path.join("release-bravo"));
}

pub(super) fn wait_until_both_configs_are_prepared(
    barrier: &Path,
    alpha: &mut Child,
    bravo: &mut Child,
) -> Result<(), String> {
    let path = CString::new(barrier.join("ready").as_os_str().as_bytes()).unwrap();
    // SAFETY: `path` is a NUL-terminated FIFO path. The returned descriptor
    // is owned immediately by `File`, and the flags are valid on Unix.
    let descriptor = unsafe { libc::open(path.as_ptr(), libc::O_RDWR | libc::O_NONBLOCK) };
    if descriptor < 0 {
        return Err(format!(
            "failed to open the Nuxt config FIFO: {}",
            std::io::Error::last_os_error()
        ));
    }
    // SAFETY: `descriptor` is a fresh owned descriptor from `libc::open`.
    let mut ready_file = unsafe { fs::File::from_raw_fd(descriptor) };
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut ready = [0_u8; 2];
    let mut received = 0;
    loop {
        match ready_file.read(&mut ready[received..]) {
            Ok(0) => {}
            Ok(count) => {
                received += count;
                if received == ready.len() {
                    return (ready == *b"xx")
                        .then_some(())
                        .ok_or_else(|| format!("invalid Nuxt config tokens: {ready:?}"));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(error) => return Err(format!("failed to read Nuxt config FIFO: {error}")),
        }
        for (name, child) in [("alpha", &mut *alpha), ("bravo", &mut *bravo)] {
            if let Some(status) = child
                .try_wait()
                .map_err(|error| format!("failed to inspect {name}: {error}"))?
            {
                return Err(format!(
                    "{name} exited before both reached the barrier: {status}"
                ));
            }
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "timed out after 30s waiting for Nuxt configs ({received}/2 ready)"
            ));
        }
        std::thread::park_timeout(Duration::from_millis(10));
    }
}

pub(super) fn release(barrier: &Path, participant: &str) {
    let release = barrier.join(format!("release-{participant}"));
    let encoded = CString::new(release.as_os_str().as_bytes()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        // SAFETY: `encoded` is a NUL-terminated FIFO path and the returned
        // descriptor is immediately owned by `File` when successful.
        let descriptor = unsafe {
            libc::open(
                encoded.as_ptr(),
                libc::O_WRONLY | libc::O_NONBLOCK | libc::O_CLOEXEC,
            )
        };
        if descriptor >= 0 {
            // SAFETY: `descriptor` is a fresh owned descriptor from `libc::open`.
            let mut writer = unsafe { fs::File::from_raw_fd(descriptor) };
            writer.write_all(b"go\n").unwrap();
            return;
        }
        let error = std::io::Error::last_os_error();
        assert_eq!(
            error.raw_os_error(),
            Some(libc::ENXIO),
            "failed to open release FIFO {}: {error}",
            release.display()
        );
        assert!(
            Instant::now() < deadline,
            "timed out after 30s waiting for release FIFO reader: {}",
            release.display()
        );
        std::thread::park_timeout(Duration::from_millis(10));
    }
}

pub(super) fn describe_output(name: &str, output: &Output) -> String {
    format!(
        "{name} status={}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}
