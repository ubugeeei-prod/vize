//! Whole Tree slot props must instantiate both event inference arguments.

mod replay;

use super::{install, minimal, native, packets};
use crate::{BatchTypeChecker, BatchTypeCheckerOptions, BatchTypeCheckerTrait};
use std::{fs, path::Path};
use vize_carton::{String, cstr};

const ROOT: &str = "packages/core/src/Tree/TreeRoot.vue";
const ITEM: &str = "packages/core/src/Tree/TreeItem.vue";
const STORY: &str = "packages/core/src/Tree/story/TreeAsync.story.vue";
const INDEX: &str = "packages/core/src/Tree/index.ts";
const CORPUS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/generic-event-spreads"
);

fn source(name: &str) -> String {
    fs::read_to_string(Path::new(CORPUS).join(name))
        .unwrap()
        .into()
}

#[test]
fn event_spread_native_contract_keeps_slot_inference_order_and_real_refusals() {
    let Some(native) = native() else { return };
    let temporary = tempfile::tempdir().unwrap();
    let (root, provider) = minimal::install(temporary.path());
    packets::retain("tree-minimal-provider", &provider);
    qualify(&root, &native, false);
    if std::env::var("VIZE_TEST_REQUIRE_PIN_INPUT_FIXTURE").as_deref() == Ok("1") {
        let temporary = tempfile::tempdir().unwrap();
        let (root, provider) = install::snapshot(temporary.path());
        packets::retain("tree-pinned-provider", &provider);
        for (path, name) in [
            (ROOT, "TreeRoot.vue.txt"),
            (ITEM, "TreeItem.vue.txt"),
            (STORY, "TreeAsync.story.vue.txt"),
            (INDEX, "index.ts.txt"),
        ] {
            assert_eq!(
                fs::read_to_string(root.join(path)).unwrap(),
                source(name).as_str()
            );
        }
        qualify(&root, &native, true);
    }
}

fn qualify(root: &Path, native: &Path, pinned: bool) {
    let original = source(if pinned {
        "TreeAsync.story.vue.txt"
    } else {
        "minimal-story.vue.txt"
    });
    let cases = if pinned {
        vec![("pinned-original", original, false, true)]
    } else {
        let wrong = original.replace(
            "event: ToggleEvent<TreeNode>",
            "event: ToggleEvent<{ id: number }>",
        );
        vec![
            ("minimal-original", original.clone(), false, true),
            ("minimal-wrong-callback", wrong.into(), true, true),
            (
                "minimal-spread-last",
                original
                    .replace(
                        "v-bind=\"item.bind\"",
                        ":value=\"{ id: 0 }\" v-bind=\"item.bind\"",
                    )
                    .into(),
                false,
                true,
            ),
            (
                "minimal-value-last",
                original
                    .replace(
                        "v-bind=\"item.bind\"",
                        "v-bind=\"item.bind\" :value=\"{ id: 0 }\"",
                    )
                    .into(),
                true,
                true,
            ),
            (
                "minimal-no-spread",
                original
                    .replace(
                        "v-bind=\"item.bind\"",
                        ":value=\"item.value\" :level=\"item.bind.level\"",
                    )
                    .into(),
                false,
                false,
            ),
            (
                "minimal-lexical-shadow",
                original
                    .replace("const treeData =", "const item = { bind: { value: { id: 0 }, level: 0 } };\nvoid item;\nconst treeData =")
                    .into(),
                false,
                true,
            ),
        ]
    };
    let mut verdicts = Vec::new();
    for (name, story, wrong, spread) in cases {
        if !pinned {
            for (path, name) in [
                (ROOT, "minimal-root.vue.txt"),
                (ITEM, "minimal-item.vue.txt"),
            ] {
                fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
                fs::write(root.join(path), source(name).as_str()).unwrap();
            }
        }
        fs::create_dir_all(root.join(STORY).parent().unwrap()).unwrap();
        fs::write(root.join(STORY), story.as_str()).unwrap();
        let mut checker = BatchTypeChecker::with_options_and_corsa_path(
            root,
            BatchTypeCheckerOptions {
                tsconfig_path: Some(root.join(super::super::CONFIG_PATH)),
                ..Default::default()
            },
            Some(native),
        )
        .unwrap();
        if !pinned {
            checker.preserve_native_event_navigation_for_test();
        }
        checker.set_server_count(Some(1));
        checker.set_checker_count(Some(1));
        let paths = [root.join(ROOT), root.join(ITEM), root.join(STORY)];
        checker.scan_paths(&paths).unwrap();
        let public = checker.check_project();
        packets::retain(
            &cstr!("tree-{name}-public"),
            &serde_json::json!({
                "declaredSourceConfig": root.join(super::super::CONFIG_PATH),
                "wholePublicResult": cstr!("{public:#?}"),
                "editorDocumentOptions": if pinned { serde_json::json!(null) } else {
                    serde_json::json!({"preserveEventNavigation":true,"remainingOptions":"original editor factory defaults"})
                },
                "scope":"same complete authored modules; explicitly selected core config is distinct from original CLI selection",
            }),
        );
        public.expect("actual public Tree check must complete");
        let project_paths = checker.native_project_paths_for_test();
        let files = checker.virtual_files();
        let file = files
            .iter()
            .find(|file| file.original_path == root.join(STORY))
            .unwrap();
        let (after, after_packet) = packets::diagnose_selected(
            root,
            native,
            &files,
            name,
            "production-after",
            None,
            (&project_paths.0, &project_paths.1),
            &paths,
        );
        let before_code = replay::without_event_spreads(file, spread, !pinned, name);
        fs::write(&file.virtual_path, before_code.as_str()).unwrap();
        let (before, before_packet) = packets::diagnose_selected(
            root,
            native,
            &files,
            name,
            "prior-argument-omission-replay",
            Some(packets::Replay {
                path: &file.virtual_path,
                code: &before_code,
                scope: "equal-width erasure of synthetic event-inference spread copies only; diagnostic expression copies require unchanged byte-identical ALL-owned owners; enclosing diagnostic tag anchors require retained ALL tag owners plus complete duplicated-expression coverage; original failed App packet is separately retained; not an old factory/native execution",
            }),
            (&project_paths.0, &project_paths.1),
            &paths,
        );
        fs::write(&file.virtual_path, file.content.as_str()).unwrap();
        verdicts.push((
            name,
            story,
            wrong,
            spread,
            before,
            before_packet,
            after,
            after_packet,
        ));
    }
    // Every case retains all raw/native/source/config/map packets before a verdict.
    for (name, story, wrong, spread, before, before_packet, after, after_packet) in verdicts {
        let offset = story.find("@toggle=").unwrap() + 1;
        let (line, column) =
            crate::batch::source_map::offset_to_line_col(&story, offset as u32).unwrap();
        let handler = |row: &&packets::Row| {
            row.0 == STORY
                && row.1 == line
                && row.2 == column
                && row.3 == Some(2322)
                && row.5.contains("ToggleEvent")
        };
        let before_other: Vec<_> = before.iter().filter(|row| !handler(row)).cloned().collect();
        let after_other: Vec<_> = after.iter().filter(|row| !handler(row)).cloned().collect();
        assert_eq!(after_other, before_other, "{name}: {after_packet}");
        assert_eq!(
            after.iter().filter(handler).count(),
            usize::from(wrong),
            "{after_packet}"
        );
        assert_eq!(
            before.iter().filter(handler).count(),
            usize::from(spread || wrong),
            "{before_packet}"
        );
        if !pinned {
            assert_eq!(after.len(), usize::from(wrong), "{after_packet}");
            assert_eq!(
                before.len(),
                usize::from(spread || wrong),
                "{before_packet}"
            );
        }
        if !spread {
            assert_eq!(before, after, "{before_packet}");
        }
    }
}
