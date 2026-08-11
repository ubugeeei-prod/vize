//! User-authored `paths` stay authoritative over installed package shadows.

use std::path::{Path, PathBuf};

use super::vue_document::{CorsaVueVirtualDocumentOptions, build_vue_virtual_project};

#[test]
fn editor_rewrite_prefers_user_path_over_same_named_installed_package() {
    let fixture = fixture();
    let project = build_vue_virtual_project(
        &fixture.host,
        SOURCE,
        CorsaVueVirtualDocumentOptions::default(),
    )
    .unwrap();

    assert!(
        !project.host.code.contains(".vize-package-routes"),
        "user paths must bypass the installed package selector: {}",
        project.host.code
    );
    assert!(project.host.code.contains("Local.vue"));
}

#[test]
fn native_editor_diagnostics_and_definition_follow_the_user_path() {
    let Some(corsa_path) = std::env::var_os("CORSA_PATH").map(PathBuf::from) else {
        return;
    };
    if !corsa_path.is_file() {
        return;
    }
    let fixture = fixture();
    install_runtime_stubs(&fixture.app);
    let bridge = super::CorsaBridge::with_config(super::CorsaBridgeConfig {
        corsa_path: Some(corsa_path),
        working_dir: Some(fixture.app.clone()),
        timeout_ms: 30_000,
        ..Default::default()
    });

    corsa::runtime::block_on(async {
        bridge.spawn().await.unwrap();
        let document = bridge
            .open_vue_virtual_document(
                &fixture.host,
                SOURCE,
                CorsaVueVirtualDocumentOptions::default(),
            )
            .await
            .unwrap();
        let diagnostics = bridge.get_diagnostics(&document.request_uri).await.unwrap();
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.code.as_ref().is_none_or(|code| code != 2353)),
            "installed package won over the user path: {diagnostics:#?}"
        );
        let offset = document.code.find("Widget").unwrap() + 1;
        let before = &document.code[..offset];
        let line = before.bytes().filter(|byte| *byte == b'\n').count() as u32;
        let character = before
            .rsplit_once('\n')
            .map_or(before.len(), |(_, tail)| tail.len()) as u32;
        let definitions = bridge
            .definition(&document.request_uri, line, character)
            .await
            .unwrap();
        let paths = definitions
            .iter()
            .filter_map(|location| crate::file_uri::file_uri_to_path(&location.uri))
            .collect::<Vec<_>>();
        assert!(
            paths
                .iter()
                .any(|path| path.to_string_lossy().contains("Local.vue")),
            "definition did not follow the user path: {paths:#?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.to_string_lossy().contains("Installed.vue")),
            "definition followed the installed package: {paths:#?}"
        );
        bridge.shutdown().await.unwrap();
    });
}

#[test]
fn native_editor_keeps_paths_declarations_inferred() {
    let Some(corsa_path) = std::env::var_os("CORSA_PATH").map(PathBuf::from) else {
        return;
    };
    if !corsa_path.is_file() {
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app");
    let host = app.join("src/Host.vue");
    write(
        &app.join("tsconfig.json"),
        r#"{"compilerOptions":{"allowJs":true,"checkJs":true,"moduleResolution":"bundler","paths":{"@/*":["./src/*"]}}}"#,
    );
    let source = "<script>import { transactionList } from '@/api/remote-search';\nconst marker = 'first';\nexport default { methods: { load() { return transactionList() } } };\nvoid marker;\n</script>\n";
    write(&host, source);
    write(
        &app.join("src/api/remote-search.d.ts"),
        "export declare function transactionList(): Promise<unknown>;\n",
    );
    install_runtime_stubs(&app);
    let bridge = super::CorsaBridge::with_config(super::CorsaBridgeConfig {
        corsa_path: Some(corsa_path),
        working_dir: Some(app),
        timeout_ms: 30_000,
        ..Default::default()
    });

    corsa::runtime::block_on(async {
        bridge.spawn().await.unwrap();
        let document = bridge
            .open_vue_virtual_document(
                &host,
                source,
                CorsaVueVirtualDocumentOptions {
                    options_api: true,
                    legacy_vue2: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let diagnostics = bridge.get_diagnostics(&document.request_uri).await.unwrap();
        assert!(
            diagnostics.is_empty(),
            "reachable inferred declaration widened or failed resolution: {diagnostics:#?}"
        );
        let changed = source.replace("'first'", "'second'");
        write(&host, &changed);
        let changed_document = bridge
            .open_vue_virtual_document(
                &host,
                &changed,
                CorsaVueVirtualDocumentOptions {
                    options_api: true,
                    legacy_vue2: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let changed_diagnostics = bridge
            .get_diagnostics(&changed_document.request_uri)
            .await
            .unwrap();
        assert!(
            changed_diagnostics.is_empty(),
            "inferred declaration widened or detached after editor change: {changed_diagnostics:#?}"
        );
        bridge.shutdown().await.unwrap();
    });
}

const SOURCE: &str = r#"<script setup lang="ts">
import Widget from '@scope/ui'
type Props = InstanceType<typeof Widget>['$props']
const props: Props = { localOnly: 'ok' }
void props
</script>
"#;

struct Fixture {
    _root: tempfile::TempDir,
    app: PathBuf,
    host: PathBuf,
}

fn fixture() -> Fixture {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app");
    let host = app.join("src/Host.vue");
    write(
        &app.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"moduleResolution":"bundler","paths":{"@scope/ui":["./src/Local.vue"]}}}"#,
    );
    write(&host, SOURCE);
    write(
        &app.join("src/Local.vue"),
        "<script setup lang=\"ts\">defineProps<{ localOnly: string }>()</script>\n",
    );
    let package = app.join("node_modules/@scope/ui");
    write(
        &package.join("package.json"),
        "{\"name\":\"@scope/ui\",\"exports\":\"./Installed.vue\"}\n",
    );
    write(
        &package.join("Installed.vue"),
        "<script setup lang=\"ts\">defineProps<{ installedOnly: number }>()</script>\n",
    );
    Fixture {
        _root: root,
        app,
        host,
    }
}

fn install_runtime_stubs(project_root: &Path) {
    let node_modules = project_root.join("node_modules");
    crate::batch::write_vue_facade(&node_modules).unwrap();
    let runtime_dom = node_modules.join("@vue/runtime-dom");
    write(
        &runtime_dom.join("package.json"),
        "{\"name\":\"@vue/runtime-dom\",\"types\":\"index.d.ts\"}\n",
    );
    write(
        &runtime_dom.join("index.d.ts"),
        crate::batch::VUE_RUNTIME_DOM_STUB_TYPES,
    );
}

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}
