// Appended to the exact original collector harness; open64 is Linux-only.

use std::io::Read;
use std::os::unix::{ffi::OsStrExt, fs::MetadataExt};

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn open64(path: *const std::ffi::c_char, flags: std::ffi::c_int, ...) -> std::ffi::c_int;
    fn close(fd: std::ffi::c_int) -> std::ffi::c_int;
}

fn frame(
    out: &mut impl Write,
    index: usize,
    operation: &str,
    status: &str,
    detail: &str,
    data: &[u8],
) -> io::Result<()> {
    let header = format!("{index}\t{operation}\t{status}\t{}", detail.escape_debug());
    out.write_all(&(header.len() as u64).to_le_bytes())?;
    out.write_all(header.as_bytes())?;
    out.write_all(&(data.len() as u64).to_le_bytes())?;
    out.write_all(data)
}

fn error_frame(
    out: &mut impl Write,
    index: usize,
    operation: &str,
    phase: &str,
    path: &Path,
    error: &io::Error,
    data: &[u8],
) -> io::Result<()> {
    let detail = format!(
        "phase={phase};display={error};debug={error:?};kind={:?};raw_os_error={:?}",
        error.kind(),
        error.raw_os_error()
    );
    eprintln!("probe index={index} operation={operation} path={path:?}: {detail}");
    frame(out, index, operation, "error", &detail, data)
}

fn probe() -> io::Result<bool> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 5 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected ROOT VECTOR COLLECTED FRAMES",
        ));
    }
    let root = Path::new(&args[1]);
    let mut collected = Vec::new();
    let mut failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        collect_vue_files(root, &mut collected);
    }))
    .is_err();
    eprintln!(
        "probe collector_ok={} collected={}",
        !failed,
        collected.len()
    );
    let mut vector_out = io::BufWriter::new(fs::File::create(&args[3])?);
    for path in collected {
        let logical = path.strip_prefix(root).map_err(io::Error::other)?;
        vector_out.write_all(logical.as_os_str().as_bytes())?;
        vector_out.write_all(&[0])?;
    }
    vector_out.flush()?;
    let vector = fs::read(&args[2])?;
    if vector.is_empty() || vector.last() != Some(&0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "incomplete fixed NUL vector",
        ));
    }
    let mut out = io::BufWriter::new(fs::File::create(&args[4])?);
    for (index, logical) in vector[..vector.len() - 1]
        .split(|byte| *byte == 0)
        .enumerate()
    {
        let logical = std::str::from_utf8(logical)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if logical.is_empty()
            || logical.starts_with('/')
            || logical.split('/').any(|p| ["", ".", ".."].contains(&p))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "foreign fixed logical path",
            ));
        }
        let mut literal = root.as_os_str().to_os_string();
        literal.push("/");
        literal.push(logical);
        let path = PathBuf::from(literal);
        macro_rules! bad {
            ($op:literal, $phase:literal, $error:expr, $data:expr) => {{
                failed = true;
                error_frame(&mut out, index, $op, $phase, &path, &$error, $data)?;
            }};
        }
        match fs::metadata(&path) {
            Ok(meta) => frame(
                &mut out,
                index,
                "stat",
                "ok",
                &format!(
                    "type={:?};len={};ino={};dev={};mode={:o};nlink={}",
                    meta.file_type(),
                    meta.len(),
                    meta.ino(),
                    meta.dev(),
                    meta.mode(),
                    meta.nlink()
                ),
                &[],
            )?,
            Err(error) => bad!("stat", "metadata", error, &[]),
        }
        #[cfg(target_os = "linux")]
        match std::ffi::CString::new(path.as_os_str().as_bytes()) {
            Ok(c_path) => {
                // Linux O_RDONLY | O_CLOEXEC; no creation, retries or path rewriting.
                let fd = unsafe { open64(c_path.as_ptr(), 0x80000) };
                if fd < 0 {
                    bad!("open64", "open", io::Error::last_os_error(), &[]);
                } else if unsafe { close(fd) } < 0 {
                    bad!(
                        "open64",
                        "close_after_successful_open",
                        io::Error::last_os_error(),
                        &[]
                    );
                } else {
                    frame(
                        &mut out,
                        index,
                        "open64",
                        "ok",
                        &format!("fd={fd};close=ok"),
                        &[],
                    )?;
                }
            }
            Err(error) => bad!(
                "open64",
                "path_cstring",
                io::Error::new(io::ErrorKind::InvalidInput, error),
                &[]
            ),
        }
        #[cfg(not(target_os = "linux"))]
        frame(
            &mut out,
            index,
            "open64",
            "unsupported",
            "Linux-only direct libc open64; no fallback operation",
            &[],
        )?;
        match fs::File::open(&path) {
            Ok(mut file) => {
                let mut bytes = Vec::new();
                match file.read_to_end(&mut bytes) {
                    Ok(_) => frame(
                        &mut out,
                        index,
                        "file_open",
                        "ok",
                        "open=ok;read_to_end=ok",
                        &bytes,
                    )?,
                    Err(error) => bad!("file_open", "read_after_successful_open", error, &bytes),
                }
            }
            Err(error) => bad!("file_open", "open", error, &[]),
        }
        match fs::read_to_string(&path) {
            Ok(text) => frame(
                &mut out,
                index,
                "read_to_string",
                "ok",
                "utf8=ok",
                text.as_bytes(),
            )?,
            Err(error) => bad!("read_to_string", "read_to_string", error, &[]),
        }
    }
    out.flush()?;
    Ok(failed)
}

fn main() {
    match probe() {
        Ok(false) => {}
        Ok(true) => std::process::exit(1),
        Err(error) => {
            eprintln!(
                "probe infrastructure failure: {error} (debug={error:?}, kind={:?}, raw_os_error={:?})",
                error.kind(),
                error.raw_os_error()
            );
            std::process::exit(1);
        }
    }
}
