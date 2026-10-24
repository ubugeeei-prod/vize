//! Failure-only observations; errors and exhausted budgets stay explicit.
use serde_json::{Value, json};
use std::{
    fs,
    io::{self, Read},
    path::Path,
};

const FILE_LIMIT: usize = 64 * 1024;
const SNAPSHOT_LIMIT: usize = 4 * 1024 * 1024;

struct Budget {
    remaining: usize,
}

impl Budget {
    fn new(limit: usize) -> Self {
        Self {
            remaining: limit.min(SNAPSHOT_LIMIT),
        }
    }

    fn charge(&mut self, bytes: usize) -> usize {
        let retained = bytes.min(self.remaining);
        self.remaining -= retained;
        retained
    }
}

fn exhausted() -> Value {
    json!({"complete":false,"reason":"observation byte budget exhausted"})
}

pub(crate) fn snapshot(pid: u32) -> Value {
    if cfg!(target_os = "linux") {
        from_proc(Path::new("/proc"), pid)
    } else {
        json!({"available":false,"reason":"Linux /proc observation is unavailable on this platform"})
    }
}

fn error(error: io::Error) -> Value {
    json!({"errorKind":format!("{:?}",error.kind()),"error":format!("{error}")})
}

pub(crate) fn read(path: &Path) -> Value {
    read_with_budget(path, &mut Budget::new(SNAPSHOT_LIMIT))
}

fn read_with_budget(path: &Path, budget: &mut Budget) -> Value {
    if budget.remaining == 0 {
        return exhausted();
    }
    let allowance = budget.remaining.min(FILE_LIMIT + 1);
    let mut bytes = Vec::new();
    let result =
        fs::File::open(path).and_then(|file| file.take(allowance as u64).read_to_end(&mut bytes));
    // Charge probe bytes and partial reads even if the read reports an error.
    budget.charge(bytes.len());
    match result {
        Ok(_) => {
            let complete = bytes.len() <= FILE_LIMIT && bytes.len() < allowance;
            bytes.truncate(FILE_LIMIT);
            let mut observed = json!({"complete":complete,"bytes":bytes});
            if budget.remaining == 0 {
                observed["reason"] = "observation byte budget exhausted".into();
            }
            observed
        }
        Err(err) => error(err),
    }
}

pub(crate) fn entries(directory: &Path, limit: usize, observe: impl Fn(&Path) -> Value) -> Value {
    entries_with_budget(
        directory,
        limit,
        &mut Budget::new(SNAPSHOT_LIMIT),
        |path, _| observe(path),
    )
}

fn entries_with_budget(
    directory: &Path,
    limit: usize,
    budget: &mut Budget,
    mut observe: impl FnMut(&Path, &mut Budget) -> Value,
) -> Value {
    if budget.remaining == 0 {
        return exhausted();
    }
    let listed = fs::read_dir(directory).and_then(|mut entries| {
        let mut paths = Vec::new();
        while paths.len() <= limit && budget.remaining != 0 {
            let Some(entry) = entries.next() else {
                break;
            };
            let entry = entry?;
            let name = entry.file_name();
            if budget.charge(name.as_encoded_bytes().len()) != name.as_encoded_bytes().len() {
                break;
            }
            paths.push(entry.path());
        }
        Ok(paths)
    });
    match listed {
        Ok(mut paths) => {
            paths.sort();
            let complete = paths.len() <= limit && budget.remaining != 0;
            paths.truncate(limit);
            let entries = paths
                .iter()
                .map(|path| {
                    json!({
                        "name":path.file_name().map(|name|name.to_string_lossy()),
                        "observation":observe(path, budget)
                    })
                })
                .collect::<Vec<_>>();
            let mut observed =
                json!({"complete":complete && budget.remaining != 0,"entries":entries});
            if budget.remaining == 0 {
                observed["reason"] = "observation byte budget exhausted".into();
            }
            observed
        }
        Err(err) => error(err),
    }
}

fn link_with_budget(path: &Path, budget: &mut Budget) -> Value {
    if budget.remaining == 0 {
        return exhausted();
    }
    // std::read_link can transiently allocate one complete link target. This is
    // a retained-byte budget, not a bound on that allocation or filesystem time.
    match fs::read_link(path) {
        Ok(target) => {
            let raw = target.as_os_str().as_encoded_bytes();
            let retained = budget.charge(raw.len());
            if retained != raw.len() {
                return json!({
                    "complete":false,"reason":"observation byte budget exhausted",
                    "originalBytes":raw.len(),"targetBytes":&raw[..retained]
                });
            }
            json!({"target":target.to_string_lossy(),"targetBytes":raw})
        }
        Err(err) => error(err),
    }
}

pub(crate) fn from_proc(proc_root: &Path, pid: u32) -> Value {
    from_proc_with_limit(proc_root, pid, SNAPSHOT_LIMIT)
}

// The finite fixture entry point can only lower the production budget.
pub(crate) fn from_proc_with_limit(proc_root: &Path, pid: u32, limit: usize) -> Value {
    let mut budget = Budget::new(limit);
    let initial = budget.remaining;
    let root = proc_root.join(format!("{pid}"));
    let stat = read_with_budget(&root.join("stat"), &mut budget);
    let status = read_with_budget(&root.join("status"), &mut budget);
    let wchan = read_with_budget(&root.join("wchan"), &mut budget);
    let tasks = entries_with_budget(&root.join("task"), 512, &mut budget, |path, budget| {
        json!({
            "comm":read_with_budget(&path.join("comm"),budget),
            "stat":read_with_budget(&path.join("stat"),budget),
            "status":read_with_budget(&path.join("status"),budget),
            "wchan":read_with_budget(&path.join("wchan"),budget),
            "syscall":read_with_budget(&path.join("syscall"),budget),
            "stack":read_with_budget(&path.join("stack"),budget),
            "children":read_with_budget(&path.join("children"),budget)
        })
    });
    let fds = entries_with_budget(&root.join("fd"), 256, &mut budget, link_with_budget);
    json!({
        "available":true,"pid":pid,"stat":stat,"status":status,"wchan":wchan,
        "tasks":tasks,"fds":fds,
        "rawByteBudget":{
            "limitBytes":initial,"chargedBytes":initial-budget.remaining,
            "exhausted":budget.remaining==0,
            "scope":"retained file/link/name bytes and file probes; excludes transient read_link allocation and JSON expansion"
        }
    })
}
