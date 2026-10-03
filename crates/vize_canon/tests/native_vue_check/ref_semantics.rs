//! Actual Vue package types expose the difference between membership and unref.

use super::*;
use vize_canon::NativeVueError;
use vize_l4::targets::ts::vue::VueProjectionError;

#[test]
fn genuine_vue_ref_types_keep_script_diagnostics_but_refuse_unproven_template_unwrapping() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let dependencies = repo.join("docs/node_modules");
    let package: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dependencies.join("vue/package.json")).unwrap())
            .unwrap();
    assert_eq!(package["version"], "3.5.35");
    let script = "import {counter} from './counter'; let value=counter; value.toFixed;";
    for ts in [false, true] {
        let root = tempfile::TempDir::new().unwrap();
        std::os::unix::fs::symlink(&dependencies, root.path().join("node_modules")).unwrap();
        std::fs::write(
            root.path().join("counter.ts"),
            "import {ref} from 'vue'; export const counter=ref(1);",
        )
        .unwrap();
        let bridge = configured(root.path());
        block_on(bridge.spawn()).unwrap();
        let arena = Allocator::default();
        for template in [
            "",
            "<template>{{1}}</template>",
            "<template>{{value.toFixed}}</template>",
        ] {
            let source = cstr!(
                "{template}<script setup{}>{script}</script>",
                if ts { " lang=ts" } else { "" }
            );
            let path = root.path().join("Original.vue");
            std::fs::write(&path, source.as_bytes()).unwrap();
            let original = lower_sfc_native(&arena, &source, options());
            assert!(original.admitted().is_some(), "{:?}", original.issues());
            let checked = block_on(bridge.check_native_vue(original.admitted().unwrap(), &path));
            if template.contains("value") {
                assert!(matches!(
                    checked,
                    Err(NativeVueError::Projection(
                        VueProjectionError::UnsupportedTemplateBindings
                    ))
                ));
                let embed = original.template().unwrap().embeds().first().unwrap();
                let resolution = original
                    .file()
                    .unwrap()
                    .file()
                    .expression(embed.node.unwrap())
                    .unwrap();
                assert_eq!(resolution.table().unwrap().occurrences().len(), 1);
                assert!(std::ptr::eq(
                    resolution.table().unwrap().expression().ast,
                    embed.syntax.expression().unwrap()
                ));
            } else {
                let checked = checked.unwrap();
                let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(full)) =
                    checked.report()
                else {
                    panic!("complete raw report")
                };
                let at = script.find("toFixed").unwrap();
                assert_eq!(
                    serde_json::to_value(&full.full_document_diagnostic_report.items).unwrap(),
                    serde_json::json!([{"range":{"start":{"line":0,"character":at},"end":{"line":0,"character":at+7}},"severity":1,"code":2339,"source":"ts","message":"Property 'toFixed' does not exist on type 'Ref<number, number>'."}])
                );
                let expected = source.rfind("toFixed").unwrap() as u32;
                assert_eq!(
                    checked.authored_spans(),
                    [Ok(Span::new(expected, expected + 7))]
                );
                assert_eq!(
                    checked.diagnostic_configuration_path(),
                    root.path().join("tsconfig.json").canonicalize().unwrap()
                );
                assert_eq!(
                    checked.projection().document().as_str(),
                    cstr!(
                        "{script}\n;\nexport {{}};\n{}",
                        if template.is_empty() {
                            ""
                        } else {
                            "void (\n1\n);\n"
                        }
                    )
                );
                assert!(std::ptr::eq(
                    checked.projection().original().observation(),
                    &original
                ));
            }
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
        }
        block_on(bridge.shutdown()).unwrap();
    }
}
