use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs, io,
    path::Path,
    thread,
    time::{Duration, Instant},
};

fn proc_stat_bytes(path: &Path, result: io::Result<Vec<u8>>) -> Result<Option<Vec<u8>>, String> {
    match result {
        Ok(bytes) => Ok(Some(bytes)),
        // An opened proc file can outlive its task: Linux proc_single_show then
        // returns ESRCH. Only this per-PID stat read may treat it as disappeared.
        Err(error)
            if error.kind() == io::ErrorKind::NotFound
                || error.raw_os_error() == Some(libc::ESRCH) =>
        {
            Ok(None)
        }
        Err(error) => Err(format!("read process stat {}: {error}", path.display())),
    }
}

pub(super) fn session_members(session: u32) -> Result<Vec<(i32, i32, char)>, String> {
    let mut members = Vec::new();
    for entry in fs::read_dir("/proc").map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<i32>().ok())
        else {
            continue;
        };
        let path = entry.path().join("stat");
        let Some(stat) = proc_stat_bytes(&path, fs::read(&path))? else {
            continue;
        };
        let tail = stat
            .rsplit(|byte| *byte == b')')
            .next()
            .ok_or("malformed process stat")?;
        let fields = std::str::from_utf8(tail)
            .map_err(|e| e.to_string())?
            .split_whitespace()
            .collect::<Vec<_>>();
        if fields.get(3).and_then(|s| s.parse::<u32>().ok()) == Some(session) {
            members.push((
                pid,
                fields
                    .get(2)
                    .and_then(|s| s.parse().ok())
                    .ok_or("invalid process group")?,
                fields[0].chars().next().ok_or("missing process state")?,
            ));
        }
    }
    members.sort_unstable();
    Ok(members)
}
pub(super) fn cleanup_session(session: u32) -> Result<Value, String> {
    if session == 0 || session == std::process::id() || unsafe { libc::getsid(0) } == session as i32
    {
        return Err("refusing to signal the supervisor's session".into());
    }
    unsafe {
        libc::kill(session as i32, libc::SIGSTOP);
    }
    let started = Instant::now();
    let mut members = session_members(session)?;
    loop {
        let groups = members
            .iter()
            .filter(|m| !matches!(m.2, 'Z' | 'X'))
            .map(|m| m.1)
            .collect::<BTreeSet<_>>();
        for group in &groups {
            unsafe {
                libc::kill(-*group, libc::SIGSTOP);
            }
        }
        thread::sleep(Duration::from_millis(2));
        let next = session_members(session)?;
        if next == members && next.iter().all(|m| matches!(m.2, 'T' | 't' | 'Z' | 'X')) {
            members = next;
            break;
        }
        members = next;
        if started.elapsed() > Duration::from_secs(1) {
            for group in groups {
                unsafe {
                    libc::kill(-group, libc::SIGKILL);
                }
            }
            return Err("owned session did not quiesce before cleanup deadline".into());
        }
    }
    let groups = members
        .iter()
        .filter(|m| !matches!(m.2, 'Z' | 'X'))
        .map(|m| m.1)
        .collect::<BTreeSet<_>>();
    for group in groups.iter().filter(|g| **g != session as i32) {
        unsafe {
            libc::kill(-*group, libc::SIGKILL);
        }
    }
    if groups.contains(&(session as i32)) {
        unsafe {
            libc::kill(-(session as i32), libc::SIGKILL);
        }
    }
    while session_members(session)?
        .iter()
        .any(|m| !matches!(m.2, 'Z' | 'X'))
    {
        if started.elapsed() > Duration::from_secs(2) {
            return Err("owned session still has live writers after SIGKILL".into());
        }
        thread::sleep(Duration::from_millis(5));
    }
    Ok(
        json!({"members":members,"groups":groups,"allWritersStopped":true,"elapsedMs":started.elapsed().as_millis()}),
    )
}

#[cfg(test)]
mod tests {
    use super::{cleanup_session, proc_stat_bytes, session_members};
    use std::{
        fs,
        io::{self, Read},
        os::unix::process::CommandExt,
        path::Path,
        process::{Child, Command, Stdio},
    };

    struct Process(Child);
    impl Drop for Process {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    fn idle_process(own_session: bool) -> Process {
        let mut command = Command::new("node");
        command
            .args(["-e", "setInterval(()=>{},1000)"])
            .stdin(Stdio::null());
        if own_session {
            unsafe {
                command.pre_exec(|| {
                    if libc::setsid() == -1 {
                        Err(io::Error::last_os_error())
                    } else {
                        Ok(())
                    }
                });
            }
        }
        Process(command.spawn().unwrap())
    }

    #[test]
    fn proc_stat_disappearance_is_the_only_per_entry_io_exception() {
        let path = Path::new("/proc/123/stat");
        for code in [libc::ENOENT, libc::ESRCH] {
            assert!(
                proc_stat_bytes(path, Err(io::Error::from_raw_os_error(code)))
                    .unwrap()
                    .is_none()
            );
        }
        for code in [
            libc::EPERM,
            libc::EACCES,
            libc::EIO,
            libc::EINVAL,
            libc::EINTR,
        ] {
            let error = io::Error::from_raw_os_error(code);
            let expected = format!("read process stat /proc/123/stat: {error}");
            assert_eq!(proc_stat_bytes(path, Err(error)).unwrap_err(), expected);
        }
        let bytes = b"123 (whole authored stat) R 1 123 123".to_vec();
        assert_eq!(
            proc_stat_bytes(path, Ok(bytes.clone())).unwrap(),
            Some(bytes)
        );
    }

    #[test]
    fn reaped_proc_fd_does_not_hide_live_owned_writers_or_signal_a_sentinel() {
        let mut sentinel = idle_process(false);
        let mut writer = idle_process(true);
        let mut disappearing = idle_process(false);
        let stat_path = std::path::PathBuf::from(format!("/proc/{}/stat", disappearing.0.id()));
        let mut stat = fs::File::open(&stat_path).unwrap();
        disappearing.0.kill().unwrap();
        disappearing.0.wait().unwrap();
        let error = stat.read_to_end(&mut Vec::new()).unwrap_err();
        assert_eq!(error.raw_os_error(), Some(libc::ESRCH));
        assert!(proc_stat_bytes(&stat_path, Err(error)).unwrap().is_none());

        let session = writer.0.id();
        let members = session_members(session).unwrap();
        assert!(
            members
                .iter()
                .any(|m| m.0 == session as i32 && !matches!(m.2, 'Z' | 'X'))
        );
        assert!(sentinel.0.try_wait().unwrap().is_none());
        let receipt = cleanup_session(session).unwrap();
        assert_eq!(receipt["allWritersStopped"], true);
        assert!(
            receipt["members"]
                .as_array()
                .unwrap()
                .iter()
                .any(|m| m[0] == session)
        );
        writer.0.wait().unwrap();
        assert!(session_members(session).unwrap().is_empty());
        assert!(sentinel.0.try_wait().unwrap().is_none());
    }
}
