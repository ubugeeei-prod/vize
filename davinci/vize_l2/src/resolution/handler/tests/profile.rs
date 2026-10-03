use super::{
    Allocator, DescriptorOptions, NativeHandlerInput, NativeTemplateComponent, Outer,
    ResolutionErrorKind, Vue, VueDialect, VueVersion, resolve_handler,
};
use vize_l1::embed::Lang;

#[test]
fn original_ts_profile_is_intrinsic_and_type_annotations_remain_precise_refusals() {
    let allocator = Allocator::default();
    for (source, supported) in [
        (
            "<script setup lang='ts'></script><template><b @click='let x=1;x'/></template>",
            true,
        ),
        (
            "<script setup lang='ts'></script><template><b @click='let x:number=1;x'/></template>",
            false,
        ),
    ] {
        let descriptor = Vue.observe_descriptor(
            &allocator,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: vize_l1::SurfaceParseOptions::default(),
            },
        );
        let selected =
            NativeTemplateComponent::parse_in(&allocator, descriptor.admitted().unwrap())
                .unwrap()
                .unwrap();
        let element = selected.children().next().unwrap().into_element().unwrap();
        let operand = selected
            .observe_attribute_handler(element.attributes().next().unwrap())
            .unwrap();
        assert_eq!(operand.syntax().grammar().lang, Lang::Ts);
        assert!(operand.syntax().source_type().is_typescript());
        let original = NativeHandlerInput::new(operand).unwrap();
        let body = original.body();
        let result = resolve_handler(original, &Outer(&[]));
        if supported {
            let resolution = result.unwrap();
            assert!(core::ptr::eq(resolution.input().body(), body));
            assert_eq!(resolution.references().len(), 1);
        } else {
            let rejected = result.unwrap_err();
            assert_eq!(rejected.error.kind, ResolutionErrorKind::UnsupportedSyntax);
            assert_eq!(
                rejected
                    .error
                    .span
                    .slice(rejected.input().operand().syntax().source().text()),
                "x:number=1"
            );
            assert!(core::ptr::eq(rejected.input().body(), body));
            assert!(
                rejected
                    .input()
                    .operand()
                    .syntax()
                    .source_type()
                    .is_typescript()
            );
        }
    }
}
