//! A component under `v-once`, its own or an ancestor's, is created with the
//! runtime's `once` flag, so its props and slots are read a single time.

use super::{VaporCompilerOptions, compile_vapor};
use vize_carton::{Allocator, String};

fn compile(source: &str, davinci_retained_lane: bool) -> String {
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
    result.code
}

#[test]
fn a_component_with_v_once_is_created_once() {
    for retained in [false, true] {
        let code = compile(r#"<Child v-once :value="count"/>"#, retained);
        assert!(
            code.contains("{ value: () => (_ctx.count) }, null, true, true)"),
            "{code}"
        );
    }
}

#[test]
fn a_component_inside_a_v_once_element_is_created_once() {
    for retained in [false, true] {
        let code = compile(
            r#"<div v-once><p><Child :value="count"><template #label>{{ label }}</template></Child></p></div>"#,
            retained,
        );
        assert!(code.contains("}, false, true)"), "{code}");
    }
}

#[test]
fn a_component_outside_v_once_stays_reactive() {
    for retained in [false, true] {
        let code = compile(
            r#"<div v-once>static</div><Child :value="count"/>"#,
            retained,
        );
        assert!(
            code.contains("{ value: () => (_ctx.count) }, null, false)"),
            "{code}"
        );
    }
}
