//! Deliberate real-process startup failures must preserve source and reap overlays.
#![cfg(unix)]

use crate::{CorsaBridge, CorsaBridgeConfig, CorsaBridgeError, OriginalProgramError};
use corsa::runtime::block_on;
use std::{
    path::Path,
    time::{Duration, Instant},
};
use vize_l0::{Allocator, String, cstr};
use vize_l1::embed::Lang;

fn shell_quote(path: &Path) -> String {
    cstr!("'{}'", path.to_str().unwrap().replace('\'', "'\\''"))
}

#[test]
fn failed_and_timed_out_original_overlays_are_reaped_without_source_writes() {
    use std::os::unix::fs::PermissionsExt;
    for pause in [false, true] {
        let root = tempfile::TempDir::new().unwrap();
        drop(project(root.path(), true));
        let path = root.path().join("source.mjs");
        let source = "const value=1; value.missing;";
        std::fs::write(&path, source).unwrap();
        let pid_path = root.path().join("overlay.pid");
        let wrapper = root.path().join("tsc");
        let script = cstr!(
            "#!/bin/sh\nif [ \"$1\" = \"--lsp\" ]; then\n  echo \"$$\" > {}\n  {}\n  exit 23\nfi\nexec {} \"$@\"\n",
            shell_quote(&pid_path),
            if pause { "sleep 1" } else { ":" },
            shell_quote(&backend())
        );
        std::fs::write(&wrapper, script).unwrap();
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut bridge = CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(wrapper),
            working_dir: Some(root.path().to_path_buf()),
            timeout_ms: 30000,
            ..Default::default()
        });
        block_on(bridge.spawn()).unwrap();
        if pause {
            bridge.config.timeout_ms = 250;
        }
        let arena = Allocator::default();
        let original = file(&arena, source, Lang::Js);
        let result = block_on(bridge.check_original_program(&original, &path));
        if pause {
            assert!(matches!(
                result,
                Err(OriginalProgramError::Backend(CorsaBridgeError::Timeout))
            ));
        } else {
            assert!(matches!(
                result,
                Err(OriginalProgramError::Backend(
                    CorsaBridgeError::CommunicationError(_)
                ))
            ));
        }
        let started = Instant::now();
        while !pid_path.exists() {
            assert!(
                started.elapsed() < Duration::from_secs(10),
                "owned LSP process never started"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        let pid: i32 = std::fs::read_to_string(pid_path)
            .unwrap()
            .trim()
            .parse()
            .unwrap();

        loop {
            // SAFETY: signal zero only observes whether this owned test child exists.
            let exists = unsafe { libc::kill(pid, 0) } == 0;
            if !exists {
                break;
            }
            assert!(
                started.elapsed() < Duration::from_secs(10),
                "owned LSP process was not reaped"
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
    }
}

#[test]
fn diagnosing_project_and_configuration_changes_refuse_and_reap_real_overlays() {
    use std::os::unix::fs::PermissionsExt;
    for nested in [true, false] {
        let root = tempfile::TempDir::new().unwrap();
        drop(project(root.path(), true));
        let config = root.path().join("tsconfig.json");
        let configuration = std::fs::read(&config).unwrap();
        let subdirectory = root.path().join("nested");
        std::fs::create_dir(&subdirectory).unwrap();
        let path = subdirectory.join("source.mjs");
        let source = "const value=1; value.missing;";
        std::fs::write(&path, source).unwrap();
        let pid_path = root.path().join("overlay.pid");
        let wrapper = root.path().join("tsc");
        // Root membership and the front-end nested-config check run before
        // the independent diagnosing process starts. Only this real process
        // startup changes its configured project or the root's source bytes.
        let mutation = if nested {
            cstr!(
                "printf '%s' '{{\"compilerOptions\":{{\"allowJs\":true,\"checkJs\":true,\"types\":[],\"noEmit\":true}}}}' > {}",
                shell_quote(&subdirectory.join("tsconfig.json"))
            )
        } else {
            cstr!("echo ' ' >> {}", shell_quote(&config))
        };
        let script = cstr!(
            "#!/bin/sh\nif [ \"$1\" = \"--lsp\" ]; then\n  echo \"$$\" > {}\n  {}\nfi\nexec {} \"$@\"\n",
            shell_quote(&pid_path),
            mutation,
            shell_quote(&backend())
        );
        std::fs::write(&wrapper, script).unwrap();
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755)).unwrap();
        let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(wrapper),
            working_dir: Some(root.path().to_path_buf()),
            timeout_ms: 30000,
            ..Default::default()
        });
        block_on(bridge.spawn()).unwrap();
        let arena = Allocator::default();
        let original = file(&arena, source, Lang::Js);
        assert!(original.is_complete(), "{:?}", original.issues());
        let result = block_on(bridge.check_original_program(&original, &path));
        if nested {
            assert!(matches!(
                result,
                Err(OriginalProgramError::UnconfiguredSource)
            ));
            assert_eq!(std::fs::read(&config).unwrap(), configuration);
        } else {
            assert!(matches!(
                result,
                Err(OriginalProgramError::ConfigurationChanged)
            ));
            assert_ne!(std::fs::read(&config).unwrap(), configuration);
        }
        let pid: i32 = std::fs::read_to_string(&pid_path)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        let started = Instant::now();
        loop {
            // SAFETY: signal zero only observes this owned exec child's PID.
            if unsafe { libc::kill(pid, 0) } != 0 {
                break;
            }
            assert!(
                started.elapsed() < Duration::from_secs(10),
                "overlay not reaped"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        block_on(bridge.shutdown()).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
    }
}

fn backend() -> std::path::PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    vize_carton::corsa_resolver::resolve_corsa_executable(
        vize_carton::corsa_resolver::CorsaResolveRequest {
            explicit_path: std::env::var_os("CORSA_PATH").as_deref().map(Path::new),
            project_root: Some(root),
        },
    )
    .unwrap()
}
fn project(root: &Path, _force: bool) -> CorsaBridge {
    std::fs::write(root.join("tsconfig.json"),r#"{"compilerOptions":{"allowJs":true,"checkJs":true,"moduleDetection":"force","types":[],"noEmit":true,"module":"ESNext","moduleResolution":"Bundler","target":"ESNext"},"include":["**/*"]}"#).unwrap();
    CorsaBridge::with_config(CorsaBridgeConfig {
        working_dir: Some(root.to_path_buf()),
        ..Default::default()
    })
}
fn file<'a>(arena: &'a Allocator, source: &'a str, lang: Lang) -> vize_l2::file::FileArtifact<'a> {
    use vize_l0::SourceRoot;
    use vize_l1::embed::{
        EmbedSource,
        syntax::{ProgramOptions, parse_program_once},
    };
    use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};
    let block = SourceRoot::new(source).unwrap().whole_block();
    let syntax = parse_program_once(
        arena,
        EmbedSource::authored(source, block.span()).unwrap(),
        ProgramOptions::module(lang),
    );
    let mut producer = FileProducer::new(arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(syntax.admitted_program().unwrap(), block, 17).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    producer.finish().unwrap()
}
