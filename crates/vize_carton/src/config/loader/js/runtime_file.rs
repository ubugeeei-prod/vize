//! A private, scoped ES module keeps evaluator code out of process arguments.

use std::{
    io::{self, Write},
    path::Path,
    process::Command,
};

/// Preserve the logical argv contract previously supplied by Node eval mode.
const ARGV_COMPATIBILITY: &[u8] = b"process.argv.splice(1, 1);\n";

pub(super) struct NodeRuntimeFile {
    path: tempfile::TempPath,
}

impl NodeRuntimeFile {
    pub(super) fn new(script: &str) -> io::Result<Self> {
        let mut file = tempfile::Builder::new()
            .prefix("vize-config-runtime-")
            .suffix(".mjs")
            .tempfile()?;
        file.write_all(ARGV_COMPATIBILITY)?;
        file.write_all(script.as_bytes())?;
        // Close the writer before Node opens it, including on Windows. The
        // TempPath still removes the module on every success/error exit.
        Ok(Self {
            path: file.into_temp_path(),
        })
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn command(&self, config_path: &Path) -> Command {
        let mut command = Command::new("node");
        command
            .arg(self.path())
            .arg(config_path)
            .current_dir(config_path.parent().unwrap_or_else(|| Path::new(".")));
        command
    }
}

#[cfg(test)]
mod tests;
