//! Opt-in observations around the original collector and once-only source reader.

use std::{any::Any, env, fs, io, panic, path::Path, path::PathBuf};

use davinci_test_support::corpus::{CorpusSweep, read_corpus_source};

const TRACE_ENV: &str = "VIZE_DAVINCI_SSR_SOURCE_IO_TRACE";

fn directory() -> Option<PathBuf> {
    env::var_os(TRACE_ENV).map(PathBuf::from)
}

#[cfg(unix)]
fn raw(path: &Path) -> &[u8] {
    use std::os::unix::ffi::OsStrExt;
    path.as_os_str().as_bytes()
}

#[cfg(unix)]
fn vector<'a>(paths: impl Iterator<Item = &'a Path>) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    for path in paths {
        if raw(path).contains(&0) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "NUL in path"));
        }
        bytes.extend_from_slice(raw(path));
        bytes.push(0);
    }
    Ok(bytes)
}

#[cfg(unix)]
fn save_sweep(directory: &Path, sweep: &CorpusSweep) -> io::Result<()> {
    if !directory.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "trace directory must be absolute",
        ));
    }
    fs::create_dir_all(directory)?;
    let cwd = env::current_dir()?;
    let absolute: Vec<_> = sweep.files.iter().map(|path| cwd.join(path)).collect();
    let relative: Vec<_> = sweep
        .files
        .iter()
        .map(|path| {
            path.strip_prefix(&sweep.root)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
        })
        .collect::<io::Result<_>>()?;
    for (name, bytes) in [
        ("root.nul", vector(std::iter::once(sweep.root.as_path()))?),
        (
            "selected-vector.nul",
            vector(sweep.files.iter().map(PathBuf::as_path))?,
        ),
        (
            "absolute-vector.nul",
            vector(absolute.iter().map(PathBuf::as_path))?,
        ),
        ("relative-vector.nul", vector(relative.into_iter())?),
    ] {
        fs::write(directory.join(name), bytes)?;
    }
    fs::write(
        directory.join("selected-count.txt"),
        sweep.files.len().to_string(),
    )?;
    fs::write(directory.join("collector-cwd.bytes"), raw(&cwd))?;
    fs::write(
        directory.join("cargo-manifest-dir.bytes"),
        env!("CARGO_MANIFEST_DIR").as_bytes(),
    )
}

pub(super) fn archive_sweep(sweep: &CorpusSweep) {
    let Some(directory) = directory() else {
        return;
    };
    #[cfg(unix)]
    if let Err(error) = save_sweep(&directory, sweep) {
        eprintln!("SSR source IO trace archive incomplete: {error:?}");
    }
    #[cfg(not(unix))]
    panic!("SSR source IO trace requires raw Unix OsStr paths: {directory:?}, {sweep:?}");
}

#[cfg(unix)]
fn save_failure(
    directory: &Path,
    ordinal: usize,
    path: &Path,
    payload: &(dyn Any + Send),
) -> io::Result<()> {
    let pid = std::process::id();
    let directory = directory.join(format!("failure-{ordinal}-{pid}"));
    fs::create_dir_all(&directory)?;
    fs::write(directory.join("path.bytes"), raw(path))?;
    fs::write(directory.join("ordinal.txt"), ordinal.to_string())?;
    fs::write(directory.join("pid.txt"), pid.to_string())?;
    let thread = std::thread::current();
    fs::write(
        directory.join("thread-name.bytes"),
        thread.name().unwrap_or("").as_bytes(),
    )?;
    fs::write(
        directory.join("rust-thread-id.txt"),
        format!("{:?}", thread.id()),
    )?;
    if let Some(value) = payload.downcast_ref::<String>() {
        fs::write(directory.join("panic-payload.bytes"), value.as_bytes())?;
        fs::write(directory.join("panic-payload-kind.txt"), b"String")?;
    } else if let Some(value) = payload.downcast_ref::<&str>() {
        fs::write(directory.join("panic-payload.bytes"), value.as_bytes())?;
        fs::write(directory.join("panic-payload-kind.txt"), b"&str")?;
    } else {
        fs::write(
            directory.join("panic-payload-kind.txt"),
            format!("opaque {:?}; original box resumed", payload.type_id()),
        )?;
    }
    fs::write(directory.join("cwd.bytes"), raw(&env::current_dir()?))?;
    fs::write(directory.join("exe.bytes"), raw(&env::current_exe()?))?;
    #[cfg(target_os = "linux")]
    {
        let link = fs::read_link("/proc/thread-self")?;
        fs::write(directory.join("thread-self.bytes"), raw(&link))?;
        let tid = link
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "thread-self has no TID"))?;
        use std::os::unix::ffi::OsStrExt;
        fs::write(directory.join("tid.bytes"), tid.as_bytes())?;
    }
    Ok(())
}

pub(super) fn read(ordinal: usize, path: &Path) -> vize_l0::CompactString {
    let Some(directory) = directory() else {
        return read_corpus_source(path);
    };
    #[cfg(not(unix))]
    panic!("SSR source IO trace requires raw Unix OsStr paths: {directory:?}, {ordinal}, {path:?}");
    #[cfg(unix)]
    match panic::catch_unwind(panic::AssertUnwindSafe(|| read_corpus_source(path))) {
        Ok(source) => source,
        Err(payload) => {
            if let Err(error) = save_failure(&directory, ordinal, path, payload.as_ref()) {
                eprintln!("SSR source IO trace failure custody incomplete: {error:?}");
            }
            panic::resume_unwind(payload)
        }
    }
}
