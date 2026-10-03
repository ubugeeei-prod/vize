use super::*;
use std::{
    os::unix::fs::PermissionsExt,
    time::{Duration, Instant},
};

fn quoted(path: &Path) -> vize_l0::String {
    cstr!("'{}'", path.to_str().unwrap().replace('\'', "'\\''"))
}

#[test]
fn externally_revised_original_jsx_cannot_publish_old_diagnostics_and_reaps_overlay() {
    let root = tempfile::TempDir::new().unwrap();
    configuration(root.path(), true, false);
    let source = "export const view=<div/>;";
    let path = root.path().join("source.jsx");
    std::fs::write(&path, source).unwrap();
    let pid_path = root.path().join("overlay.pid");
    let wrapper = root.path().join("tsc");
    std::fs::write(&wrapper,cstr!("#!/bin/sh\nif [ \"$1\" = \"--lsp\" ]; then\n  echo \"$$\" > {}\n  printf '\\n' >> {}\nfi\nexec {} \"$@\"\n",quoted(&pid_path),quoted(&path),quoted(&backend()))).unwrap();
    std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755)).unwrap();
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(wrapper),
        working_dir: Some(root.path().to_path_buf()),
        ..Default::default()
    });
    block_on(bridge.spawn()).unwrap();
    let arena = Allocator::default();
    let original = owner(&arena, source, SourceType::jsx());
    assert!(matches!(
        block_on(bridge.check_original_jsx(&original, &path)),
        Err(OriginalProgramError::SourceChanged)
    ));
    let pid: i32 = std::fs::read_to_string(&pid_path)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let started = Instant::now();
    loop {
        // SAFETY: signal zero observes only this owned test overlay process.
        if unsafe { libc::kill(pid, 0) } != 0 {
            break;
        }
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "owned overlay was not reaped"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(std::fs::read_to_string(&path).unwrap(), cstr!("{source}\n"));
    assert_eq!(original.observation().admitted().unwrap().source(), source);
    block_on(bridge.shutdown()).unwrap();
}
