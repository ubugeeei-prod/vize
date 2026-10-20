//! Asset components resolve once at the top of the render, and loop rows that
//! are components carry `IS_COMPONENT` so each row unmounts its component, as
//! `@vue/compiler-vapor` 3.6.0-rc.11 emits them. A component created once in
//! the root block still resolves where it is created.

use super::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

#[test]
fn loop_row_components_resolve_once_and_unmount_one_by_one() {
    let source = r#"<div><MyEl v-for="i in list" :key="i"/><Other/></div>"#;
    let expected = r#"import { resolveComponent as _resolveComponent, createComponentWithFallback as _createComponentWithFallback, setInsertionState as _setInsertionState, createFor as _createFor, template as _template } from 'vue';
const t0 = _template("<div></div>", true)

export function render(_ctx) {
  const _component_MyEl = _resolveComponent("MyEl")
  const n1 = t0()
  _setInsertionState(n1)
  const n2 = _createFor(() => (_ctx.list), (_for_item0) => {
    const n4 = _createComponentWithFallback(_component_MyEl)
    return n4
  }, (i) => (i), 2 /* IS_COMPONENT */)
  const _component_Other = _resolveComponent("Other")
  _setInsertionState(n1, 1)
  const n0 = _createComponentWithFallback(_component_Other, null, null, false)
  return n1
}"#;
    for retained in [false, true] {
        let allocator = Allocator::new();
        let result = compile_vapor(
            &allocator,
            source,
            VaporCompilerOptions {
                prefix_identifiers: true,
                davinci_retained_lane: retained,
                ..Default::default()
            },
        );
        assert_eq!(
            result.error_messages.len(),
            0,
            "{:?}",
            result.error_messages
        );
        assert_eq!(result.code.trim_end(), expected, "retained: {retained}");
    }
}
