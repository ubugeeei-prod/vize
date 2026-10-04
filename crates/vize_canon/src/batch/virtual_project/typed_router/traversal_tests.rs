use super::{FILE, ROUTE, SourceType, apply, output, transform, unchanged};
use crate::virtual_ts::VirtualTsOutput;

fn language(script: &str, source_type: SourceType) -> (VirtualTsOutput, bool) {
    let mut result = output(script);
    let imported = apply(
        &mut result,
        script,
        |at| at,
        0..script.len(),
        source_type,
        FILE,
    );
    (result, imported)
}

#[test]
fn matched_typescript_define_page_stops_at_its_argument() {
    for import in [
        "",
        "import { definePage } from 'vue-router'; ",
        "import { definePage } from 'vue-router/auto'; ",
    ] {
        let script = format!("{import}definePage({{ name: useRoute().missing }});");
        let (result, imported) = transform(&script, false);
        assert!(!imported);
        assert_eq!(
            result.code,
            script.replace("definePage(", &format!("definePage<{FILE}>("))
        );
        let source = script.find("useRoute().missing").unwrap();
        let generated = result.code.find("useRoute().missing").unwrap();
        assert_eq!(result.mapping.to_generated(source), Some(generated));
        assert_eq!(result.mapping.to_authored(generated), Some(source));
    }
}

#[test]
fn javascript_define_page_keeps_macro_but_descends_to_nested_use_route() {
    for import in [
        "",
        "import { definePage } from 'vue-router'; ",
        "import { definePage } from 'vue-router/auto'; ",
    ] {
        let script = format!("{import}definePage({{ name: useRoute().missing }});");
        let (result, imported) = transform(&script, true);
        assert!(imported);
        assert_eq!(
            result.code,
            script.replace(
                "useRoute()",
                &format!("(useRoute() as ReturnType<typeof useRoute{ROUTE}>)")
            )
        );
        assert!(!result.code.contains(&format!("definePage<{FILE}>")));
        let source = script.find("missing").unwrap();
        assert_eq!(
            result.mapping.to_generated(source),
            result.code.find("missing")
        );
    }
}

#[test]
fn explicit_generic_multiple_arguments_and_non_authorized_macros_still_descend() {
    for script in [
        "definePage<string>({ name: useRoute().name });",
        "definePage({}, { name: useRoute().name });",
        "router.definePage({ name: useRoute().name });",
        "import { definePage } from './custom'; definePage({ name: useRoute().name });",
        "import { definePage as custom } from 'vue-router'; custom({ name: useRoute().name });",
        "function definePage(value: unknown) {} definePage({ name: useRoute().name });",
    ] {
        let (result, imported) = transform(script, false);
        assert!(imported);
        assert_eq!(
            result.code,
            script.replace("useRoute()", &format!("useRoute{ROUTE}()"))
        );
        assert!(!result.code.contains(&format!("definePage<{FILE}>")));
    }
}

#[test]
fn single_spread_tuple_gets_file_generic_with_unchanged_argument_mapping() {
    let script = "const tuple = [{ name: 'page' }] as const; definePage(...tuple);";
    let (result, imported) = transform(script, false);
    assert!(!imported);
    assert_eq!(
        result.code,
        script.replace("definePage(", &format!("definePage<{FILE}>("))
    );
    let source = script.find("...tuple").unwrap();
    let generated = result.code.find("...tuple").unwrap();
    assert_eq!(result.mapping.to_generated(source), Some(generated));
    assert_eq!(
        result
            .mapping
            .diagnostic_range_to_authored(generated, generated + 8),
        Some((source, source + 8))
    );
}

#[test]
fn matched_macro_does_not_descend_into_argument_type_queries_either() {
    let script = "definePage({ check: () => { type R = typeof useRoute; return useRoute(); } });";
    let (result, imported) = transform(script, false);
    assert!(!imported);
    assert_eq!(
        result.code,
        script.replace("definePage(", &format!("definePage<{FILE}>("))
    );
}

#[test]
fn typescript_jsx_keeps_type_query_specialization_and_javascript_jsx_keeps_casts() {
    let tsx =
        "type Route = ReturnType<typeof useRoute>; const view = <div>{useRoute().name}</div>;";
    let (result, imported) = language(tsx, SourceType::tsx());
    assert!(imported);
    assert!(result.code.contains(&format!("typeof useRoute{ROUTE}>")));
    assert!(
        result
            .code
            .contains(&format!("<div>{{useRoute{ROUTE}().name}}</div>"))
    );
    let jsx = "const view = <div>{useRoute().name}</div>;";
    let (result, imported) = language(jsx, SourceType::jsx());
    assert!(imported);
    assert!(result.code.contains(&format!(
        "(useRoute() as ReturnType<typeof useRoute{ROUTE}>).name"
    )));
    let runtime = "const fn = typeof useRoute; const view = <div/>;";
    let (result, imported) = language(runtime, SourceType::jsx());
    assert!(!imported);
    assert_eq!(result.code, runtime);
    assert_eq!(result.mapping, output(runtime).mapping);
}

#[test]
fn valid_typescript_angle_assertion_and_generic_arrow_do_not_block_route_edits() {
    let script = "const value = <number>1; const id = <T>(value: T) => value; const r = useRoute(); const bad = r.missing;";
    let (result, imported) = transform(script, false);
    assert!(imported);
    assert_eq!(
        result.code,
        script.replace("useRoute()", &format!("useRoute{ROUTE}()"))
    );
    let source = script.find("bad").unwrap();
    assert_eq!(result.mapping.to_generated(source), result.code.find("bad"));
    assert_eq!(
        result.mapping.to_authored(result.code.find("bad").unwrap()),
        Some(source)
    );
}

#[test]
fn plain_ts_and_js_do_not_implicitly_accept_jsx_syntax() {
    let script = "const view = <div>{useRoute().name}</div>;";
    for source_type in [SourceType::ts(), SourceType::mjs()] {
        let (result, imported) = language(script, source_type);
        assert!(!imported);
        assert_eq!(result.code, script);
        assert_eq!(result.mapping, output(script).mapping);
    }
}

#[test]
fn invalid_parse_and_non_route_scripts_keep_entire_output() {
    for script in [
        "const x = useRoute(;",
        "const x = 'useRoute()'; // definePage({})",
        "const useRoute = ;",
    ] {
        unchanged(script, false);
    }
}

fn token_bytes_stay_mapped(result: &VirtualTsOutput, script: &str, token: &str) {
    let source = script.rfind(token).unwrap();
    let generated = result.code.rfind(token).unwrap();
    for byte in 0..token.len() {
        assert_eq!(
            result.mapping.to_generated(source + byte),
            Some(generated + byte)
        );
        assert_eq!(
            result.mapping.to_authored(generated + byte),
            Some(source + byte)
        );
    }
}

#[test]
fn entirely_escaped_ambient_route_names_use_decoded_ast_authority_in_ts_and_js() {
    for name in [
        r"useR\u006fute",
        r"useR\u{006f}ute",
        r"\u0075\u0073\u0065\u0052\u006f\u0075\u0074\u0065",
    ] {
        let script = format!("const r = {name}(); const bad = r.missing;");
        assert!(!script.contains("useRoute"));
        for js in [false, true] {
            let (result, imported) = transform(&script, js);
            assert!(imported);
            let replacement = if js {
                format!("({name}() as ReturnType<typeof useRoute{ROUTE}>)")
            } else {
                format!("{name}{ROUTE}()")
            };
            assert_eq!(
                result.code,
                script.replace(&format!("{name}()"), &replacement)
            );
            token_bytes_stay_mapped(&result, &script, name);
            token_bytes_stay_mapped(&result, &script, "r.missing");
        }
    }
}

#[test]
fn escaped_direct_import_and_call_share_the_decoded_router_symbol() {
    let name = r"useR\u006fute";
    for module in ["vue-router", "vue-router/auto"] {
        let script = format!(
            "import {{ {name} }} from '{module}'; const r = {name}(); const bad = r.missing;"
        );
        assert!(!script.contains("useRoute"));
        let (result, imported) = transform(&script, false);
        assert!(imported);
        assert_eq!(
            result.code,
            script.replace(&format!("{name}()"), &format!("{name}{ROUTE}()"))
        );
        token_bytes_stay_mapped(&result, &script, name);
        token_bytes_stay_mapped(&result, &script, "bad");
    }
}

#[test]
fn escaped_only_type_query_is_specialized_and_keeps_its_authored_tokens() {
    let name = r"useR\u006fute";
    let script = format!("type Route = ReturnType<typeof {name}>; const bad = 1;");
    assert!(!script.contains("useRoute"));
    let (result, imported) = transform(&script, false);
    assert!(imported);
    assert_eq!(result.code, script.replace(name, &format!("{name}{ROUTE}")));
    token_bytes_stay_mapped(&result, &script, name);
    token_bytes_stay_mapped(&result, &script, "bad");
}

#[test]
fn escaped_only_define_page_uses_file_generic_in_ts_and_stays_unchanged_in_js() {
    let name = r"defineP\u0061ge";
    for import in [
        "".to_owned(),
        format!("import {{ {name} }} from 'vue-router'; "),
    ] {
        let script = format!("{import}{name}({{ name: 'page' }}); const bad = 1;");
        assert!(!script.contains("definePage"));
        let (result, imported) = transform(&script, false);
        assert!(!imported);
        assert_eq!(
            result.code,
            script.replace(&format!("{name}("), &format!("{name}<{FILE}>("))
        );
        token_bytes_stay_mapped(&result, &script, name);
        token_bytes_stay_mapped(&result, &script, "bad");
        unchanged(&script, true);
    }
}

#[test]
fn escaped_local_shadow_alias_and_non_router_import_remain_unchanged() {
    for script in [
        r"const useR\u006fute = () => 1; useR\u006fute();",
        r"function f(useR\u006fute: () => number) { return useR\u006fute(); }",
        r"function defineP\u0061ge(value: unknown) {} defineP\u0061ge({});",
        r"import { useRoute as useR\u006fute } from 'vue-router'; useR\u006fute();",
        r"import { useR\u006fute } from './custom'; useR\u006fute();",
    ] {
        unchanged(script, false);
    }
}

#[test]
fn escaped_explicit_and_property_shapes_keep_original_output_and_mapping() {
    unchanged(
        r"useR\u006fute('name'); useR\u006fute<string>(); router.useR\u006fute(); type R = typeof router.useR\u006fute; type E = typeof useR\u006fute<string>; defineP\u0061ge<{}>({});",
        false,
    );
}
