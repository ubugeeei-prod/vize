use super::*;
use crate::JsxSyntaxProduct;
use vize_atelier_dom::DomOutputProduct;
use vize_atelier_ssr::SsrOutputProduct;
use vize_atelier_vapor::VaporOutputProduct;
use vize_rendu::RenduProduct;

#[test]
fn typed_compile_product_reuses_the_syntax_plan_and_cache() {
    crate::syntax::reset_frontend_counters();
    let mut compilation = Compilation::new();
    super::super::register_atlas_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source("App.jsx", "const App = () => <div>{message}</div>")
        .unwrap();
    JsxCompileSettings::default()
        .install(&mut compilation)
        .unwrap();
    let first = compilation.query::<JsxCompileProduct>(source).unwrap();
    assert!(first.plan().contains::<JsxSyntaxProduct>());
    assert!(first.plan().contains::<JsxRenderModuleProduct>());
    assert!(first.plan().contains::<DomOutputProduct>());
    assert!(!first.plan().contains::<SsrOutputProduct>());
    assert!(!first.plan().contains::<VaporOutputProduct>());
    assert!(first.value().code.contains("_createElementBlock(\"div\""));
    assert_eq!(crate::syntax::frontend_counters().2, 0);
    assert!(
        first
            .execution()
            .observations()
            .iter()
            .all(|observation| { observation.code() != "jsx.compile.legacy-root-materialization" })
    );
    let second = compilation.query::<JsxCompileProduct>(source).unwrap();
    assert_eq!(second.status(), vize_atlas::ProductStatus::CacheHit);
}

#[test]
fn graph_compile_preserves_modes_state_and_final_source_map_coordinates() {
    let source = r#"
const App = () => {
  const count = 1;
  return <section>{count}</section>;
};
"#;
    let mut config = JsxCompileConfig::default();
    config.vdom.source_map = true;
    let artifact = compile_jsx_with_atlas(source, "App.tsx", JsxLang::Tsx, config.clone()).unwrap();
    assert!(
        artifact.code.contains("_defineComponent"),
        "{}",
        artifact.code
    );
    assert!(
        artifact.code.contains("const count = 1"),
        "{}",
        artifact.code
    );
    assert!(
        artifact.map.is_none(),
        "stateful module maps are intentionally withheld"
    );
    let mapped = compile_jsx_with_atlas(
        "const Mapped = () => <section>{count}</section>",
        "Mapped.tsx",
        JsxLang::Tsx,
        config,
    )
    .unwrap();
    let map: serde_json::Value = serde_json::from_str(mapped.map.unwrap().as_str()).unwrap();
    assert_eq!(map["sources"], serde_json::json!(["Mapped.tsx"]));

    let vapor = compile_jsx_with_atlas(
        "const Fast = () => { \"use vue:vapor\"; return <main/>; }",
        "Fast.jsx",
        JsxLang::Jsx,
        JsxCompileConfig::default(),
    )
    .unwrap();
    assert!(vapor.code.contains("_template("), "{}", vapor.code);

    let ssr_config = JsxCompileConfig {
        ssr: true,
        ..Default::default()
    };
    let ssr = compile_jsx_with_atlas(
        "const Page = () => <article>{message}</article>",
        "Page.jsx",
        JsxLang::Jsx,
        ssr_config,
    )
    .unwrap();
    assert!(ssr.code.contains("ssrRender"), "{}", ssr.code);
}

#[test]
fn graph_vapor_emits_components_control_flow_slots_directives_and_scope() {
    let source = r#"
const Fast = () => {
  "use vue:vapor";
  return <Panel {...attrs} title={title} v-focus:lazy={focus}>
    {ok ? <slot name="body">fallback</slot> : <ul {...attrs} v-focus:lazy={focus}>{items.map((item, index) => <li key={index}>{item}</li>)}</ul>}
    <style scoped>{`.fast { color: red; }`}</style>
  </Panel>;
};
"#;
    let artifact = compile_jsx_with_atlas(
        source,
        "Fast.jsx",
        JsxLang::Jsx,
        JsxCompileConfig::default(),
    )
    .unwrap();
    let code = artifact.code.as_str();
    for marker in [
        "_createComponentWithFallback",
        "_setDynamicProps",
        "_withDirectives",
        "_createIf",
        "_createFor",
        "default: () =>",
        "data-v-",
    ] {
        assert!(code.contains(marker), "missing {marker}: {code}");
    }
    assert_eq!(artifact.scoped_styles.len(), 1);
    assert_eq!(crate::syntax::frontend_counters().2, 0);
}

#[test]
fn mixed_jsx_module_selects_both_client_backends_over_one_rendu_product() {
    let mut compilation = Compilation::new();
    super::super::register_atlas_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source(
            "Mixed.jsx",
            r#"const Fast = () => { "use vue:vapor"; return <main/>; };
const Stable = () => <aside/>;"#,
        )
        .unwrap();
    let mut settings = JsxCompileSettings::default();
    settings.insert(
        source,
        JsxCompileRequest::new(JsxLang::Jsx, JsxCompileConfig::default()),
    );
    settings.install(&mut compilation).unwrap();
    let output = compilation.query::<JsxCompileProduct>(source).unwrap();
    assert!(output.plan().contains::<DomOutputProduct>());
    assert!(output.plan().contains::<VaporOutputProduct>());
    assert!(!output.plan().contains::<SsrOutputProduct>());
    assert_eq!(
        compilation
            .counters()
            .for_product::<RenduProduct>()
            .executions(),
        1
    );
    assert!(output.value().code.contains("_template("));
    assert!(
        output
            .value()
            .code
            .contains("_createElementBlock(\"aside\"")
    );
}
