//! Every original nested negative's custody law, now required on accepted output.
use vize_l2::{
    expr::ExprRef, file::NativeFileInterpolationState, lang::js::NativeSelectedSetup, op::Op,
};

pub fn assert_original(setup: &NativeSelectedSetup<'_, '_>, source: &str) {
    let name = "nested";
    let file = setup.file();
    let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
        panic!("{name}: original root element missing")
    };
    let [Op::Interpolation(interpolation)] = element.children.ops.as_slice() else {
        panic!("{name}: original nested interpolation missing")
    };
    let ExprRef::Js(expression) = interpolation.expression else {
        panic!("{name}: original JS expression missing")
    };
    let [record] = file.native_interpolations() else {
        panic!("{name}: exact original input missing")
    };
    let NativeFileInterpolationState::Admitted(node) = record.state() else {
        panic!("{name}: genuine interpolation admission missing")
    };
    assert!(core::ptr::eq(
        file.native_interpolation(node).unwrap(),
        record
    ));
    let operand = record.input().operand();
    let syntax = operand.syntax();
    assert_eq!(operand.full_span(), interpolation.span);
    assert_eq!(operand.content_span().slice(source), "value");
    assert_eq!(operand.raw_content(), "value");
    assert_eq!(syntax.diagnostics().count(), 0);
    assert!(core::ptr::eq(syntax.source().authored_root(), source));
    assert!(core::ptr::eq(syntax.source().text(), expression.source));
    assert_eq!(syntax.source().span(), expression.span);
    assert!(core::ptr::eq(
        syntax.admitted_expression().unwrap().expression(),
        expression.ast
    ));
    let resolution = file.expression(node).unwrap();
    assert!(core::ptr::eq(resolution.file(), file));
    assert_eq!(resolution.scope(), Some(setup.scope()));
    let table = resolution.table().unwrap();
    assert!(core::ptr::eq(table.expression().ast, expression.ast));
    assert!(core::ptr::eq(
        table.expression().coordinates.unwrap(),
        expression.coordinates.unwrap()
    ));
    let [occurrence] = table.occurrences() else {
        panic!("{name}: actual original reference missing")
    };
    let binding = resolution.binding(occurrence.binding).unwrap();
    assert!(resolution.accepts(binding));
    assert_eq!(
        setup.binding(binding).unwrap().declaration().unwrap().unit,
        setup.unit()
    );
}
