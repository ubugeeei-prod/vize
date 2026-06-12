//! Croquis semantic analysis exposed alongside the lowered roots.

mod common;

use vize_atelier_jsx::{JsxLang, lower_source};
use vize_carton::Bump;

#[test]
fn top_level_bindings_are_collected() {
    let bump = Bump::new();
    let out = lower_source(
        &bump,
        "const count = 1;\nconst App = () => <div>{count}</div>;",
        JsxLang::Jsx,
    );
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    let bindings = out.bindings();
    assert!(bindings.contains("count"));
    assert!(bindings.contains("App"));
}

#[test]
fn ref_binding_is_recognized_as_reactive() {
    let bump = Bump::new();
    let out = lower_source(
        &bump,
        "import { ref } from 'vue';\nconst n = ref(0);\nconst C = () => <p>{n.value}</p>;",
        JsxLang::Jsx,
    );
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    assert!(out.bindings().is_ref("n"));
}

#[test]
fn analysis_runs_on_tsx_modules() {
    let bump = Bump::new();
    let out = lower_source(
        &bump,
        "const label: string = 'x';\nconst C = (): JSX.Element => <span>{label}</span>;",
        JsxLang::Tsx,
    );
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    assert!(out.bindings().contains("label"));
}

#[test]
fn undefined_binding_is_absent() {
    let bump = Bump::new();
    let out = lower_source(&bump, "const C = () => <div>{ghost}</div>;", JsxLang::Jsx);
    assert!(!out.bindings().contains("ghost"));
}
