use super::{JsxTypecheckEmit, frontend_counters, reset_frontend_counters, snapshot_jsx};
use crate::JsxLang;

#[test]
fn snapshot_runs_one_parse_and_one_lowering_without_direct_fallback() {
    reset_frontend_counters();
    let snapshot = snapshot_jsx(
        "const Comp = ({ msg }) => <strong>{msg}</strong>;",
        JsxLang::Tsx,
    );

    assert_eq!(snapshot.typecheck_roots().len(), 1);
    assert_eq!(frontend_counters(), (1, 1, 0));
}

#[test]
fn snapshot_projection_covers_valid_typecheck_inputs() {
    let source = "const Comp = ({ items, chosen, color }) => (\n  <>\n    {items.map((item, index) => (\n      <input v-model={[chosen.value, ['trim']]} value={item.label} data-index={index} />\n    ))}\n    <style scoped>{`.row { color: ${color}; }`}</style>\n  </>\n);\n";
    let snapshot = snapshot_jsx(source, JsxLang::Tsx);
    assert!(
        snapshot.diagnostics.is_empty(),
        "{:?}",
        snapshot.diagnostics
    );
    let [root] = snapshot.typecheck_roots() else {
        panic!("expected one JSX root");
    };
    let [
        JsxTypecheckEmit::ForScope {
            source,
            value,
            index,
            body,
        },
        JsxTypecheckEmit::Expression(style),
    ] = root.emits.as_slice()
    else {
        panic!("expected v-for followed by scoped-style interpolation");
    };
    assert_eq!(source.code.as_ref(), "items");
    assert_eq!(
        value.as_ref().map(|value| value.code.as_ref()),
        Some("item")
    );
    assert_eq!(
        index.as_ref().map(|index| index.code.as_ref()),
        Some("index")
    );
    assert!(matches!(
        body.first(),
        Some(JsxTypecheckEmit::ModelTarget(_))
    ));
    assert_eq!(style.code.as_ref(), "color");
}

#[test]
fn snapshot_projection_preserves_malformed_diagnostics() {
    let source = "const Broken = ({ msg }) => <div>{msg</div>;";
    let snapshot = snapshot_jsx(source, JsxLang::Tsx);
    assert!(snapshot.has_errors());
}
