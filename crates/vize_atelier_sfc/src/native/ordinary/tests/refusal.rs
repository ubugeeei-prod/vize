use crate::{NativeSfcCompileError, NativeSfcCompileOptions, compile_native_sfc};
use vize_l0::Allocator;
use vize_l2::lang::js::VueOrdinaryEmpty;

#[test]
fn ordinary_empty_product_keeps_other_options_mixed_setup_and_strict_refusals() {
    for (lang, body) in [
        ("", ""),
        ("", ";;;"),
        ("", "const value=1;"),
        ("", "export default {name:'options'};"),
        ("", "export default ({});"),
        ("", "export default function(){}"),
        ("", "import 'dep';export default {};"),
        ("", "export default {};export {};"),
        ("", "export default {};42;"),
        ("", "'\\1';export default {};"),
        ("", "'\\8';export default {};"),
        ("", ";'use strict';export default {};"),
        (" lang='ts'", "export default {} as const;"),
        (" lang='ts'", "export default {} satisfies {};"),
        (" lang='ts'", "export default {};export default {};"),
        (" lang='ts'", "type Value=number;export default {};"),
    ] {
        let source = format!("<script{lang}>{body}</script><template><p>kept</p></template>");
        let arena = Allocator::default();
        let result = compile_native_sfc(&arena, &source, NativeSfcCompileOptions::default());
        let script = result.observation().scripts().first().unwrap();
        assert!(
            matches!(result.result(), Err(NativeSfcCompileError::ScriptCompilationUnavailable{container_index,span})
            if container_index == script.container_index() && span == script.block().span()),
            "{source}"
        );
        let syntax = script.syntax().unwrap();
        assert_eq!(syntax.diagnostics().count(), 0, "{source}");
        assert!(core::ptr::eq(
            syntax.source().text(),
            script.block().source()
        ));
        assert!(core::ptr::eq(script.block().root_source(), source.as_str()));
    }
    for setup in ["", "const value=1;", "const value=defineProps();"] {
        let source = format!(
            "<script>export default {{}};</script><script setup>{setup}</script><template>kept</template>"
        );
        let arena = Allocator::default();
        let result = compile_native_sfc(&arena, &source, NativeSfcCompileOptions::default());
        assert!(matches!(
            result.result(),
            Err(NativeSfcCompileError::ScriptCompilationUnavailable { .. })
        ));
        assert_eq!(result.observation().scripts().len(), 2);
        assert!(
            result
                .observation()
                .descriptor()
                .admitted()
                .unwrap()
                .setup()
                .is_some()
        );
        assert!(
            result
                .observation()
                .scripts()
                .iter()
                .all(|script| script.syntax().is_some())
        );
    }
}

#[test]
fn ordinary_ts_source_and_css_owners_survive_nonzero_container_index_together() {
    let source = "<style>/*first*/ .a{color:red}</style><script lang='ts'>/*script*/ export default {};</script><template><p>kept</p></template><style>.b{color:blue}</style>";
    let arena = Allocator::default();
    let result = compile_native_sfc(
        &arena,
        source,
        NativeSfcCompileOptions {
            filename: "Union.vue",
            source_map: true,
            ..NativeSfcCompileOptions::default()
        },
    );
    let output = result.result().unwrap();
    assert_eq!(
        output.css(),
        Some("/*first*/ .a{color:red}\n.b{color:blue}")
    );
    assert!(output.css_source_map().is_some());
    assert!(output.source_map().is_some());
    let owner = result.observation();
    let descriptor = owner.descriptor().admitted().unwrap();
    assert!(descriptor.setup().is_none());
    let script = descriptor.ordinary().unwrap();
    assert_eq!(script.container_index(), 1);
    assert_eq!(descriptor.styles().count(), 2);
    let file = owner.admitted().unwrap().file().file();
    let syntax = owner.scripts().first().unwrap().syntax().unwrap();
    let ordinary =
        VueOrdinaryEmpty::checked(file, script, syntax.admitted_program().unwrap()).unwrap();
    assert_eq!(ordinary.unit().index(), 1);
    assert!(ordinary.program().source_type().is_typescript());
    assert_eq!(syntax.comments().count(), 1);
    assert!(core::ptr::eq(ordinary.file(), file));
    assert!(core::ptr::eq(ordinary.source().root_source(), source));
    let plain_source =
        "<script lang='ts'>/*script*/ export default {};</script><template><p>kept</p></template>";
    let plain = compile_native_sfc(&arena, plain_source, NativeSfcCompileOptions::default());
    assert_eq!(output.code(), plain.result().unwrap().code());
    assert!(plain.result().unwrap().css().is_none());
}
