//! Actual admitted Programs and diagnostic File factory rows feed the sole-file L3 walk.
mod dom_file_support;

use dom_file_support::{construct, script};
use vize_l0::Allocator;
use vize_l2::file::TemplateScope;
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};
use vize_l3::decision::dom::{DomDependency, DomUnsupported, ValueKind};
use vize_l3::decision::{DecisionBuildError, build_dom_file_decisions, policy::TargetPolicy};

#[test]
fn genuine_file_rows_keep_original_ast_coordinates_scope_and_dynamic_facts() {
    let arena = Allocator::default();
    let source = "é<script setup>const msg = 'hello'; const styles = 'color:red';</script><template><p :title=\"msg\" :style=\"styles\">{{msg}}</p></template>";
    let (syntax, block) = script(
        &arena,
        source,
        "const msg = 'hello'; const styles = 'color:red';",
    )
    .unwrap();
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap(),
            ProgramScope::Nested,
        )
        .unwrap();
    let native = construct(
        &arena,
        source,
        "<p :title=\"msg\" :style=\"styles\">{{msg}}</p>",
        &mut producer,
        TemplateScope::LastUnit,
    )
    .unwrap();
    assert!(native.is_supported());
    let file = producer.finish().unwrap();
    let result = build_dom_file_decisions(&file).unwrap();
    assert!(core::ptr::eq(result.file(), &file));
    assert!(core::ptr::eq(result.artifact(), file.artifact()));
    assert_eq!(result.policy(), TargetPolicy::Dom);
    let facts = result.dom().unwrap();
    assert!(facts.unsupported().is_empty());
    assert_eq!(
        result.tables().nodes.len(),
        result.artifact().node_count() as usize
    );
    for embed in &native.embeds {
        let node = embed.node.unwrap();
        let row = facts.file_expression(node).unwrap();
        assert_eq!(row.scope(), Some(file.units().last().unwrap().scope));
        assert!(core::ptr::eq(row.resolution().file(), &file));
        let table = row.resolution().table().unwrap();
        assert!(core::ptr::eq(
            table.expression().ast,
            embed.syntax.expression().unwrap()
        ));
        assert_eq!(table.expression().span, embed.syntax.source().span());
        assert_eq!(table.occurrences().len(), 1);
        if let Some(binding) = facts.binding(node) {
            assert_eq!(binding.value, ValueKind::FileDependent);
        }
    }
    assert!(
        facts
            .dependencies()
            .contains(&DomDependency::StyleNormalization)
    );
    assert!(facts.dependencies().contains(&DomDependency::DisplayValue));
    assert_eq!(syntax.diagnostics().count(), 0);
}

#[test]
fn authored_entity_literal_keeps_exact_resolution_and_zero_reference_semantics() {
    let arena = Allocator::default();
    let source = "<p :title=\"'a &amp; b'\">{{'日本語'}}</p>";
    let mut producer = FileProducer::new(&arena, source).unwrap();
    let native = construct(&arena, source, source, &mut producer, TemplateScope::Root).unwrap();
    assert!(native.is_supported());
    let file = producer.finish().unwrap();
    let result = build_dom_file_decisions(&file).unwrap();
    let facts = result.dom().unwrap();
    assert!(facts.unsupported().is_empty());
    for embed in &native.embeds {
        let node = embed.node.unwrap();
        let table = facts
            .file_expression(node)
            .unwrap()
            .resolution()
            .table()
            .unwrap();
        assert!(table.occurrences().is_empty());
        assert!(core::ptr::eq(
            table.expression().ast,
            embed.syntax.expression().unwrap()
        ));
        if let Some(binding) = facts.binding(node) {
            assert_eq!(binding.value, ValueKind::LiteralConstant);
        }
    }
    let entity = native.embeds.first().unwrap();
    assert_eq!(entity.syntax.source().text(), "'a & b'");
    assert_eq!(entity.syntax.diagnostics().count(), 0);
}

#[test]
fn actual_nested_unit_shadowing_uses_recorded_scope_without_context_fallback() {
    let arena = Allocator::default();
    let source = "<script>const msg = 'outer';</script><script setup>const msg = 'inner';</script><template>{{msg}}</template>";
    let (ordinary, ordinary_block) = script(&arena, source, "const msg = 'outer';").unwrap();
    let (setup, setup_block) = script(&arena, source, "const msg = 'inner';").unwrap();
    let mut producer = FileProducer::new(&arena, source).unwrap();
    let ordinary_unit = producer
        .program(
            ProgramInput::checked(ordinary.admitted_program().unwrap(), ordinary_block, 0).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    let nested_unit = producer
        .program(
            ProgramInput::checked(setup.admitted_program().unwrap(), setup_block, 1).unwrap(),
            ProgramScope::Nested,
        )
        .unwrap();
    let native = construct(
        &arena,
        source,
        "{{msg}}",
        &mut producer,
        TemplateScope::LastUnit,
    )
    .unwrap();
    assert!(native.is_supported());
    let file = producer.finish().unwrap();
    let result = build_dom_file_decisions(&file).unwrap();
    let resolution = result
        .dom()
        .unwrap()
        .file_expression(native.embeds.first().unwrap().node.unwrap())
        .unwrap()
        .resolution();
    assert_eq!(resolution.scope(), Some(file.units().last().unwrap().scope));
    let occurrence = resolution.table().unwrap().occurrences().first().unwrap();
    let declaration = resolution
        .binding(occurrence.binding)
        .unwrap()
        .declaration()
        .unwrap();
    assert_eq!(declaration.unit, nested_unit);
    assert_ne!(declaration.unit, ordinary_unit);
    assert!(result.dom().unwrap().unsupported().is_empty());
}

#[test]
fn missing_child_local_rejects_incomplete_file_before_returning_analysis() {
    let arena = Allocator::default();
    let source = "<script setup>function f(x) { const local = x; return local; }</script><template>{{local}}</template>";
    let (syntax, block) = script(
        &arena,
        source,
        "function f(x) { const local = x; return local; }",
    )
    .unwrap();
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap(),
            ProgramScope::Nested,
        )
        .unwrap();
    let native = construct(
        &arena,
        source,
        "{{local}}",
        &mut producer,
        TemplateScope::LastUnit,
    )
    .unwrap();
    assert!(!native.is_supported());
    let file = producer.finish().unwrap();
    assert!(!file.is_complete());
    assert!(matches!(
        build_dom_file_decisions(&file),
        Err(DecisionBuildError::IncompleteFile)
    ));
    assert_eq!(syntax.diagnostics().count(), 0);
}

#[test]
fn complete_zero_reference_nonliteral_does_not_gain_constant_or_runtime_eligibility() {
    let arena = Allocator::default();
    let source = "<p :title=\"[]\"/>";
    let mut producer = FileProducer::new(&arena, source).unwrap();
    let native = construct(&arena, source, source, &mut producer, TemplateScope::Root).unwrap();
    assert!(native.is_supported());
    let file = producer.finish().unwrap();
    let result = build_dom_file_decisions(&file).unwrap();
    let facts = result.dom().unwrap();
    let node = native.embeds.first().unwrap().node.unwrap();
    assert!(
        file.expression(node)
            .unwrap()
            .table()
            .unwrap()
            .occurrences()
            .is_empty()
    );
    assert!(facts.file_expression(node).is_none());
    assert!(facts.binding(node).is_none());
    assert!(
        facts
            .unsupported()
            .iter()
            .any(|row| row.node == node && row.reason == DomUnsupported::Expression)
    );
}

#[test]
fn equal_numeric_nodes_and_bindings_keep_distinct_file_owners_and_original_ast_rows() {
    let arena = Allocator::default();
    let source = "<script setup>const msg = 'hello';</script><template>{{msg}}</template>";
    let (syntax, block) = script(&arena, source, "const msg = 'hello';").unwrap();
    let mut first = FileProducer::new(&arena, source).unwrap();
    first
        .program(
            ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap(),
            ProgramScope::Nested,
        )
        .unwrap();
    let first_native = construct(
        &arena,
        source,
        "{{msg}}",
        &mut first,
        TemplateScope::LastUnit,
    )
    .unwrap();
    let first = first.finish().unwrap();
    let mut second = FileProducer::new(&arena, source).unwrap();
    second
        .program(
            ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap(),
            ProgramScope::Nested,
        )
        .unwrap();
    let second_native = construct(
        &arena,
        source,
        "{{msg}}",
        &mut second,
        TemplateScope::LastUnit,
    )
    .unwrap();
    let second = second.finish().unwrap();
    let first_result = build_dom_file_decisions(&first).unwrap();
    let second_result = build_dom_file_decisions(&second).unwrap();
    let node = first_native.embeds.first().unwrap().node.unwrap();
    assert_eq!(node, second_native.embeds.first().unwrap().node.unwrap());
    let first_row = first_result
        .dom()
        .unwrap()
        .file_expression(node)
        .unwrap()
        .resolution();
    let second_row = second_result
        .dom()
        .unwrap()
        .file_expression(node)
        .unwrap()
        .resolution();
    let first_use = first_row.table().unwrap().occurrences().first().unwrap();
    let second_use = second_row.table().unwrap().occurrences().first().unwrap();
    assert_eq!(first_use.binding, second_use.binding);
    let actual = first_row.binding(first_use.binding).unwrap();
    let foreign = second_row.binding(second_use.binding).unwrap();
    assert!(!actual.same_owner(foreign));
    assert!(!first_row.accepts(foreign));
    assert!(core::ptr::eq(first_row.file(), first_result.file()));
    assert!(core::ptr::eq(second_row.file(), second_result.file()));
    assert!(!core::ptr::eq(
        first_row.table().unwrap().expression().ast,
        second_row.table().unwrap().expression().ast
    ));
}
