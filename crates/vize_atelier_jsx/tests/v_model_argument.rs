//! `v-model` array-form argument validation (#3466).
//!
//! A non-literal argument needs computed prop keys and update-listener names.
//! Until that codegen exists, lowering must reject the input instead of
//! silently binding `modelValue` and changing the component contract.

use vize_atelier_jsx::{JsxLang, VdomCompileOptions, compile_to_vdom, lower_source};
use vize_carton::Bump;

const SOURCE: &str = "const A = () => <B v-model={[foo, bar]}/>;";

#[test]
fn dynamic_array_argument_is_rejected_without_model_value_fallback() {
    let bump = Bump::new();
    let lowered = lower_source(&bump, SOURCE, JsxLang::Jsx);
    let errors: Vec<_> = lowered
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.is_error())
        .collect();

    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].message.as_str(),
        "v-model argument `bar` must be a string literal; dynamic arguments are not supported."
    );
    assert_eq!(
        &SOURCE[errors[0].start as usize..errors[0].end as usize],
        "bar"
    );

    let bump = Bump::new();
    let compiled = compile_to_vdom(&bump, SOURCE, JsxLang::Jsx, VdomCompileOptions::default());
    assert!(compiled.has_errors());
    assert_eq!(compiled.components.len(), 1);
    assert_eq!(
        compiled.components[0].code.as_str(),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B))\n}"
    );
}
