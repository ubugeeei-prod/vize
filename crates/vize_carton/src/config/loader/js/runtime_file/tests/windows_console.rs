use super::{NodeRuntimeFile, Path, PathBuf, env, fs};

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetConsoleWindow() -> *mut std::ffi::c_void;
}

#[test]
fn runtime_command_has_no_console() {
    const CHILD: &str = "VIZE_CONFIG_CONSOLE_CHILD";
    if let Some(marker) = env::var_os(CHILD) {
        // SAFETY: GetConsoleWindow takes no arguments and returns the calling
        // process's console handle; the handle is only compared with null.
        assert_eq!(unsafe { GetConsoleWindow() }, std::ptr::null_mut());
        fs::write(marker, b"console-free").unwrap();
        return;
    }

    // Stand in for node.exe with this test process so the exact production
    // command can observe its own console without loading a native Node addon.
    let project = tempfile::tempdir().unwrap();
    fs::copy(env::current_exe().unwrap(), project.path().join("node.exe")).unwrap();
    let path = env::var_os("PATH").unwrap_or_default();
    let path = env::join_paths(
        std::iter::once(project.path().to_path_buf()).chain(env::split_paths(&path)),
    )
    .unwrap();
    let runtime = NodeRuntimeFile {
        path: PathBuf::from("--exact"),
        _lease: None,
        _temporary: None,
    };
    // Rust's test names omit the crate prefix from module_path!().
    let test_name = concat!(module_path!(), "::runtime_command_has_no_console")
        .split_once("::")
        .unwrap()
        .1;
    let marker = project.path().join("console-result");
    let output = runtime
        .command(Path::new(test_name))
        .current_dir(project.path())
        .env("PATH", path)
        .env(CHILD, &marker)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stderr, b"");
    assert_eq!(fs::read(marker).unwrap(), b"console-free");
}
