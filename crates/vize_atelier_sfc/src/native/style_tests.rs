use super::{NativeSfcCompileError, NativeSfcCompileOptions, compile_native_sfc};
use vize_l0::Allocator;

#[test]
fn original_style_sources_never_disappear_into_a_successful_module() {
    for source in [
        "<style>p{color:red}</style><template><p/></template>",
        "<template><p/></template><style scoped lang=scss>p{color:v-bind(color)}</style>",
        "<template><p/></template><style module=theme>p{color:red}</style><style></style>",
        "<script>const x=1</script><template><p/></template><style>p{color:red}</style>",
    ] {
        let arena = Allocator::default();
        let compilation = compile_native_sfc(&arena, source, NativeSfcCompileOptions::default());
        let descriptor = compilation.observation().descriptor().admitted().unwrap();
        let style = descriptor.styles().next().unwrap();
        assert_eq!(
            compilation.result().unwrap_err(),
            NativeSfcCompileError::StyleCompilationUnavailable {
                container_index: style.container_index(),
                span: style.block().span(),
            }
        );
        assert_eq!(style.original_block().content, style.block().span());
        assert!(core::ptr::eq(style.source(), source));
        assert!(compilation.observation().template().is_some());
        assert!(compilation.observation().file().is_some());
    }
}
