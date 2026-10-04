use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{SurfaceParseOptions, container::vue::DescriptorOptions};
use vize_l1_to_l2::native_file::lower_selected_setup_sfc_native;
use vize_l2::file::{DeclarationKind, InitializerKind};
use vize_l3::decision::{
    dom::{ValueKind, vue::VueReadKind},
    native::build_native_selected_setup_dom_decisions,
};

#[test]
fn genuine_selected_dom_reads_preserve_all_original_primitive_classes() {
    for (literal, expected) in [
        (r#"'a\0b'"#, InitializerKind::PrimitiveStringWithNulOrCr),
        (r#"'a\rb'"#, InitializerKind::PrimitiveStringWithNulOrCr),
        (
            r#"'a\ud800b'"#,
            InitializerKind::PrimitiveStringWithLoneSurrogates,
        ),
        (
            r#"'a\udc00b'"#,
            InitializerKind::PrimitiveStringWithLoneSurrogates,
        ),
        (r#"'a\nb'"#, InitializerKind::PrimitiveLiteral),
        (r#"'a\ud83c\udf38b'"#, InitializerKind::PrimitiveLiteral),
        (r#"'a\ufffdb'"#, InitializerKind::PrimitiveLiteral),
    ] {
        for (keyword, kind, read_kind) in [
            ("const", DeclarationKind::Const, VueReadKind::SetupConst),
            ("let", DeclarationKind::Let, VueReadKind::SetupLet),
            ("var", DeclarationKind::Var, VueReadKind::SetupLet),
        ] {
            let arena = Allocator::default();
            let source = format!(
                "<script setup>{keyword} value={literal};</script><template>{{{{value}}}}</template>"
            );
            let observation = lower_selected_setup_sfc_native(
                &arena,
                &source,
                DescriptorOptions {
                    version: VueVersion::V3,
                    dialect: VueDialect::Vue,
                    template: SurfaceParseOptions::default(),
                },
            );
            assert!(observation.original().issues().is_empty(), "{source}");
            let selected = observation.admitted().unwrap();
            let setup = selected.setup();
            let analysis = build_native_selected_setup_dom_decisions(setup).unwrap();
            assert!(analysis.dom().unwrap().unsupported().is_empty(), "{source}");
            let expression = analysis.expression(vize_l0::id::NodeId::FIRST).unwrap();
            assert_eq!(
                expression.value(),
                if kind == DeclarationKind::Const {
                    ValueKind::LiteralConstant
                } else {
                    ValueKind::FileDependent
                }
            );
            let [read] = expression.reads() else {
                panic!("one original read")
            };
            assert_eq!(read.kind(), read_kind);
            let row = read.binding().declaration().unwrap();
            assert_eq!(row.initializer, expected);
            assert_eq!(row.kind, kind);
            assert_eq!(row.unit, setup.unit());
            assert_eq!(row.scope, setup.scope());
            assert!(setup.binding(read.binding()).is_ok());
            assert!(core::ptr::eq(read.binding().file(), setup.file()));
        }
    }
}
