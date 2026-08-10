use std::{
    fs::{self, OpenOptions, Permissions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::Path,
};

pub(super) fn write_executable(path: &Path, source: &str) -> Result<(), String> {
    let publishing = path.with_extension("publishing");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&publishing)
        .map_err(|error| format!("create executable {}: {error}", publishing.display()))?;
    file.write_all(source.as_bytes())
        .map_err(|error| format!("write executable {}: {error}", publishing.display()))?;
    file.set_permissions(Permissions::from_mode(0o755))
        .map_err(|error| format!("chmod executable {}: {error}", publishing.display()))?;
    file.sync_all()
        .map_err(|error| format!("sync executable {}: {error}", publishing.display()))?;
    drop(file);
    fs::rename(&publishing, path).map_err(|error| {
        format!(
            "publish executable {} as {}: {error}",
            publishing.display(),
            path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use std::{fs::OpenOptions, process::Command};

    use super::*;

    #[test]
    fn publication_replaces_a_busy_inode_before_exec() {
        let root = tempfile::TempDir::new().expect("temp root");
        let executable = root.path().join("traced-tsgo");
        fs::write(&executable, "#!/bin/sh\nprintf 'old'\n").expect("seed executable");
        let mut busy_inode = OpenOptions::new()
            .write(true)
            .open(&executable)
            .expect("hold old executable inode open for writing");
        busy_inode.write_all(b"#").expect("keep old inode busy");

        write_executable(&executable, "#!/bin/sh\nprintf 'ready'\n").expect("publish executable");
        let output = Command::new(&executable)
            .output()
            .expect("execute published wrapper");

        assert!(output.status.success());
        assert_eq!(output.stdout, b"ready");
        drop(busy_inode);
    }
}
