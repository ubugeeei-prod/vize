//! Genuine File-owned For receipts, without loop output admission.

use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    embed::{
        EmbedSource,
        syntax::{ProgramOptions, parse_program_once},
    },
    markup::NativeTemplateComponent,
};
use vize_l2::{
    lang::js::{NativeTemplateFile, NativeTemplateOwner},
    op::{Op, OriginalForOp},
};
use vize_l3::decision::{
    ControlKind, StaticLevel, build_dom_decisions, build_dom_file_decisions,
    dom::{DomUnsupported, LiteralExpressions},
    native::build_native_dom_file_decisions,
};

mod original_for_decision_cases;

fn prepared<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateOwner<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let admitted = descriptor.admitted().unwrap();
    let script = admitted.setup().unwrap();
    let syntax = parse_program_once(
        arena,
        EmbedSource::authored(source, script.block().span()).unwrap(),
        ProgramOptions::module(script.lang()),
    );
    let selected = NativeTemplateComponent::parse_in(arena, admitted)
        .unwrap()
        .unwrap();
    let mut owner = NativeTemplateOwner::new(selected).unwrap_or_else(|_| panic!("owner"));
    owner
        .setup_program(syntax.admitted_program().unwrap())
        .unwrap();
    owner
}

fn completed<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateFile<'a> {
    let mut owner = prepared(arena, source);
    {
        let mut walk = owner.begin().unwrap();
        for child in walk.selected().children() {
            walk.child(child).unwrap();
        }
        walk.complete().unwrap();
    }
    core::hint::black_box(owner.finish())
}

fn first<'f, 'a>(owner: &'f NativeTemplateFile<'a>) -> &'f OriginalForOp<'a> {
    let [Op::OriginalFor(original)] = owner.file().unwrap().artifact().root().ops.as_slice() else {
        panic!("actual original For")
    };
    original
}

#[test]
fn js_and_ts_receipts_retain_original_file_allocation_source_and_stock_parameters() {
    let arena = Allocator::default();
    for lang in ["", " lang='ts'"] {
        let source = format!(
            "<script setup{lang}>const items=2;</script><template><div v-for='(item,index) in items'>text</div></template>"
        );
        let owner = completed(&arena, &source);
        let file = owner.file().unwrap();
        let original = first(&owner);
        let lower = file.for_head_for(original).unwrap();
        let facts = lower.resolution().unwrap();
        let analysis = build_native_dom_file_decisions(owner.view().unwrap()).unwrap();
        let row = analysis.for_head(original.id().node()).unwrap();
        assert!(core::ptr::eq(analysis.owner(), &owner));
        assert!(core::ptr::eq(row.head().file(), file));
        assert!(core::ptr::eq(row.original(), original));
        assert!(core::ptr::eq(row.resolution(), facts));
        assert!(row.accepts_original(original));
        assert!(core::ptr::eq(row.resolution().input(), facts.input()));
        assert!(core::ptr::eq(
            row.resolution().input().collection(),
            facts.input().collection()
        ));
        assert!(core::ptr::eq(
            row.resolution()
                .input()
                .operand()
                .syntax()
                .source()
                .authored_root(),
            source.as_str()
        ));
        assert_eq!(
            row.resolution().input().operand().raw_value(),
            "(item,index) in items"
        );
        assert_eq!(
            row.resolution().collection_authored_span().slice(&source),
            "items"
        );
        assert_eq!(row.collection().id(), facts.collection().binding);
        assert!(core::ptr::eq(row.collection().file(), file));
        assert!(row.collection().declaration().is_some());
        for (binding, expected) in [
            (row.head().value().unwrap(), facts.value_declaration()),
            (row.head().key().unwrap(), facts.key_declaration().unwrap()),
        ] {
            let declaration = binding.template_declaration().unwrap();
            let original_declaration = declaration.declaration().original().unwrap();
            assert!(core::ptr::eq(declaration.file(), file));
            assert_eq!(declaration.declaration().origin(), original.id());
            assert_eq!(Some(declaration.declaration().scope()), lower.scope());
            assert!(core::ptr::eq(original_declaration.resolution(), facts));
            assert!(core::ptr::eq(
                original_declaration.parameter(),
                expected.parameter()
            ));
            assert!(core::ptr::eq(
                original_declaration.pattern(),
                expected.pattern()
            ));
            assert_eq!(original_declaration.fact(), expected.fact());
        }
        let token = owner
            .selected()
            .children()
            .next()
            .unwrap()
            .into_element()
            .unwrap()
            .attributes()
            .next()
            .unwrap();
        let admitted = row
            .resolution()
            .input()
            .admitted_for(owner.selected(), token)
            .unwrap();
        let stock = admitted.for_head().unwrap();
        assert!(core::ptr::eq(
            stock.aliases().parameters().items.as_slice(),
            facts.input().aliases()
        ));
        assert!(core::ptr::eq(
            stock.collection().expression(),
            facts.input().collection()
        ));
        assert_eq!(analysis.tables().nodes.len(), 3);
        assert_eq!(analysis.tables().controls.len(), 1);
        assert_eq!(
            analysis
                .tables()
                .controls
                .get(original.id().node())
                .unwrap()
                .kind,
            ControlKind::Loop
        );
        assert_eq!(
            analysis
                .tables()
                .nodes
                .get(original.id().node())
                .unwrap()
                .output_level,
            StaticLevel::Dynamic
        );
        let refusals = analysis.dom().unwrap().unsupported();
        assert_eq!(refusals.len(), 1);
        assert_eq!(refusals[0].reason, DomUnsupported::Operation);
        assert_eq!(refusals[0].node, original.id().node());
        assert_eq!(refusals[0].span, original.span);
        let mut expression_count = 0;
        file.artifact()
            .visit_nodes(&mut |_, node| {
                node.for_each_expression(&mut |_| expression_count += 1);
            })
            .unwrap();
        assert_eq!(
            expression_count, 0,
            "a declaration head is never an ExprRef"
        );
    }
}

#[test]
fn foreign_equal_bytes_and_ids_never_replace_the_retained_allocation() {
    let arena = Allocator::default();
    let source =
        "<script setup>const items=2;</script><template><div v-for='item in items'/></template>";
    let first_owner = completed(&arena, source);
    let second_owner = completed(&arena, source);
    let original = first(&first_owner);
    let foreign = first(&second_owner);
    assert_eq!(original.id(), foreign.id());
    assert_eq!(original.span, foreign.span);
    let analysis = build_dom_file_decisions(first_owner.file().unwrap()).unwrap();
    let row = analysis
        .dom()
        .unwrap()
        .file_for_head(original.id().node())
        .unwrap();
    assert!(row.accepts_original(original));
    assert!(!row.accepts_original(foreign));
    assert!(
        second_owner
            .file()
            .unwrap()
            .for_head_for(original)
            .is_none()
    );
    assert!(row.head().key().is_none());
    assert!(row.resolution().key_declaration().is_none());
}

#[test]
fn bare_artifact_keeps_the_loop_refusal_without_file_custody() {
    let arena = Allocator::default();
    let owner = completed(
        &arena,
        "<script setup>const items=2;</script><template><div v-for='item in items'/></template>",
    );
    let original = first(&owner);
    let analysis =
        build_dom_decisions(owner.file().unwrap().artifact(), &LiteralExpressions).unwrap();
    assert!(
        analysis
            .dom()
            .unwrap()
            .file_for_head(original.id().node())
            .is_none()
    );
    assert_eq!(
        analysis.dom().unwrap().unsupported()[0].reason,
        DomUnsupported::Operation
    );
    assert_eq!(analysis.tables().nodes.len(), 2);
    assert_eq!(analysis.tables().controls.len(), 1);
}
