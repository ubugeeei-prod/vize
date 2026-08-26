//! Diagnostics and hover reuse of one standard editor LSP session.

use super::*;

#[cfg(unix)]
#[test]
fn bridge_diagnostics_and_hover_reuse_one_standard_editor_lsp_session() {
    use std::os::unix::fs::PermissionsExt;

    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };

    let project = tempfile::TempDir::new().unwrap();
    let project_root = project.path();
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::create_dir(project_root.join("node_modules")).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    std::fs::write(src_dir.join("Parent.vue"), "<template />\n").unwrap();
    std::fs::write(src_dir.join("Child.vue"), "<template />\n").unwrap();

    let trace_dir = tempfile::TempDir::new().unwrap();
    let trace_log = trace_dir.path().join("tsgo-argv.log");
    let traced_tsgo = trace_dir.path().join("traced-tsgo");
    std::fs::write(
        &traced_tsgo,
        vize_s0::cstr!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> {}\nexec {} \"$@\"\n",
            shell_quote_path(&trace_log),
            shell_quote_path(&corsa_path)
        )
        .as_str(),
    )
    .unwrap();
    let mut permissions = std::fs::metadata(&traced_tsgo).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&traced_tsgo, permissions).unwrap();

    let child_virtual = "export const message: string = 'hello';\n";
    let parent_virtual = "import { message } from './Child.vue.ts';\nconst shouldBeNumber: number = message;\nmessage;\n";
    let child_path = src_dir.join("Child.vue.ts");
    let parent_path = src_dir.join("Parent.vue.ts");
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(traced_tsgo),
        working_dir: Some(project_root.to_path_buf()),
        timeout_ms: 30_000,
        ..Default::default()
    });

    let (diagnostics, hover) = corsa::runtime::block_on(async {
        bridge.spawn().await.unwrap();
        let child_uri = child_path.display().to_compact_string();
        bridge
            .open_or_update_virtual_document(child_uri.as_str(), child_virtual)
            .await
            .unwrap();
        let parent_uri = parent_path.display().to_compact_string();
        let parent_uri = bridge
            .open_or_update_virtual_document(parent_uri.as_str(), parent_virtual)
            .await
            .unwrap();
        let diagnostics = bridge.get_diagnostics(parent_uri.as_str()).await.unwrap();
        let hover = bridge.hover(parent_uri.as_str(), 2, 1).await.unwrap();
        bridge.shutdown().await.unwrap();
        (diagnostics, hover)
    });

    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == Some(serde_json::json!(2322))
                && diagnostic.message == "Type 'string' is not assignable to type 'number'."
        }),
        "diagnostics should resolve the in-memory imported virtual dependency: {diagnostics:#?}"
    );
    assert!(
        hover_contains(&hover, "message") && hover_contains(&hover, "string"),
        "hover should reuse the same virtual project state after diagnostics: {hover:#?}"
    );

    let lsp_launches = std::fs::read_to_string(&trace_log)
        .unwrap_or_default()
        .lines()
        .filter(|line| line.contains("--lsp"))
        .count();
    assert!(
        lsp_launches == 1,
        "diagnostics and hover must share one standard LSP session; launches: {lsp_launches}"
    );
    assert!(!child_path.exists());
    assert!(!parent_path.exists());
}

#[cfg(unix)]
fn shell_quote_path(path: &std::path::Path) -> vize_s0::String {
    let path = path.to_string_lossy();
    vize_s0::cstr!("'{}'", path.replace('\'', "'\"'\"'"))
}
