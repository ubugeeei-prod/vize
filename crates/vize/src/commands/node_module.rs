use std::{
    ffi::OsStr,
    io,
    io::Write,
    process::{Command, Output, Stdio},
    thread,
};

pub(super) enum NodeModuleError {
    Start(io::Error),
    Wait(io::Error),
}

/// The CLI ships as one binary, so its Node modules are embedded sources.
/// Node evaluates one as the ES module entry straight from argv; stdin stays
/// free for `input`, and nothing is written to disk.
pub(super) fn run(
    node: &OsStr,
    options: &[&str],
    source: &str,
    input: &[u8],
) -> Result<(Output, io::Result<()>), NodeModuleError> {
    let mut child = Command::new(node)
        .args(options)
        .args(["--input-type=module", "--eval", source])
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
