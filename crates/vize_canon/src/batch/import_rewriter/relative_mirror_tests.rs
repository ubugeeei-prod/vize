#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use tempfile::TempDir;

use super::ImportRewriter;
use crate::batch::virtual_project::script_virtual_path;

struct Fixture {
    _workspace: TempDir,
    root: PathBuf,
    sibling: PathBuf,
    mirror: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let workspace = TempDir::new().unwrap();
        let workspace_path = workspace.path().canonicalize().unwrap();
        let root = workspace_path.join("packages/app");
        let sibling = workspace_path.join("packages/ui");
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir_all(sibling.join("src")).unwrap();
        let mirror = workspace_path.join("mirror");
        Self {
            _workspace: workspace,
            root,
            sibling,
            mirror,
        }
    }

    fn project(&self, source: &str, dir: &Path, generated: bool) -> std::string::String {
        let rewriter = ImportRewriter::new();
        let roots = (self.root.as_path(), self.mirror.as_path());
        let options = super::VirtualProjectRewriteOptions {
            has_external_sources: true,
            preserve_relative_declarations: false,
            mirrorable_project_files: None,
            alias_rewrite_policy: None,
            module_resolver: None,
        };
        let (rewritten, _) = if generated {
            rewriter.rewrite_generated_for_virtual_project_with_star_capture(
                source,
                SourceType::ts(),
                roots,
                Some(dir),
                options,
                false,
            )
        } else {
            rewriter.rewrite_for_virtual_project_with_star_capture(
                source,
                SourceType::ts(),
                roots,
                Some(dir),
                options,
                false,
            )
        };
        rewritten.code.to_string()
    }

    fn expected_specifier(&self, source_dir: &Path, target: &Path) -> std::string::String {
        let source = script_virtual_path((&self.root, &self.mirror), source_dir, "", true).unwrap();
        let target = script_virtual_path((&self.root, &self.mirror), target, "", true).unwrap();
        let common = source
            .ancestors()
            .find(|ancestor| target.starts_with(ancestor))
            .unwrap();
        let mut relative = PathBuf::new();
        for _ in source.strip_prefix(common).unwrap().components() {
            relative.push("..");
        }
        relative.push(target.strip_prefix(common).unwrap());
        let relative = relative.to_string_lossy().replace('\\', "/");
        if relative.starts_with("../") {
            relative
        } else {
            format!("./{relative}")
        }
    }
}

#[test]
fn relative_sibling_vue_imports_follow_external_mirrors_in_both_directions() {
    let fixture = Fixture::new();
    let app_dir = fixture.root.join("src");
    let sibling_dir = fixture.sibling.join("src");
    std::fs::write(sibling_dir.join("Shared.vue"), "<template />").unwrap();
    std::fs::write(app_dir.join("App.vue"), "<template />").unwrap();
    for generated in [false, true] {
        let expected = fixture.expected_specifier(&app_dir, &sibling_dir.join("Shared.vue"));
        assert_eq!(
            fixture.project(
                "import Shared from '../../ui/src/Shared.vue';",
                &app_dir,
                generated
            ),
            format!("import Shared from '{expected}.ts';")
        );
        let expected = fixture.expected_specifier(&sibling_dir, &app_dir.join("App.vue"));
        assert_eq!(
            fixture.project(
                "export { default } from '../../app/src/App.vue';",
                &sibling_dir,
                generated
            ),
            format!("export {{ default }} from '{expected}.ts';")
        );
    }
}

#[test]
fn crossing_scripts_keep_authored_extension_and_directory_spelling() {
    let fixture = Fixture::new();
    let app_dir = fixture.root.join("src");
    let sibling_dir = fixture.sibling.join("src");
    std::fs::write(sibling_dir.join("model.ts"), "export type Model = number").unwrap();
    std::fs::create_dir_all(sibling_dir.join("directory")).unwrap();
    std::fs::write(sibling_dir.join("directory/index.ts"), "export {}").unwrap();
    for suffix in ["model", "model.js", "model.ts", "directory"] {
        let expected = fixture.expected_specifier(&app_dir, &sibling_dir.join(suffix));
        let source = format!(
            "export type {{ Model }} from '../../ui/src/{suffix}'; type Lazy = import('../../ui/src/{suffix}').Model; const load = import('../../ui/src/{suffix}');"
        );
        let expected = source.replace(&format!("../../ui/src/{suffix}"), &expected);
        for generated in [false, true] {
            assert_eq!(fixture.project(&source, &app_dir, generated), expected);
        }
    }
}

#[test]
fn external_neighbors_and_missing_generated_collisions_keep_their_semantics() {
    let fixture = Fixture::new();
    let sibling_dir = fixture.sibling.join("src");
    std::fs::write(sibling_dir.join("Shared.vue"), "<template />").unwrap();
    let source = "import Shared from './Shared.vue'; import Missing from '../../app/src/Missing.vue'; import Forged from './Shared.vue.ts';";
    let missing = crate::batch::virtual_specifier_message::MISSING_VUE_IMPORT_SENTINEL;
    let forged = crate::batch::AUTHORED_VUE_TS_SENTINEL;
    let expected = format!(
        "import Shared from './Shared.vue.ts'; import Missing from '../../app/src/Missing.vue.ts{missing}'; import Forged from './Shared.vue.ts{forged}';"
    );
    for generated in [false, true] {
        assert_eq!(fixture.project(source, &sibling_dir, generated), expected);
    }
}

#[test]
fn crossing_authored_vue_ts_keeps_its_collision_free_module_identity() {
    let fixture = Fixture::new();
    let app_dir = fixture.root.join("src");
    let sibling_dir = fixture.sibling.join("src");
    std::fs::write(sibling_dir.join("Shared.vue"), "<template />").unwrap();
    std::fs::write(
        sibling_dir.join("Shared.vue.ts"),
        "export const authored = 1",
    )
    .unwrap();
    let expected = fixture.expected_specifier(&app_dir, &sibling_dir.join("Shared.vue"));
    let marker = crate::batch::AUTHORED_VUE_TS_ALIAS_SENTINEL;
    for generated in [false, true] {
        assert_eq!(
            fixture.project(
                "import { authored } from '../../ui/src/Shared.vue.ts';",
                &app_dir,
                generated
            ),
            format!("import {{ authored }} from '{expected}.ts{marker}';")
        );
    }
}

#[cfg(unix)]
#[test]
fn relative_file_symlinks_follow_the_registered_physical_identity() {
    let fixture = Fixture::new();
    let app_dir = fixture.root.join("src");
    let sibling_dir = fixture.sibling.join("src");
    std::fs::write(sibling_dir.join("Shared.vue"), "<template />").unwrap();
    std::fs::write(sibling_dir.join("model.ts"), "export const value = 1").unwrap();
    std::os::unix::fs::symlink(sibling_dir.join("Shared.vue"), app_dir.join("Linked.vue")).unwrap();
    std::os::unix::fs::symlink(
        sibling_dir.join("model.ts"),
        app_dir.join("linked-model.ts"),
    )
    .unwrap();
    let vue = fixture.expected_specifier(&app_dir, &sibling_dir.join("Shared.vue"));
    let model = fixture.expected_specifier(&app_dir, &sibling_dir.join("model"));
    for generated in [false, true] {
        assert_eq!(
            fixture.project("import Shared from './Linked.vue';", &app_dir, generated),
            format!("import Shared from '{vue}.ts';")
        );
        assert_eq!(
            fixture.project(
                "export { value } from './linked-model';",
                &app_dir,
                generated
            ),
            format!("export {{ value }} from '{model}';")
        );
        assert_eq!(
            fixture.project(
                "export { value } from './linked-model.js';",
                &app_dir,
                generated
            ),
            format!("export {{ value }} from '{model}.js';")
        );
    }
}
