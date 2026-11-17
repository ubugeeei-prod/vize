//! The checker helpers a runtime prop typed from its constructor value names.

use crate::macros::PropDefinition;
use vize_carton::String;

/// The checker preamble's own aliases, so the declaration stays self-contained.
pub(super) const RUNTIME_PROP_CTOR_HELPERS: &str = concat!(
    "type __RuntimePropValue<T> = T extends (...args: any[]) => infer V ? V : T extends abstract new (...args: any[]) => infer V ? V : never;\n",
    "type __RuntimePropCtorInner<T> = T extends null | undefined ? never : T extends readonly (infer U)[] ? __RuntimePropCtorInner<U> : T extends { type: infer U } ? __RuntimePropCtorInner<U> : T extends StringConstructor ? string : T extends NumberConstructor ? number : T extends BooleanConstructor ? boolean : T extends ArrayConstructor ? unknown[] : T extends ObjectConstructor ? Record<string, any> : T extends DateConstructor ? Date : T extends FunctionConstructor ? (...args: any[]) => any : __RuntimePropValue<T>;\n",
    "type __RuntimePropCtor<T> = [__RuntimePropCtorInner<T>] extends [never] ? unknown : __RuntimePropCtorInner<T>;\n",
);

pub(super) fn emit(ts: &mut String, props: &[PropDefinition]) {
    if props.iter().any(|prop| {
        prop.prop_type
            .as_deref()
            .is_some_and(|ty| ty.contains("__RuntimePropCtor<"))
    }) {
        ts.push_str(RUNTIME_PROP_CTOR_HELPERS);
    }
}
