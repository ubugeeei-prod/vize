//! The actual diagnosing project must retain the admitted inherited options.

use super::*;
use std::os::unix::fs::PermissionsExt;
use vize_canon::{NativeVueError, OriginalProgramError};

fn quote(path: &Path) -> String {
    cstr!("'{}'", path.to_str().unwrap().replace('\'', "'\\''"))
}

#[test]
fn inherited_options_changed_at_lsp_startup_refuse_without_sfc_or_root_writes() {
    for ts in [false, true] {
        let root = tempfile::TempDir::new().unwrap();
        drop(project(
            root.path(),
            serde_json::json!({"extends":"./base.json","include":["**/*"]}),
        ));
        let initial = r#"{"compilerOptions":{"strict":false,"allowJs":true,"checkJs":true,"types":[],"noEmit":true,"module":"ESNext","moduleResolution":"Bundler","target":"ESNext"}}"#;
        let changed = initial.replace("\"strict\":false", "\"strict\":true");
        let base = root.path().join("base.json");
        std::fs::write(&base, initial).unwrap();
        let configuration = std::fs::read(root.path().join("tsconfig.json")).unwrap();
        let source = cstr!(
            "<template>{{{{value.toFixed()}}}}</template><script setup{}>const value=null;</script>",
            if ts { " lang=ts" } else { "" }
        );
        let path = root.path().join("Original.vue");
        std::fs::write(&path, &source).unwrap();
        let replacement = root.path().join("changed.json");
        std::fs::write(&replacement, &changed).unwrap();
        let pid_path = root.path().join("overlay.pid");
        let wrapper = root.path().join("tsc");
        std::fs::write(&wrapper, cstr!("#!/bin/sh\nif [ \"$1\" = \"--lsp\" ]; then\n echo \"$$\" > {}\n cat {} > {}\nfi\nexec {} \"$@\"\n", quote(&pid_path), quote(&replacement), quote(&base), quote(&backend()))).unwrap();
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755)).unwrap();
        let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(wrapper),
            working_dir: Some(root.path().to_path_buf()),
            ..Default::default()
        });
        block_on(bridge.spawn()).unwrap();
        let arena = Allocator::default();
        let observed = lower_sfc_native(&arena, &source, options());
        let result = block_on(bridge.check_native_vue(observed.admitted().unwrap(), &path));
        assert!(matches!(
            result,
            Err(NativeVueError::Identity(
                OriginalProgramError::ConfigurationChanged
            ))
        ));
        assert_eq!(
            observed.file().unwrap().file().artifact().source(),
            source.as_str()
        );
        assert_eq!(
            std::fs::read(root.path().join("tsconfig.json")).unwrap(),
            configuration
        );
        assert_eq!(std::fs::read(&base).unwrap(), changed.as_bytes());
        assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
        assert!(
            !root
                .path()
                .join(if ts {
                    "Original.vue.ts"
                } else {
                    "Original.vue.mjs"
                })
                .exists()
        );
        let pid: i32 = std::fs::read_to_string(&pid_path)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        // SAFETY: signal zero observes only this owned diagnosing exec child.
        assert_ne!(
            unsafe { libc::kill(pid, 0) },
            0,
            "diagnosing process survived refusal"
        );
        block_on(bridge.shutdown()).unwrap();
    }
}
