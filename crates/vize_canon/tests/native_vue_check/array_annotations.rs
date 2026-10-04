//! Complete script diagnostics remain callable without array template exposure.
use super::{Allocator, Span, block_on, configured, cstr, lower_sfc_native, options};
use vize_canon::NativeVueError;
use vize_l0::line_index::LineBreaks;
use vize_l4::targets::ts::vue::VueProjectionError;

#[test]
fn original_array_vue_keeps_script_and_literal_checks_but_refuses_bound_templates() {
    let script = "/*😀*/ const items:{id:number}[]=[{id:\"bad\"}];items[0].id;";
    let primary: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../davinci/vize_l4/tests/fixtures/array-program-checker.json"
    ))
    .unwrap();
    let primary = &primary["cases"][3]["diagnostics"][0];
    let at = |diagnostic: &serde_json::Value| {
        let start = diagnostic["start"].as_u64().unwrap() as u32;
        let length = diagnostic["length"].as_u64().unwrap() as u32;
        let byte = vize_l0::line_index::utf16_offset(script, start).unwrap();
        let end = vize_l0::line_index::utf16_offset(script, start + length).unwrap();
        let (sl, sc) = LineBreaks::Lsp.offset_to_position(script, byte);
        let (el, ec) = LineBreaks::Lsp.offset_to_position(script, end);
        (
            byte,
            end,
            serde_json::json!({"start":{"line":sl,"character":sc},"end":{"line":el,"character":ec}}),
        )
    };
    let root = tempfile::TempDir::new().unwrap();
    let bridge = configured(root.path());
    block_on(bridge.spawn()).unwrap();
    for template in [
        "",
        "<template>{{1}}</template>",
        "<template>{{items[0].id}}</template>",
    ] {
        let arena = Allocator::default();
        let source = cstr!("{template}<script setup lang=ts>{script}</script>");
        let path = root.path().join("Original.vue");
        std::fs::write(&path, source.as_bytes()).unwrap();
        let observed = lower_sfc_native(&arena, &source, options());
        assert!(observed.admitted().is_some(), "{:?}", observed.issues());
        let checked = block_on(bridge.check_native_vue(observed.admitted().unwrap(), &path));
        if template.contains("items") {
            assert!(matches!(
                checked,
                Err(NativeVueError::Projection(
                    VueProjectionError::UnsupportedTemplateBindings
                ))
            ));
        } else {
            let checked = checked.unwrap();
            assert!(core::ptr::eq(
                checked.projection().original().observation(),
                &observed
            ));
            assert!(core::ptr::eq(
                checked.projection().file(),
                observed.file().unwrap().file()
            ));
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
            let (start, end, range) = at(primary);
            let (_, _, related_range) = at(&primary["related"][0]);
            assert_eq!(
                serde_json::to_value(checked.report()).unwrap(),
                serde_json::json!({
                    "kind":"full","items":[{
                        "range":range,"severity":1,"code":2322,"source":"ts","message":primary["message"],
                        "relatedInformation":[{
                            "location":{"uri":checked.projection_uri(),"range":related_range},
                            "message":primary["related"][0]["message"],
                        }],
                    }]
                })
            );
            let offset = source.find(script).unwrap();
            assert_eq!(
                checked.authored_spans(),
                [Ok(Span::new(
                    (offset + start) as u32,
                    (offset + end) as u32
                ))]
            );
            for state in [
                checked.diagnosing_configuration().before(),
                checked.diagnosing_configuration().after(),
            ] {
                assert_eq!(
                    state.project().compiler_options,
                    checked.configuration().options
                );
            }
        }
        assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
        assert!(!root.path().join("Original.vue.ts").exists());
    }
    block_on(bridge.shutdown()).unwrap();
}
