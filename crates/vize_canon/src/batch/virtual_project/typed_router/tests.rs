use super::{SourceType, apply};
use crate::virtual_ts::{
    ProjectionMapping, ProjectionMeta, ProjectionSpanKind, VirtualTsOutput, VizeMapping,
    VizeSemanticLink, VizeSemanticLinkKind, VizeSubSpan,
};
#[path = "traversal_tests.rs"]
mod traversal_tests;
const FILE: &str = "\"/src/pages/[id].vue\"";
const ROUTE: &str =
    "<import('vue-router/auto-routes')._RouteNamesForFilePath<\"/src/pages/[id].vue\">>";
fn output(script: &str) -> VirtualTsOutput {
    let mut mapping = ProjectionMapping::new();
    mapping.push_with(
        VizeMapping::new(0..script.len(), 0..script.len()),
        ProjectionMeta::of_kind(ProjectionSpanKind::Script),
    );
    VirtualTsOutput {
        code: script.into(),
        mapping,
    }
}
fn transform(script: &str, javascript: bool) -> (VirtualTsOutput, bool) {
    let mut result = output(script);
    let imported = rewrite(&mut result, script, javascript);
    (result, imported)
}

fn rewrite(result: &mut VirtualTsOutput, script: &str, js: bool) -> bool {
    let language = if js {
        SourceType::mjs()
    } else {
        SourceType::ts()
    };
    apply(result, script, |at| at, 0..script.len(), language, FILE)
}

fn unchanged(script: &str, javascript: bool) {
    let original = output(script);
    let (result, imported) = transform(script, javascript);
    assert!(!imported);
    assert_eq!(result.code, original.code);
    assert_eq!(result.mapping, original.mapping);
}

#[test]
fn direct_import_multiple_calls_keep_crlf_unicode_and_following_diagnostic_anchors() {
    let script = "import { useRoute } from 'vue-router';\r\nconst 日本 = '😀';\r\nconst a = useRoute /* a */ (); const b = useRoute(); const bad = 日本.missing;";
    let (result, imported) = transform(script, false);
    assert!(imported);
    assert_eq!(
        result.code,
        script
            .replace("useRoute /*", &format!("useRoute{ROUTE} /*"))
            .replace("useRoute()", &format!("useRoute{ROUTE}()"))
    );
    for word in ["日本.missing", "bad", "const b"] {
        let source = script.find(word).unwrap();
        let generated = result.code.find(word).unwrap();
        assert_eq!(result.mapping.to_generated(source), Some(generated));
        assert_eq!(result.mapping.to_authored(generated), Some(source));
        assert_eq!(
            result
                .mapping
                .diagnostic_range_to_authored(generated, generated + word.len()),
            Some((source, source + word.len()))
        );
    }
    assert!(
        result
            .mapping
            .rows()
            .all(|row| row.meta.kind == ProjectionSpanKind::Script)
    );
}

#[test]
fn auto_imports_remain_distinct_from_nested_parameter_and_block_shadows() {
    let script = "const a = useRoute(); function f(useRoute: () => number) { return useRoute(); } { const useRoute = () => 1; useRoute(); } const b = (() => useRoute())();";
    let (result, imported) = transform(script, false);
    assert!(imported);
    assert_eq!(result.code.matches(ROUTE).count(), 2);
    assert!(result.code.contains("return useRoute();"));
    assert!(result.code.contains("() => 1; useRoute();"));
}

#[test]
fn bound_locals_aliases_other_modules_and_type_only_imports_are_unchanged() {
    for script in [
        "useRoute(); const useRoute = () => 1;",
        "function useRoute() {} useRoute();",
        "import { useRoute } from './custom'; useRoute();",
        "import { other as useRoute } from 'vue-router'; useRoute();",
        "import { useRoute as useRoute } from 'vue-router'; useRoute();",
        "import { useRoute as route } from 'vue-router'; route();",
        "import useRoute from 'vue-router'; useRoute();",
        "import * as useRoute from 'vue-router'; useRoute();",
        "import type { useRoute } from 'vue-router'; type R = typeof useRoute;",
        "import { type useRoute } from 'vue-router'; type R = typeof useRoute;",
        "import { definePage } from './custom'; definePage({});",
        "function definePage(value: unknown) {} definePage({});",
    ] {
        unchanged(script, false);
    }
}

#[test]
fn explicit_arguments_generics_property_and_optional_calls_are_unchanged() {
    unchanged(
        "useRoute('name'); useRoute<string>(); router.useRoute(); useRoute?.(); type R = typeof router.useRoute; type E = typeof useRoute<string>; definePage(); definePage({}, {}); definePage<string>({});",
        false,
    );
}

#[test]
fn typeof_queries_follow_the_same_resolved_value_symbol() {
    let script = "import { useRoute } from 'vue-router'; type R = ReturnType<typeof useRoute>; function f(useRoute: () => number) { type Local = typeof useRoute; }";
    let (result, imported) = transform(script, false);
    assert!(imported);
    assert!(
        result
            .code
            .contains(&format!("ReturnType<typeof useRoute{ROUTE}>"))
    );
    assert!(result.code.contains("type Local = typeof useRoute;"));
}

#[test]
fn define_page_uses_only_file_literal_and_does_not_report_route_import_need() {
    for module in [None, Some("vue-router"), Some("vue-router/auto")] {
        let script = module.map_or_else(
            || "definePage({ name: 'page' });".to_owned(),
            |module| {
                format!("import {{ definePage }} from '{module}'; definePage({{ name: 'page' }});")
            },
        );
        unchanged(&script, true);
        let (result, imported) = transform(&script, false);
        assert!(!imported);
        assert!(
            result
                .code
                .contains(&format!("definePage<{FILE}>({{ name: 'page' }})"))
        );
        assert!(!result.code.contains("auto-routes"));
    }
}

#[test]
fn router_auto_value_import_is_authorized() {
    let (result, imported) = transform(
        "import { useRoute } from 'vue-router/auto'; useRoute();",
        false,
    );
    assert!(imported);
    assert!(result.code.contains(&format!("useRoute{ROUTE}()")));
}

#[test]
fn javascript_wrapper_preserves_call_bytes_and_following_source_mappings() {
    let script = "const a = useRoute /*😀*/ (); const b = useRoute(); const bad = b.missing;";
    let (result, imported) = transform(script, true);
    assert!(imported);
    assert!(result.code.contains(&format!(
        "(useRoute /*😀*/ () as ReturnType<typeof useRoute{ROUTE}>)"
    )));
    for word in ["useRoute /*", "useRoute()", "bad", "b.missing"] {
        let source = script.find(word).unwrap();
        let generated = result.code.find(word).unwrap();
        assert_eq!(result.mapping.to_generated(source), Some(generated));
        assert_eq!(result.mapping.to_authored(generated), Some(source));
    }
}

#[test]
fn physical_setup_gates_split_blocks_with_out_of_line_generated_order() {
    let plain = "import { useRoute } from 'vue-router'; const plain = useRoute();";
    let setup = "const route = useRoute(); const bad = route.missing;";
    let script = format!("{plain}\n{setup}");
    let split = plain.len() + 1;
    let prefix = "/* generated wrapper */\n";
    let code = format!("{prefix}{setup}\n{plain}");
    let mut mapping = ProjectionMapping::new();
    mapping.push(VizeMapping::new(
        prefix.len()..prefix.len() + setup.len(),
        200..200 + setup.len(),
    ));
    mapping.push(VizeMapping::new(
        prefix.len() + setup.len() + 1..code.len(),
        20..20 + plain.len(),
    ));
    let mut result = VirtualTsOutput {
        code: code.into(),
        mapping,
    };
    assert!(apply(
        &mut result,
        &script,
        |at| if at < split {
            at + 20
        } else {
            at - split + 200
        },
        200..200 + setup.len(),
        SourceType::ts(),
        FILE
    ));
    assert!(result.code.ends_with(plain));
    assert!(
        result
            .code
            .contains(&format!("const route = useRoute{ROUTE}();"))
    );
    let authored = 200 + setup.find("bad").unwrap();
    let generated = result.code.find("bad").unwrap();
    assert_eq!(result.mapping.to_generated(authored), Some(generated));
    assert_eq!(result.mapping.to_authored(generated), Some(authored));
}

#[test]
fn authored_base_and_semantic_link_endpoints_keep_their_original_tokens() {
    let script = "useRoute(); const after = 1;";
    let after = script.find("after").unwrap();
    let link = VizeSemanticLink {
        source_range: 0..8,
        target_range: after..after + 5,
        kind: VizeSemanticLinkKind::VueSetupImportSpecialization,
    };
    let mut result = VirtualTsOutput {
        code: script.into(),
        mapping: ProjectionMapping::from_parts(
            vec![VizeMapping::new(0..script.len(), 0..script.len())],
            vec![link.clone()],
        ),
    };
    result.mapping.set_authored_base(50);
    assert!(apply(
        &mut result,
        script,
        |at| at + 50,
        50..50 + script.len(),
        SourceType::ts(),
        FILE
    ));
    let actual = &result.mapping.semantic_links()[0];
    assert_eq!(actual.source_range, link.source_range);
    assert_eq!(
        actual.target_range,
        after + ROUTE.len()..after + ROUTE.len() + 5
    );
    assert_eq!(actual.kind, link.kind);
    assert_eq!(
        result.mapping.to_authored(after + ROUTE.len()),
        Some(after + 50)
    );
}

#[test]
fn exact_subspan_at_row_boundary_is_preserved() {
    let script = "useRoute();";
    let mut token = VizeMapping::new(0..8, 0..8);
    token.sub_spans.push(VizeSubSpan {
        gen_range: 0..8,
        src_range: 0..8,
    });
    let mut result = output(script);
    result.mapping = ProjectionMapping::from_spans(vec![
        token.clone(),
        VizeMapping::new(8..script.len(), 8..script.len()),
    ]);
    assert!(rewrite(&mut result, script, false));
    assert_eq!(result.mapping.spans()[0], token);
    assert_eq!(result.mapping.to_generated(8), Some(8 + ROUTE.len()));
}

#[test]
fn missing_ambiguous_nonlinear_or_interior_subspan_mapping_declines_edits() {
    let script = "useRoute();";
    let mut sub = VizeMapping::new(0..script.len(), 0..script.len());
    sub.sub_spans.push(VizeSubSpan {
        gen_range: 0..8,
        src_range: 0..8,
    });
    for spans in [
        vec![],
        vec![VizeMapping::new(0..script.len(), 0..script.len() + 1)],
        vec![
            VizeMapping::new(0..script.len(), 0..script.len()),
            VizeMapping::new(script.len()..script.len() * 2, 0..script.len()),
        ],
        vec![sub],
    ] {
        let mut result = VirtualTsOutput {
            code: format!("{script}{script}").into(),
            mapping: ProjectionMapping::from_spans(spans),
        };
        let original_code = result.code.clone();
        let original_mapping = result.mapping.clone();
        assert!(!rewrite(&mut result, script, false));
        assert_eq!(result.code, original_code);
        assert_eq!(result.mapping, original_mapping);
    }
}

#[test]
fn mismatching_generated_bytes_and_discontinuous_physical_sites_are_unchanged() {
    let script = "useRoute();";
    for discontinuous in [false, true] {
        let mut result = output(script);
        if !discontinuous {
            result.code = "useOther();".into();
        }
        let original_code = result.code.clone();
        let original_mapping = result.mapping.clone();
        assert!(!apply(
            &mut result,
            script,
            |at| if discontinuous && at >= 8 { at + 1 } else { at },
            0..script.len() + 1,
            SourceType::ts(),
            FILE
        ));
        assert_eq!(result.code, original_code);
        assert_eq!(result.mapping, original_mapping);
    }
}

#[test]
fn empty_setup_invalid_literal_and_repeat_application_do_not_mutate() {
    let script = "useRoute();";
    for (setup, literal) in [(0..0, FILE), (0..script.len(), "not a literal")] {
        let mut result = output(script);
        let original_mapping = result.mapping.clone();
        assert!(!apply(
            &mut result,
            script,
            |at| at,
            setup,
            SourceType::ts(),
            literal
        ));
        assert_eq!(result.code, script);
        assert_eq!(result.mapping, original_mapping);
    }
    let (mut result, _) = transform(script, false);
    let code = result.code.clone();
    let mapping = result.mapping.clone();
    assert!(!rewrite(&mut result, script, false));
    assert_eq!(result.code, code);
    assert_eq!(result.mapping, mapping);
}
