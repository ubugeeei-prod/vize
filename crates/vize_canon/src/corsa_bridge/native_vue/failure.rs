//! Actual failed/timed-out independent sessions must be reaped without writes.
#![cfg(unix)]

use super::*;
use corsa::runtime::block_on;
use std::{
    os::unix::fs::PermissionsExt,
    time::{Duration, Instant},
};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{SurfaceParseOptions, container::vue::DescriptorOptions};
use vize_l1_to_l2::native_file::lower_sfc_native;

fn quote(path: &Path) -> String {
    cstr!("'{}'", path.to_str().unwrap().replace('\'', "'\\''"))
}

#[test]
fn failed_and_timed_out_native_vue_sessions_preserve_source_and_reap_processes() {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let executable = vize_carton::corsa_resolver::resolve_corsa_executable(
        vize_carton::corsa_resolver::CorsaResolveRequest {
            explicit_path: std::env::var_os("CORSA_PATH").as_deref().map(Path::new),
            project_root: Some(project_root),
        },
    )
    .unwrap();
    for (api, pause) in [(true, false), (false, false), (false, true)] {
        let root = tempfile::TempDir::new().unwrap();
        std::fs::write(
            root.path().join("tsconfig.json"),
            r#"{"compilerOptions":{"types":[],"allowJs":true,"checkJs":true},"include":["**/*"]}"#,
        )
        .unwrap();
        std::fs::write(root.path().join("seed.ts"), "export {};").unwrap();
        let source = "<template>{{value.missing}}</template><script setup>const value=1;</script>";
        let path = root.path().join("Source.vue");
        std::fs::write(&path, source).unwrap();
        let pid_file = root.path().join("owned.pid");
        let wrapper = root.path().join("tsc");
        let branch = if api {
            "for arg in \"$@\"; do case \"$arg\" in --callbacks=*) fail=true;; esac; done"
        } else {
            "if [ \"$1\" = \"--lsp\" ]; then fail=true; fi"
        };
        std::fs::write(&wrapper, cstr!("#!/bin/sh\nfail=false\n{branch}\nif [ \"$fail\" = true ]; then\n echo \"$$\" > {}\n {}\n exit 23\nfi\nexec {} \"$@\"\n", quote(&pid_file), if pause { "sleep 1" } else { ":" }, quote(&executable))).unwrap();
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut bridge = CorsaBridge::with_config(crate::CorsaBridgeConfig {
            corsa_path: Some(wrapper),
            working_dir: Some(root.path().to_path_buf()),
            ..Default::default()
        });
        block_on(bridge.spawn()).unwrap();
        if pause {
            bridge.config.timeout_ms = 250;
        }
        let arena = Allocator::default();
        let original = lower_sfc_native(
            &arena,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        let result = block_on(bridge.check_native_vue(original.admitted().unwrap(), &path));
        if pause {
            assert!(matches!(
                result,
                Err(NativeVueError::Backend(CorsaBridgeError::Timeout))
            ));
        } else {
            assert!(matches!(
                result,
                Err(NativeVueError::Backend(
                    CorsaBridgeError::CommunicationError(_)
                ))
            ));
        }
        let started = Instant::now();
        while !pid_file.exists() {
            assert!(started.elapsed() < Duration::from_secs(10));
            std::thread::sleep(Duration::from_millis(10));
        }
        let pid: i32 = std::fs::read_to_string(&pid_file)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        loop {
            // SAFETY: signal zero only observes the owned test subprocess.
            if unsafe { libc::kill(pid, 0) } != 0 {
                break;
            }
            assert!(
                started.elapsed() < Duration::from_secs(10),
                "owned native session was not reaped"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        loop {
            let closed = block_on(bridge.shutdown());
            if closed.is_ok() {
                break;
            }
            assert!(matches!(closed, Err(CorsaBridgeError::Timeout)));
            assert!(started.elapsed() < Duration::from_secs(10));
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
        assert!(!root.path().join("Source.vue.mjs").exists());
    }
}
