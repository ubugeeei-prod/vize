//! Failure-only Linux process observation. Errors and truncation stay explicit.
use serde_json::{Value, json};
use std::{
    fs,
    io::{self, Read},
    path::Path,
};

const FILE_LIMIT: usize = 64 * 1024;

pub(super) fn snapshot(pid: u32) -> Value {
    if cfg!(target_os = "linux") {
        from_proc(Path::new("/proc"), pid)
    } else {
        json!({"available":false,"reason":"Linux /proc observation is unavailable on this platform"})
    }
}

fn error(error: io::Error) -> Value {
    json!({"errorKind":format!("{:?}",error.kind()),"error":error.to_string()})
}

pub(super) fn read(path: &Path) -> Value {
    let result = (|| -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        fs::File::open(path)?
            .take((FILE_LIMIT + 1) as u64)
            .read_to_end(&mut bytes)?;
        Ok(bytes)
    })();
    match result {
        Ok(mut bytes) => {
            let complete = bytes.len() <= FILE_LIMIT;
            bytes.truncate(FILE_LIMIT);
            json!({"complete":complete,"bytes":bytes})
        }
        Err(err) => error(err),
    }
}

pub(super) fn entries(directory: &Path, limit: usize, observe: impl Fn(&Path) -> Value) -> Value {
    let listed = fs::read_dir(directory).and_then(|entries| {
        entries
            .take(limit + 1)
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<io::Result<Vec<_>>>()
    });
    match listed {
        Ok(mut paths) => {
            paths.sort();
            let complete = paths.len() <= limit;
            paths.truncate(limit);
            json!({"complete":complete,"entries":paths.iter().map(|path|json!({
                "name":path.file_name().map(|name|name.to_string_lossy()),"observation":observe(path)
            })).collect::<Vec<_>>()})
        }
        Err(err) => error(err),
    }
}

pub(super) fn from_proc(proc_root: &Path, pid: u32) -> Value {
    let root = proc_root.join(pid.to_string());
    json!({
        "available":true,"pid":pid,"stat":read(&root.join("stat")),
        "status":read(&root.join("status")),"wchan":read(&root.join("wchan")),
        "tasks":entries(&root.join("task"),512,|path|json!({
            "comm":read(&path.join("comm")),"stat":read(&path.join("stat")),
            "status":read(&path.join("status")),"wchan":read(&path.join("wchan")),
            "syscall":read(&path.join("syscall")),"stack":read(&path.join("stack")),
            "children":read(&path.join("children"))
        })),
        "fds":entries(&root.join("fd"),256,|path| match fs::read_link(path) {
            Ok(target)=>json!({
                "target":target.to_string_lossy(),
                "targetBytes":target.as_os_str().as_encoded_bytes()
            }),Err(err)=>error(err)
        })
    })
}
