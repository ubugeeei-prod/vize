//! Two queued didOpen notifications must not turn a private mirror into an
//! external workspace package writer (#7990).

use crate::corsa_bridge::{
    editor_session::EditorMirrorSession,
    vue_document::{
        CorsaProjectEnvironment, CorsaVueVirtualProject,
        build_vue_virtual_project_with_overlays_and_options_and_package_routes,
    },
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const CORPUS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/symlink-workspace-staging-original"
);
const APP: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/symlink-workspace-staging-original/app/App.vue.txt"
));
const OTHER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/symlink-workspace-staging-original/app/Other.vue.txt"
));

struct Fixture {
    _directory: tempfile::TempDir,
    root: PathBuf,
    app: PathBuf,
    host: PathBuf,
    other: PathBuf,
    before: BTreeMap<PathBuf, Vec<u8>>,
}

impl Fixture {
    fn new(authored_companions: bool) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        for (original, target) in [
            ("package.json", "package.json"),
            ("ui/package.json", "ui/package.json"),
            ("ui/UiButton.vue.txt", "ui/UiButton.vue"),
            ("app/tsconfig.json", "app/tsconfig.json"),
            ("app/App.vue.txt", "app/App.vue"),
            ("app/Other.vue.txt", "app/Other.vue"),
        ] {
            let destination = root.join(target);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::write(
                destination,
                fs::read(Path::new(CORPUS).join(original)).unwrap(),
            )
            .unwrap();
        }
        if authored_companions {
            fs::write(
                root.join("ui/UiButton.vue.ts"),
                "export const authored = 1;\n",
            )
            .unwrap();
            fs::write(
                root.join("ui/UiButton.d.vue.ts"),
                "export default class Authored {}\n",
            )
            .unwrap();
        }
        let app = root.join("app");
        fs::create_dir_all(app.join("node_modules")).unwrap();
        std::os::unix::fs::symlink("../../ui", app.join("node_modules/ui")).unwrap();
        let before = source_bytes(&root);
        Self {
            _directory: directory,
            host: app.join("App.vue"),
            other: app.join("Other.vue"),
            app,
            root,
            before,
        }
    }

    fn assert_authored_unchanged(&self) {
        assert_eq!(
            source_bytes(&self.root),
            self.before,
            "all authored names and full bytes must remain unchanged"
        );
        assert_eq!(
            fs::read_link(self.app.join("node_modules/ui")).unwrap(),
            Path::new("../../ui")
        );
    }

    fn build(
        &self,
        session: &EditorMirrorSession,
        source: &Path,
        content: &str,
    ) -> CorsaVueVirtualProject {
        self.try_build(session, source, content).unwrap()
    }

    fn try_build(
        &self,
        session: &EditorMirrorSession,
        source: &Path,
        content: &str,
    ) -> Result<CorsaVueVirtualProject, crate::corsa_bridge::types::CorsaBridgeError> {
        let routes = crate::PackageRouteResolver::default();
        let virtual_ts = crate::virtual_ts::VirtualTsOptions::default();
        // Both didOpen notifications can arrive before asynchronous diagnostics
        // prepare the first document. Their original complete buffers share one
        // editor revision; the first host has not imported the package yet.
        build_vue_virtual_project_with_overlays_and_options_and_package_routes(
            source,
            content,
            Default::default(),
            &[(self.other.clone(), OTHER), (self.host.clone(), APP)],
            CorsaProjectEnvironment {
                virtual_ts_options: &virtual_ts,
                package_routes: &routes,
                project_root: Some(&self.app),
                tsconfig_path: None,
                editor_session: session,
            },
        )
    }
}

fn source_bytes(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(root: &Path, directory: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let kind = entry.file_type().unwrap();
            if kind.is_symlink() {
                continue;
            }
            if kind.is_dir() {
                walk(root, &entry.path(), result);
            } else {
                result.insert(
                    entry.path().strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(entry.path()).unwrap(),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    walk(root, root, &mut result);
    result
}

#[test]
fn original_two_document_revision_stages_linked_vue_companions_privately() {
    let fixture = Fixture::new(false);
    let session = EditorMirrorSession::new();
    let session_root = session.root().unwrap().to_path_buf();
    let other = fixture.build(&session, &fixture.other, OTHER);
    fixture.assert_authored_unchanged();
    let host = fixture.build(&session, &fixture.host, APP);
    fixture.assert_authored_unchanged();
    for project in [&other, &host] {
        let request = crate::file_uri::file_uri_to_path(&project.host.request_uri).unwrap();
        assert!(request.starts_with(&session_root));
        assert!(request.is_file());
    }
    let project_root = host.session_project_root.as_ref().unwrap();
    let shadow = project_root.join("node_modules/ui");
    assert!(
        !fs::symlink_metadata(&shadow)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    for name in ["UiButton.vue.ts", "UiButton.d.vue.ts"] {
        let generated = shadow.join(name);
        assert!(
            generated.is_file(),
            "native package probe must remain present: {}",
            generated.display()
        );
        assert!(generated.canonicalize().unwrap().starts_with(&session_root));
    }
    session.clear();
    assert!(!project_root.exists());
    fixture.assert_authored_unchanged();
}

#[test]
fn genuine_authored_companions_are_never_overwritten_or_cleaned_up() {
    let fixture = Fixture::new(true);
    let session = EditorMirrorSession::new();
    fixture.build(&session, &fixture.other, OTHER);
    fixture.build(&session, &fixture.host, APP);
    fixture.assert_authored_unchanged();
    session.clear();
    fixture.assert_authored_unchanged();
}

#[test]
fn a_retargeted_cached_link_is_not_authority_to_touch_an_external_directory() {
    let fixture = Fixture::new(false);
    let session = EditorMirrorSession::new();
    let other = fixture.build(&session, &fixture.other, OTHER);
    let link = other.session_project_root.unwrap().join("node_modules/ui");
    fs::remove_file(&link).unwrap();
    std::os::unix::fs::symlink(&fixture.app, &link).unwrap();
    assert!(fixture.try_build(&session, &fixture.host, APP).is_err());
    assert_eq!(fs::read_link(&link).unwrap(), fixture.app);
    fixture.assert_authored_unchanged();
    session.clear();
    fixture.assert_authored_unchanged();
}
