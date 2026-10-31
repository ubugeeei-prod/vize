use super::{cache_aliases, fragment, reference};
use crate::virtual_ts::{VirtualTsOptions, generate_virtual_ts_with_offsets};
use vize_carton::{CompactString, FxHashSet, String, cstr};
use vize_croquis::{Analyzer, AnalyzerOptions, Croquis};

fn summary(script: &str, template: &str) -> Croquis {
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse_with_options(
        &allocator,
        template,
        vize_relief::options::ParserOptions {
            is_native_tag: Some(vize_carton::is_native_tag),
            ..Default::default()
        },
    );
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    analyzer.finish()
}

fn generate(script: &str, template: &str, options: &VirtualTsOptions) -> String {
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse_with_options(
        &allocator,
        template,
        vize_relief::options::ParserOptions {
            is_native_tag: Some(vize_carton::is_native_tag),
            ..Default::default()
        },
    );
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    generate_virtual_ts_with_offsets(&analyzer.finish(), Some(script), Some(&root), 0, 0, options)
        .code
}

fn registry_declarations(code: &str) -> Vec<&str> {
    code.lines()
        .filter_map(|line| {
            let (name, value) = line.strip_prefix("declare const ")?.split_once(':')?;
            value.contains("$GlobalComponent").then_some(name)
        })
        .collect()
}

#[test]
fn escaping_is_injective_for_authored_encoded_text_and_unicode() {
    let names = ["a-b", "a_b", "a_x2d_b", "a.b", "a:b", "雪-花", "雪_花"];
    let actual: FxHashSet<_> = names.iter().map(|name| fragment(name)).collect();
    assert_eq!(actual.len(), names.len());
    assert_eq!(fragment("a-b"), "a_x2d_b");
    assert_eq!(fragment("a_b"), "a_x5f_b");
    assert_eq!(fragment("a_x2d_b"), "a_x5f_x2d_x5f_b");
}

#[test]
fn valid_distinct_registry_tags_keep_two_providers_in_both_source_orders() {
    for (script, tags, expected) in [
        ("", ["a-b", "a_b"], ["$Component_a_x2d_b", "a_b"]),
        ("", ["A-b", "A_b"], ["$Component_A_x2d_b", "A_b"]),
        (
            "import type { AB, A_b } from './component';",
            ["A-b", "A_b"],
            ["$Component_A_x2d_b", "$Component_A_x5f_b"],
        ),
    ] {
        for order in [tags, [tags[1], tags[0]]] {
            let template = cstr!(
                "<{} v-slot=\"{{ item }}\">{{{{ item }}}}</{}><{} v-slot=\"{{ item }}\">{{{{ item }}}}</{}>",
                order[0],
                order[0],
                order[1],
                order[1]
            );
            let code = generate(script, template.as_str(), &Default::default());
            let mut declarations = registry_declarations(code.as_str());
            declarations.sort_unstable();
            let mut wanted = expected.to_vec();
            wanted.sort_unstable();
            assert_eq!(
                declarations, wanted,
                "distinct actual providers for {order:?}"
            );
            for provider in expected {
                assert!(
                    code.matches(provider).count() > 1,
                    "provider declaration and references: {provider}"
                );
            }
        }
    }
}

#[test]
fn generated_names_cannot_capture_script_or_template_locals() {
    for (script, template) in [
        (
            "import type { AB } from './component'; const $Component_A_b = 1; const $Component_A_x2d_b = 1; void $Component_A_b; void $Component_A_x2d_b;",
            "<A-b v-slot=\"{ item }\">{{ item }}</A-b>",
        ),
        (
            "import type { AB } from './component';",
            "<template v-for=\"$Component_A_b in [1]\"><template v-for=\"$Component_A_x2d_b in [1]\"><A-b v-slot=\"{ item }\">{{ item }}</A-b></template></template>",
        ),
    ] {
        let code = generate(script, template, &Default::default());
        assert_eq!(
            registry_declarations(code.as_str()),
            vec!["$Component_A_x2d_b__"]
        );
    }
}

#[test]
fn repeated_type_only_tags_share_the_actual_auto_import_declaration() {
    let script = "import type { AB } from './component';";
    let options = VirtualTsOptions {
        auto_import_stubs: vec![String::from(
            "declare const AB: { new(): { $slots: { default(props: { item: number }): any } } };",
        )],
        ..Default::default()
    };
    for template in [
        "<A-b v-slot=\"{ item }\">{{ item }}</A-b><AB v-slot=\"{ item }\">{{ item }}</AB>",
        "<AB v-slot=\"{ item }\">{{ item }}</AB><A-b v-slot=\"{ item }\">{{ item }}</A-b>",
    ] {
        let code = generate(script, template, &options);
        assert_eq!(code.matches("declare const $Component_AB:").count(), 1);
        assert!(!code.contains("declare const $Component_A_x2d_b:"));
        assert!(registry_declarations(code.as_str()).is_empty());
    }
}

#[test]
fn nested_alias_plans_restore_the_actual_projection_owner() {
    let outer = summary(
        "import type { AB } from './component'; const $Component_A_b = 1; const $Component_A_x2d_b = 1;",
        "<A-b />",
    );
    let inner = summary("import type { AB } from './component';", "<A-b />");
    let options = VirtualTsOptions::default();
    let type_only = FxHashSet::from_iter([CompactString::from("AB")]);
    let imported = FxHashSet::default();
    let _outer = cache_aliases(&outer, &options, &type_only, &imported);
    assert_eq!(
        reference(&outer, &options, &type_only, "A-b"),
        "$Component_A_x2d_b__"
    );
    {
        let _inner = cache_aliases(&inner, &options, &type_only, &imported);
        assert_eq!(
            reference(&inner, &options, &type_only, "A-b"),
            "$Component_A_b"
        );
    }
    assert_eq!(
        reference(&outer, &options, &type_only, "A-b"),
        "$Component_A_x2d_b__"
    );
}

#[test]
fn plain_script_import_inventory_reserves_unsafe_fallback_names() {
    let summary = summary("", "<a-b />");
    let options = VirtualTsOptions::default();
    let type_only = FxHashSet::default();
    assert!(super::needs_imported_names(&summary, &type_only));
    let imported = FxHashSet::from_iter(["a_b", "$Component_a_x2d_b"]);
    let _aliases = cache_aliases(&summary, &options, &type_only, &imported);
    assert_eq!(
        reference(&summary, &options, &type_only, "a-b"),
        "$Component_a_x2d_b__"
    );
}
