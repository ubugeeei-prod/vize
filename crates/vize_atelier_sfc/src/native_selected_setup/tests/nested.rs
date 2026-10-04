//! The original refused source becomes positive only with its genuine provider.

use super::*;
use vize_l2::{expr::ExprRef, file::NativeFileInterpolationState, op::Op};

#[test]
fn exact_original_nested_case_retains_element_operand_file_scope_and_setup_read() {
    let source = "<script setup>let count=1</script><template><p>{{count}}</p></template>";
    let arena = Allocator::default();
    let compiled = compile_native_selected_setup_sfc_dom(
        &arena,
        source,
        NativeSelectedSfcDomOptions::default(),
    );
    let output = compiled.result().expect("whole original nested output");
    let envelope = compiled.observation().admitted().expect("whole envelope");
    let setup = envelope.setup();
    let file = setup.file();
    assert!(file.is_complete());
    assert!(file.native_interpolation_failures().is_empty());
    assert_eq!(file.native_interpolations().len(), 1);
    assert!(core::ptr::eq(
        setup.syntax(),
        setup
            .owner()
            .retained_setup()
            .expect("normal whole Program")
    ));
    assert!(core::ptr::eq(
        setup
            .syntax()
            .program()
            .expect("stock Program")
            .body
            .as_ptr(),
        setup.program().program().body.as_ptr()
    ));
    let selected = setup.owner().selected();
    let original = selected
        .children()
        .next()
        .expect("actual original parent")
        .into_element()
        .expect("original p");
    let child = original.children().next().expect("actual nested child");
    assert!(core::ptr::eq(
        child.parent_element().expect("original parent association"),
        original.surface()
    ));
    let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
        panic!("original p region");
    };
    let [Op::Interpolation(interpolation)] = element.children.ops.as_slice() else {
        panic!("one original nested expression");
    };
    let [record] = file.native_interpolations() else {
        panic!("one complete original operand");
    };
    let NativeFileInterpolationState::Admitted(node) = record.state() else {
        panic!("normally attached original node");
    };
    assert!(core::ptr::eq(
        file.native_interpolation(node)
            .expect("same actual File row"),
        record
    ));
    assert!(
        record
            .input()
            .operand()
            .admitted_for(selected, child)
            .is_some()
    );
    assert_eq!(record.input().operand().raw_content(), "count");
    assert_eq!(interpolation.span, record.input().operand().full_span());
    assert_eq!(interpolation.span.slice(source), "{{count}}");
    assert!(core::ptr::eq(
        record.input().operand().syntax().source().authored_root(),
        source
    ));
    let ExprRef::Js(expression) = interpolation.expression else {
        panic!("genuine original expression");
    };
    assert!(core::ptr::eq(
        expression.ast,
        record
            .input()
            .operand()
            .syntax()
            .expression()
            .expect("stock root")
    ));
    let analysis = build_native_selected_setup_dom_decisions(setup).expect("sole DOM Enter");
    let row = analysis
        .expression(node)
        .expect("actual original scoped row");
    let resolution = row.resolution();
    assert!(core::ptr::eq(resolution.file(), file));
    assert_eq!(resolution.scope(), Some(setup.scope()));
    let table = resolution
        .table()
        .expect("whole immutable original occurrences");
    assert!(core::ptr::eq(table.expression().ast, expression.ast));
    assert_eq!(table.occurrences().len(), 1);
    let [read] = row.reads() else {
        panic!("one actual setup read");
    };
    assert!(core::ptr::eq(read.occurrence(), &table.occurrences()[0]));
    let binding = setup
        .binding(read.binding())
        .expect("same-File actual setup binding");
    let declaration = binding.declaration().expect("original root declaration");
    assert_eq!(declaration.unit, setup.unit());
    assert_eq!(declaration.scope, setup.scope());
    assert_eq!(read.kind(), VueReadKind::SetupLet);
    assert!(output.document().links().is_empty());
    assert!(output.source_map().is_none());
    assert!(
        output
            .code()
            .contains("_toDisplayString($setup.count), 1 /* TEXT */")
    );
}
