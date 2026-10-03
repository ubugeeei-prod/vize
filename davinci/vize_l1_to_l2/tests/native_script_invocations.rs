use oxc_ast::ast::{Expression, Statement};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{SurfaceParseOptions, container::vue::DescriptorOptions};
use vize_l1_to_l2::native_file::{NativeTemplateOutcome, lower_sfc_native};
use vize_l1_to_l2::vue_file::VueFileIssueKind;

#[test]
fn real_sfc_constructor_tagged_and_dynamic_import_keep_originals_without_admission() {
    let arena = Allocator::default();
    for (source, family) in [
        (
            "<script setup>/* original */ function Factory() {} let value = new Factory();</script><template>kept</template>",
            0,
        ),
        (
            "<script setup>/* original */ function tag() { return 1; } let value = tag`text`;</script><template>kept</template>",
            1,
        ),
        (
            "<script setup>/* original */ let value = import('dep');</script><template>kept</template>",
            2,
        ),
    ] {
        let observed = lower_sfc_native(
            &arena,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        assert!(observed.admitted().is_none(), "{source}");
        let [script] = observed.scripts() else {
            panic!("script");
        };
        let syntax = script.syntax().unwrap();
        let original = syntax.program().unwrap();
        let Statement::VariableDeclaration(variable) = original.body.last().unwrap() else {
            panic!("variable");
        };
        let [declaration] = variable.declarations.as_slice() else {
            panic!("declaration");
        };
        assert!(matches!(
            (family, declaration.init.as_ref().unwrap()),
            (0, Expression::NewExpression(_))
                | (1, Expression::TaggedTemplateExpression(_))
                | (2, Expression::ImportExpression(_))
        ));
        assert!(syntax.admitted_program().is_some());
        assert_eq!(syntax.comments().count(), 1);
        assert!(observed.rejected_file().is_some());
        assert!(
            observed
                .rejected_file()
                .unwrap()
                .issues()
                .iter()
                .any(|issue| issue.kind == VueFileIssueKind::UnsupportedInvocation)
        );
        let NativeTemplateOutcome::FactoryRefused { component, .. } =
            observed.template().unwrap().outcome()
        else {
            panic!("retained template");
        };
        assert_eq!(component.block().source(), "kept");
        assert_eq!(component.carrier().tree.children.len(), 1);
    }
}
