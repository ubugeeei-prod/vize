//! A component or `<slot>` spelling under `v-pre` stays a plain element, as
//! Vue keeps it: `@vue/compiler-vapor` 3.6.0-rc.17 compiles
//! `<div v-pre><div><Child>{{ x }}</Child></div></div>` to one static
//! template and resolves no component. (It still opens a `createSlot` for a
//! `<slot>` inside the subtree, which `@vue/compiler-dom` does not; the
//! subtree stays markup here.)

use super::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

#[test]
fn component_and_slot_tags_under_v_pre_stay_markup() {
    for (source, template) in [
        (
            "<div v-pre><div><Child>{{ x }}</Child></div></div>",
            r#"const t0 = _template("<div><div><Child>{{ x }}</Child></div></div>", true)"#,
        ),
        (
            "<div v-pre><MyComp :x=\"1\" /><slot name=\"s\">f</slot></div>",
            r#"const t0 = _template("<div><MyComp :x=\"1\"></MyComp><slot name=\"s\">f</slot></div>", true)"#,
        ),
        (
            "<Child v-pre>{{ x }}</Child>",
            r#"const t0 = _template("<Child>{{ x }}</Child>", true)"#,
        ),
    ] {
        let allocator = Allocator::new();
        let result = compile_vapor(&allocator, source, VaporCompilerOptions::default());
        assert!(
            result.error_messages.is_empty(),
            "{source}: {:?}",
            result.error_messages
        );
        let code = result.code.as_str();
        assert_eq!(
            code,
            vize_carton::cstr!(
                "import {{ template as _template }} from 'vue';\n{template}\n\nexport function render(_ctx) {{\n  const n0 = t0()\n  return n0\n}}\n"
            )
            .as_str(),
            "{source}"
        );
    }
}
