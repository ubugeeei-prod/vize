//! Whole authored parse-failure packets keep descriptor roles and attributes.
#![expect(clippy::unwrap_used, reason = "complete authored fixture laws")]
#![expect(
    clippy::disallowed_types,
    reason = "fixture strings retain exact bytes"
)]

use super::{GeneratedVueFile, VueCodegenOptions, generate_vue_virtual_ts};
use crate::virtual_ts::{VirtualTsGenerationOptions, VirtualTsOptions};
use std::path::Path;
use vize_atelier_sfc::{SfcParseOptions, croquis::SfcCroquisOptions, parse_sfc};
use vize_croquis::ScopeKind;

mod parser_authority;
use parser_authority::packet;

#[cfg(unix)]
mod native;

const WORKSPACE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/vue-language-tools/upstream/test-workspace"
);
const CORPUS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/malformed-setup-metadata"
);

fn options() -> VueCodegenOptions<'static> {
    VueCodegenOptions {
        check_options: Default::default(),
        typed_router_root: None,
        preserve_unused_diagnostics: false,
        options_api: false,
        preserve_authored_component: false,
        preserve_script_on_template_error: true,
        component_name: None,
        external_component_attrs_module: None,
        preserve_event_navigation: false,
        legacy_vue2: false,
        dialect: Default::default(),
        template_syntax: Default::default(),
        experimental_in_tag_comments: false,
        experimental_patterned_template: false,
        experimental_strict_slot_children: false,
        hoist_shared_preamble: false,
        inline_auto_import_stubs: false,
        omit_vite_client_reference: false,
        runtime_prop_resolve_cache: None,
    }
}

fn project(source: &str) -> (GeneratedVueFile, GeneratedVueFile, serde_json::Value) {
    let path = Path::new("App.vue");
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    let analysis = vize_atelier_sfc::croquis::analyze_sfc_descriptor_with_context(
        &descriptor,
        None,
        SfcCroquisOptions::full(),
    );
    let script = analysis.script_content.as_deref();
    let retained =
        super::super::diagnostics::collect_script_parse_fallbacks(path, source, &descriptor, true);
    // The same descriptor analysis and exact original generation flags are
    // the before path. No recovered AST or hand-built summary stands in for it.
    let prior = crate::virtual_ts::generate_virtual_ts_with_offsets_and_checks(
        &analysis.croquis,
        script,
        None,
        analysis.script_offset,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            split_script_setup_offsets: analysis.split_script_setup_offsets(&descriptor),
            script_syntax_boundaries: &retained.boundaries,
            ..Default::default()
        },
    );
    let (mut mappings, mut mapping_meta, semantic_links, prop_default_key_links) =
        prior.mapping.into_binding_parts_with_metadata();
    // The generic attribute post-pass reads only the generated text.
    super::generic_attribute::map_generic_attribute(
        &prior.code,
        &mut mappings,
        &mut mapping_meta,
        source,
        &descriptor,
    );
    let before = GeneratedVueFile {
        code: prior.code,
        mappings,
        mapping_meta,
        semantic_links,
        prop_default_key_links,
        diagnostics: retained.diagnostics,
        typed_router_import: false,
        empty_module_exports: false,
        module_exports: None,
    };
    let after = generate_vue_virtual_ts(
        path,
        source,
        &descriptor,
        &VirtualTsOptions::default(),
        options(),
    )
    .unwrap();
    let packet = serde_json::json!({
        "wholeAuthoredSource":source,
        "wholeDescriptor":vize_carton::cstr!("{descriptor:#?}"),
        "wholeCroquis":vize_carton::cstr!("{:#?}",analysis.croquis),
        "wholeAnalyzedScript":script,
        "scriptOffset":analysis.script_offset,
        "splitOffsets":analysis.split_script_setup_offsets(&descriptor),
        "analyzedSetup":analysis.croquis.scopes.iter().any(|scope| matches!(scope.kind,ScopeKind::ScriptSetup)),
        "before":packet(&before), "after":packet(&after),
    });
    (before, after, packet)
}

fn sources(issue: &str) -> Vec<(&'static str, std::string::String)> {
    let names: &[&str] = if issue == "5823" {
        &["main.vue"]
    } else {
        &["withScript.vue", "withoutScript.vue"]
    };
    names
        .iter()
        .map(|name| {
            let source = std::fs::read_to_string(
                Path::new(WORKSPACE)
                    .join("tsc")
                    .join(vize_carton::cstr!("_failed_#{issue}").as_str())
                    .join(name),
            )
            .unwrap();
            assert_eq!(
                source,
                std::fs::read_to_string(
                    Path::new(CORPUS).join(vize_carton::cstr!("{issue}-{name}.txt").as_str())
                )
                .unwrap()
            );
            (*name, source)
        })
        .collect()
}

fn assert_authored_maps(source: &str, output: &GeneratedVueFile) {
    assert!(!output.mappings.is_empty());
    for mapping in &output.mappings {
        assert!(source.get(mapping.src_range.clone()).is_some());
        assert!(output.code.get(mapping.gen_range.clone()).is_some());
    }
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    let setup = descriptor.script_setup.as_ref().unwrap();
    let authored = if setup.content.contains("info?: T;") {
        "info?: T;"
    } else {
        "'vue'"
    };
    assert!(
        output.code.contains(authored),
        "authored script tokens must remain intact"
    );
    let source_start = source.find(authored).unwrap();
    assert!(output.mappings.iter().any(|mapping| {
        mapping.src_range.start <= source_start && mapping.src_range.end > source_start
    }));
}

#[test]
fn full_malformed_descriptors_keep_their_setup_role_and_generic_without_recovering_an_ast() {
    for issue in ["5071", "5823"] {
        for (name, source) in sources(issue) {
            let (before, after, packet) = project(&source);
            assert_eq!(packet["analyzedSetup"], false);
            assert_authored_maps(&source, &after);
            if issue == "5823" {
                assert!(before.code.contains("function __setup()"));
                assert!(after.code.contains("function __setup<T>()"));
                assert!(after.code.contains("info?: T;"));
            } else {
                let import = after.code.find("import { ref } from 'vue'").unwrap();
                assert!(import < after.code.find("function __setup()").unwrap());
                if name == "withScript.vue" {
                    assert_eq!(after.code.matches("const __default__").count(), 1);
                    assert!(!after.code.contains("export default{}"));
                }
                // The missing terminator still refuses the whole program.
                // The clean prefix records `import('vue')` from the generated
                // preamble. The glued authored import is not its own statement.
                for phase in ["before", "after"] {
                    assert_eq!(
                        packet[phase]["wholeOxcParser"]["panicked"], true,
                        "{name} {phase}"
                    );
                    assert_eq!(
                        packet[phase]["wholeOxcParser"]["programBodyLength"], 0,
                        "{name} {phase}"
                    );
                    assert_eq!(
                        packet[phase]["wholeActualImportRewriterSpecifiers"],
                        serde_json::json!(["vue"]),
                        "{name} {phase}"
                    );
                }
                assert_eq!(after.code.matches("import { ref } from 'vue'").count(), 1);
                assert!(after.code[import..].starts_with("import { ref } from 'vue'export {};"));
            }
            assert!(!after.code.contains("async function __setup"));
        }
    }
}

#[test]
fn successful_setup_analysis_keeps_the_complete_original_projection_and_maps() {
    for issue in ["5071", "5823"] {
        for (_, source) in sources(issue) {
            let source = source
                .replace("'vue'ref", "'vue';ref")
                .replace("const foo = ;", "const foo = 1;");
            for source in [
                source.clone(),
                source.replace("const foo = 1;", "const foo = await Promise.resolve(1);"),
            ] {
                let (before, after, packet) = project(&source);
                assert_eq!(packet["analyzedSetup"], true);
                assert_eq!(before.code, after.code);
                assert_eq!(before.mappings, after.mappings);
                assert_eq!(before.semantic_links, after.semantic_links);
                assert_eq!(before.prop_default_key_links, after.prop_default_key_links);
                assert_authored_maps(&source, &after);
                if issue == "5071" {
                    for phase in ["before", "after"] {
                        assert_eq!(packet[phase]["wholeOxcParser"]["panicked"], false);
                        assert!(
                            packet[phase]["wholeActualImportRewriterSpecifiers"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .any(|specifier| specifier == "vue")
                        );
                    }
                }
            }
        }
    }
    for source in [
        "<script lang=\"ts\">export default {};\nconst invalid = ;</script>",
        "<script lang=\"ts\">\nimport { ref } from 'vue'ref('');\nexport default {};\n</script>",
    ] {
        let (before, after, _) = project(source);
        assert_eq!(before.code, after.code);
        assert_eq!(before.mappings, after.mappings);
        assert_eq!(before.semantic_links, after.semantic_links);
    }
}

fn commented_imports() -> [&'static str; 6] {
    [
        "/*comment*/\n",
        "/*comment*/;",
        "/*\ncomment*/",
        "/*comment*/\u{2028}",
        "/*\u{2029}comment*/",
        "/*comment*/\r",
    ]
}

fn quoted_import_source() -> std::string::String {
    std::fs::read_to_string(Path::new(CORPUS).join("valid-quoted-import.vue.txt")).unwrap()
}

fn overlapping_import_source() -> std::string::String {
    std::fs::read_to_string(Path::new(CORPUS).join("overlapping-imports.vue.txt")).unwrap()
}

#[test]
fn valid_commented_imports_remain_module_declarations_when_a_later_setup_parse_panics() {
    for tail in commented_imports() {
        let source = vize_carton::cstr!(
            "<script setup lang=\"ts\">\nimport {{ ref }} from 'vue'{tail}ref('');\nconst broken = ;\nvoid broken;\n</script>\n"
        );
        if tail == commented_imports()[0] {
            assert_eq!(
                source.as_str(),
                std::fs::read_to_string(Path::new(CORPUS).join("valid-commented-import.vue.txt"))
                    .unwrap()
            );
        }
        let (_, after, packet) = project(&source);
        assert_eq!(packet["analyzedSetup"], false, "{packet}");
        assert_authored_maps(&source, &after);
        let import = after.code.find("import { ref } from 'vue'").unwrap();
        assert!(
            import < after.code.find("function __setup()").unwrap(),
            "{packet}"
        );
        assert!(!after.code.contains("'vue'export"), "{packet}");
    }
    let source = quoted_import_source();
    let (_, after, packet) = project(&source);
    assert_eq!(packet["analyzedSetup"], false, "{packet}");
    assert_authored_maps(&source, &after);
    assert!(
        after
            .code
            .find("import { 'ref' as ref } from 'vue'")
            .unwrap()
            < after.code.find("function __setup()").unwrap(),
        "{packet}"
    );
    let source = overlapping_import_source();
    let (_, after, packet) = project(&source);
    assert_eq!(packet["analyzedSetup"], false, "{packet}");
    assert_authored_maps(&source, &after);
    for import in [
        "import { ref } from 'vue'",
        "import { computed } from 'vue'",
    ] {
        assert_eq!(after.code.matches(import).count(), 1, "{packet}");
        assert!(
            after.code.find(import).unwrap() < after.code.find("function __setup()").unwrap(),
            "{packet}"
        );
    }
}
