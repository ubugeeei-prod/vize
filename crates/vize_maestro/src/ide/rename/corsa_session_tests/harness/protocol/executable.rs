use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use vize_s0::cstr;

fn immutable_wrapper() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/corsa-session-wrapper.sh")
}

pub(super) fn link_session_wrapper(path: &Path) -> Result<(), String> {
    let wrapper = immutable_wrapper();
    let metadata = fs::metadata(&wrapper).map_err(|error| {
        cstr!(
            "inspect immutable Corsa wrapper {}: {error}",
            wrapper.display()
        )
    })?;
    if metadata.permissions().mode() & 0o111 == 0 {
        return Err(cstr!(
            "immutable Corsa wrapper is not executable: {}",
            wrapper.display()
        )
        .into());
    }
    std::os::unix::fs::symlink(&wrapper, path).map_err(|error| {
        cstr!(
            "link immutable Corsa wrapper {} as {}: {error}",
            wrapper.display(),
            path.display()
        )
        .into()
    })
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    use std::{fs::Permissions, os::unix::fs::PermissionsExt};
    use std::{
        fs::{self, OpenOptions},
        io::Write,
        process::Command,
    };

    use super::*;

    #[test]
    fn session_wrapper_never_publishes_a_writable_executable_inode() {
        let root = tempfile::TempDir::new().expect("temp root");
        let traces = root.path().join("protocol-traces");
        fs::create_dir(&traces).expect("trace directory");
        fs::write(root.path().join("actual-tsgo.path"), "/usr/bin/true").expect("fake tsgo path");
        fs::write(
            root.path().join("trace-stdio.pl"),
            super::super::proxy::TRACE_STDIO_PROXY,
        )
        .expect("stdio trace proxy");
        let wrapper = root.path().join("traced-tsgo");
        link_session_wrapper(&wrapper).expect("link session wrapper");
        let mut dynamic_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.path().join("session-data"))
            .expect("hold session data writable");
        dynamic_file.write_all(b"open").expect("write session data");

        let metadata = fs::symlink_metadata(&wrapper).expect("wrapper metadata");
        assert!(metadata.file_type().is_symlink());
        assert_eq!(
            wrapper.canonicalize().expect("canonical session wrapper"),
            immutable_wrapper().canonicalize().expect("canonical asset")
        );
        let output = Command::new(&wrapper)
            .current_dir(root.path())
            .output()
            .expect("execute wrapper");
        assert!(output.status.success(), "wrapper stderr: {output:?}");
        let trace_count = fs::read_dir(&traces).expect("trace files").count();
        let api = Command::new(&wrapper)
            .arg("--api")
            .current_dir(root.path())
            .output()
            .expect("execute API bypass");
        assert!(api.status.success(), "API bypass stderr: {api:?}");
        assert_eq!(
            fs::read_dir(&traces).expect("trace files").count(),
            trace_count,
            "the editor byte proxy must not alter the MsgPack API transport"
        );
        drop(dynamic_file);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn direct_final_publication_is_the_executable_busy_negative_control() {
        let root = tempfile::TempDir::new().expect("temp root");
        let executable = root.path().join("legacy-wrapper");
        fs::write(&executable, "#!/bin/sh\nexit 0\n").expect("seed executable");
        fs::set_permissions(&executable, Permissions::from_mode(0o755)).expect("chmod executable");
        let _busy_inode = OpenOptions::new()
            .write(true)
            .open(&executable)
            .expect("hold executable inode writable");

        let error = Command::new(&executable)
            .output()
            .expect_err("a writable executable inode must fail closed");
        assert_eq!(error.raw_os_error(), Some(26));
    }
}
