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
