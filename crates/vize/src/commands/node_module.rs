use std::{
    ffi::OsStr,
    fs, io,
    io::Write,
    process::{Command, Output, Stdio},
    thread,
};

pub(super) enum NodeModuleError {
    Prepare(io::Error),
    Start(io::Error),
    Wait(io::Error),
}

/// The CLI ships as one binary. Keep its private module alive until Node exits.
pub(super) fn run(
    node: &OsStr,
    options: &[&str],
    filename: &str,
    source: &str,
    input: &[u8],
) -> Result<(Output, io::Result<()>), NodeModuleError> {
    let directory = tempfile::Builder::new()
        .prefix("vize-node-")
        .tempdir()
        .map_err(NodeModuleError::Prepare)?;
    let module = directory.path().join(filename);
    fs::write(&module, source).map_err(NodeModuleError::Prepare)?;

    let mut child = Command::new(node)
        .args(options)
        .arg(module)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(NodeModuleError::Start)?;
    let pipe = child.stdin.take();
    thread::scope(|scope| {
        let writer = thread::Builder::new().spawn_scoped(scope, move || {
            let mut stdin =
                pipe.ok_or_else(|| io::Error::other("Node stdin pipe is unavailable"))?;
            stdin.write_all(input)
        });
        let output = child.wait_with_output().map_err(NodeModuleError::Wait);
        let written = writer
            .and_then(|handle| {
                handle
                    .join()
                    .map_err(|_| io::Error::other("Node stdin writer panicked"))
            })
            .flatten();
        Ok((output?, written))
    })
}
