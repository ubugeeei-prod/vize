//! Complete current public API oracles for historical JSX/TSX compiler fixes
//! (#6880). Each case compiles one authored module through the public
//! mode-aware entry and snapshots the module, every component and every
//! diagnostic. These snapshots are legacy regression evidence; they confer no
//! native credit.

#![expect(
    clippy::panic,
    clippy::disallowed_macros,
    reason = "tests panic on invalid fixtures and snapshot the complete actual Debug bytes"
)]

use serde::Deserialize;
use vize_atelier_jsx::{
    BabelJsxCustomizations, BabelJsxOptions, JsxCompatMode, JsxCompileConfig, JsxComponent,
    JsxDiagnostic, JsxLang, JsxOutputMode, compile_jsx_with_babel_customizations,
};
use vize_l0::{Allocator, String};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    history: String,
    source: String,
    #[expect(dead_code, reason = "retained in the complete Debug oracle")]
    filename: String,
    /// `jsx` or `tsx`.
    lang: String,
    /// Default output mode: `vdom` (default), `vapor` or `ssr`.
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    options: Options,
    errors: usize,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    /// `native` (default) or `babel` (`@vue/babel-plugin-jsx` semantics).
    compat: Option<String>,
    /// Babel `pragma`.
    pragma: Option<String>,
    /// Babel `mergeProps` (default `true`).
    merge_props: Option<bool>,
    /// Babel `enableObjectSlots` (default `true`).
    enable_object_slots: Option<bool>,
    /// Babel `transformOn`.
    transform_on: Option<bool>,
    /// Babel `isCustomElement`: tags starting with this prefix.
    custom_element_prefix: Option<String>,
    /// VDOM: hoist static subtrees.
    hoist_static: Option<bool>,
    /// VDOM: cache inline event handlers.
    cache_handlers: Option<bool>,
    /// Emit source maps.
    source_map: Option<bool>,
}

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "every observation field is retained in the complete Debug oracle"
)]
struct Component {
    name: Option<String>,
    mode: JsxOutputMode,
    setup: Option<vize_atelier_jsx::ComponentSetupSpan>,
    preamble: String,
    code: String,
    map: Option<String>,
}

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "every observation field is retained in the complete Debug oracle"
)]
struct Observation {
    module: String,
    module_map: Option<String>,
    components: Vec<Component>,
    diagnostics: Vec<JsxDiagnostic>,
}

fn component(component: &JsxComponent) -> Component {
    Component {
        name: component.component_name().map(Into::into),
        mode: component.mode(),
        setup: component.component_setup().cloned(),
        preamble: component.preamble().into(),
        code: component.code().into(),
        map: component.map().map(Into::into),
    }
}

fn observe(case: &Case) -> Observation {
    let lang = match case.lang.as_str() {
        "jsx" => JsxLang::Jsx,
        "tsx" => JsxLang::Tsx,
        lang => panic!("unknown fixture lang: {lang}"),
    };
    let options = &case.options;
    let (default_mode, ssr) = match case.mode.as_deref().unwrap_or("vdom") {
        "vdom" => (JsxOutputMode::Vdom, false),
        "vapor" => (JsxOutputMode::Vapor, false),
        "ssr" => (JsxOutputMode::Vdom, true),
        mode => panic!("unknown fixture mode: {mode}"),
    };
    let compat = match options.compat.as_deref().unwrap_or("native") {
        "native" => JsxCompatMode::Native,
        "babel" => JsxCompatMode::Babel,
        compat => panic!("unknown fixture compat: {compat}"),
    };
    let mut config = JsxCompileConfig {
        default_mode,
        compat,
        ssr,
        ..JsxCompileConfig::default()
    };
    config.vdom.hoist_static = options.hoist_static.unwrap_or(false);
    config.vdom.cache_handlers = options.cache_handlers.unwrap_or(false);
    config.vdom.source_map = options.source_map.unwrap_or(false);
    let prefix = options.custom_element_prefix.clone();
    let is_custom_element = move |tag: &str| prefix.as_deref().is_some_and(|p| tag.starts_with(p));
    let allocator = Allocator::new();
    let output = compile_jsx_with_babel_customizations(
        &allocator,
        &case.source,
        lang,
        &config,
        &BabelJsxOptions {
            transform_on: options.transform_on.unwrap_or(false),
        },
        BabelJsxCustomizations {
            pragma: options.pragma.as_deref(),
            merge_props: options.merge_props.unwrap_or(true),
            is_custom_element: options
                .custom_element_prefix
                .is_some()
                .then_some(&is_custom_element as &vize_atelier_jsx::BabelIsCustomElement),
            enable_object_slots: options.enable_object_slots.unwrap_or(true),
        },
    );
    Observation {
        module: output.module_code(),
        module_map: output.source_map().map(Into::into),
        components: output.components.iter().map(component).collect(),
        diagnostics: output.diagnostics.clone(),
    }
}

#[test]
fn full_current_jsx_compile_results() {
    let cases: Vec<Case> = serde_json::from_str(include_str!("fixtures/jsx-history/cases.json"))
        .expect("strict fixture schema");
    assert_eq!(
        cases.len(),
        225,
        "every reviewed case must remain registered"
    );
    let mut mismatched = serde_json::Map::new();
    for case in cases {
        assert_eq!(case.history.len(), 40, "full historical commit identity");
        let first = observe(&case);
        let first_bytes = format!("{case:#?}\n{first:#?}");
        let repeat = observe(&case);
        assert_eq!(
            first_bytes,
            format!("{case:#?}\n{repeat:#?}"),
            "{}",
            case.id
        );
        let errors = first
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.is_error())
            .count();
        if errors != case.errors {
            mismatched.insert(case.id.as_str().into(), errors.into());
        }
        insta::assert_snapshot!(case.id.as_str(), first_bytes);
    }
    assert!(
        mismatched.is_empty(),
        "observed error counts differ: {}",
        serde_json::Value::Object(mismatched)
    );
}
