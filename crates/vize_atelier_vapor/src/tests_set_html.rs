//! `v-html` writes through the runtime's `setHtml`, which maps `null` and
//! `undefined` to an empty string and skips HTML it already set, so an effect
//! re-run for another binding keeps the element's children.

use super::{VaporCompilerOptions, compile_vapor};
use vize_carton::{Allocator, String};

fn render_lines(source: &str, davinci_retained_lane: bool) -> std::vec::Vec<String> {
    let allocator = Allocator::new();
    let result = compile_vapor(
        &allocator,
        source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            davinci_retained_lane,
            ..Default::default()
        },
    );
    assert!(
        result.error_messages.is_empty(),
        "{:?}",
        result.error_messages
    );
    result
        .code
        .lines()
        .map(|line| String::from(line.trim()))
        .collect()
}

#[test]
fn v_html_sets_html_through_the_runtime_helper_on_both_lanes() {
    for retained in [false, true] {
        let lines = render_lines(r#"<div v-html="html"></div>"#, retained);
        assert_eq!(
            lines
                .iter()
                .filter(|line| *line == "_renderEffect(() => _setHtml(n0, _ctx.html))")
                .count(),
            1,
            "{lines:#?}"
        );
        let shared = render_lines(r#"<div v-html="html" :id="id"></div>"#, retained);
        assert_eq!(
            shared
                .iter()
                .filter(|line| *line == "_setHtml(n0, _ctx.html)")
                .count(),
            1,
            "{shared:#?}"
        );
        assert_eq!(
            shared
                .iter()
                .filter(|line| line.starts_with("n0.innerHTML"))
                .count(),
            0,
            "{shared:#?}"
        );
    }
}

/// On a component, `v-text` and `v-html` become the `textContent` and
/// `innerHTML` props that write its root element's content, as upstream
/// lowers them.
#[test]
fn content_directives_on_components_become_content_props() {
    let source = r#"<div><Comp v-text="msg" :a="b"/><component :is="c" v-html="h"/></div>"#;
    let expected = r#"import { resolveComponent as _resolveComponent, createComponentWithFallback as _createComponentWithFallback, createDynamicComponent as _createDynamicComponent, toDisplayString as _toDisplayString, setInsertionState as _setInsertionState, template as _template } from 'vue';
const t0 = _template("<div></div>", true)

export function render(_ctx) {
  const n2 = t0()
  const _component_Comp = _resolveComponent("Comp")
  _setInsertionState(n2)
  const n0 = _createComponentWithFallback(_component_Comp, {
    textContent: () => (_toDisplayString(_ctx.msg)),
    a: () => (_ctx.b)
  }, null, false)
  _setInsertionState(n2, 1)
  const n1 = _createDynamicComponent(() => (_ctx.c), { innerHTML: () => (_ctx.h) }, null, false)
  return n2
}"#;
    for davinci_retained_lane in [false, true] {
        let allocator = Allocator::new();
        let result = compile_vapor(
            &allocator,
            source,
            VaporCompilerOptions {
                prefix_identifiers: true,
                davinci_retained_lane,
                ..Default::default()
            },
        );
        assert_eq!(result.code.trim_end(), expected);
    }
}
