use super::*;

#[test]
fn jsx_mode_takes_precedence_over_vapor() {
    assert_eq!(
        resolve_default_mode(Some("vapor"), Some(false)),
        JsxOutputMode::Vapor
    );
    assert_eq!(
        resolve_default_mode(Some("vdom"), Some(true)),
        JsxOutputMode::Vdom
    );
}

#[test]
fn falls_back_to_vapor_bool_then_vdom() {
    assert_eq!(resolve_default_mode(None, Some(true)), JsxOutputMode::Vapor);
    assert_eq!(resolve_default_mode(None, Some(false)), JsxOutputMode::Vdom);
    assert_eq!(resolve_default_mode(None, None), JsxOutputMode::Vdom);
    assert_eq!(
        resolve_default_mode(Some("react"), Some(true)),
        JsxOutputMode::Vapor
    );
}

#[test]
fn jsx_compile_result_surfaces_scoped_style_css() {
    let source = r#"
        const App = () => (
            <div class="box">
                <style scoped>{`.box { color: red }`}</style>
            </div>
        );
    "#;
    let result = compile_jsx_impl(source.to_string(), None);

    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(result.scoped_styles.len(), 1);
    let style = &result.scoped_styles[0];
    assert!(style.scope_id.starts_with("data-v-"), "{}", style.scope_id);
    assert!(
        style.css.contains(".box") && style.css.contains(&style.scope_id),
        "{}",
        style.css
    );
}

#[test]
fn jsx_compile_result_has_no_scoped_styles_without_style_block() {
    let source = "const App = () => <div class=\"box\">hi</div>;";
    let result = compile_jsx_impl(source.to_string(), None);

    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert!(result.scoped_styles.is_empty());
}

#[test]
fn jsx_compile_result_includes_runtime_helper_preamble() {
    let source = "const App = () => <div>{message}</div>;\nexport default App;\n";
    let result = compile_jsx_impl(source.to_string(), None);

    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert!(result.code.contains("from \"vue\""), "{}", result.code);
    let import_at = result.code.find("from \"vue\"").expect("vue import");
    let usage_at = result
        .code
        .find("_createElementBlock(")
        .expect("render uses _createElementBlock");
    assert!(import_at < usage_at, "{}", result.code);
}

#[test]
fn jsx_compile_result_wraps_block_body_setup_state() {
    let source = r#"
        import { computed, ref } from "vue";

        export const App = () => {
            const count = ref(0);
            const doubled = computed(() => count.value * 2);
            const increment = () => {
                count.value += 1;
            };

            return <button onClick={increment}>{doubled.value}</button>;
        };
    "#;
    let result = compile_jsx_impl(source.to_string(), None);

    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert!(
        result
            .code
            .contains("import { defineComponent as _defineComponent } from \"vue\""),
        "{}",
        result.code
    );
    assert!(
        result
            .code
            .contains("export const App = _defineComponent({")
    );
    assert!(result.code.contains("const count = ref(0);"));
    assert!(result.code.contains("count.value += 1;"));
    assert!(result.code.contains("function render(_ctx, _cache)"));
    assert!(!result.code.contains("export function render("));
}

#[test]
fn jsx_compile_result_surfaces_source_map_when_requested() {
    let source = "const App = () => <div>{message}</div>;\nexport default App;\n";
    let without = compile_jsx_impl(source.to_string(), None);
    assert!(without.map.is_none(), "no map unless requested");

    let with = compile_jsx_impl(
        source.to_string(),
        Some(JsxCompileOptionsNapi {
            source_map: Some(true),
            ..Default::default()
        }),
    );
    assert!(with.errors.is_empty(), "errors: {:?}", with.errors);
    let map = with.map.expect("a map is surfaced when requested");
    assert!(map.contains("\"version\":3"), "v3 source map: {map}");
}

#[test]
fn jsx_compile_result_supports_ssr_output() {
    let result = compile_jsx_impl(
        "const App = () => <div>{message}</div>;".to_string(),
        Some(JsxCompileOptionsNapi {
            ssr: Some(true),
            source_map: Some(true),
            ..Default::default()
        }),
    );

    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert!(
        result.code.contains("function ssrRender"),
        "{}",
        result.code
    );
    assert!(
        result.code.contains("@vue/server-renderer"),
        "{}",
        result.code
    );
    assert!(result.map.is_none(), "SSR output has no source map yet");
}
