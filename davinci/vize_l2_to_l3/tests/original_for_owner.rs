//! Mechanical consumers retain an actual File-owned For without inventing heads.
//!
//! `emit_dom` requires an owned neutral `Lowered::root`; the original File
//! exposes only its immutable region borrow. These laws do not copy the sealed
//! original op, invent a neutral Lowered, or claim that this emitter route has
//! a File-qualified provider. Its explicit closed-family refusal remains
//! `OriginalForProviderUnavailable` until that real borrowed provider exists.
//! Generic L3 verification proves structure only, never product admission.

use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
    dump::{Dump, Mode},
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
    dump::{Op as DumpOp, Page},
    file::FileArtifact,
    lang::js::{NativeTemplateFile, NativeTemplateOwner},
    op::{Op, OriginalForOp},
};
use vize_l2_to_l3::{PartitionKind, lower};
use vize_l3::{
    op::{OpKind, RegionId},
    operand::{OperandRole, ValueKind},
    verify::verify,
};

const SOURCE: &str = "<script setup>const items=2;</script><template><div v-for='item in items'>body</div></template>";
const OWNER_SPAN: Span = Span::new(47, 84);
const BODY_SPAN: Span = Span::new(74, 78);
const DIAGNOSTIC: &str = "\
[l2-dump-v2]
ops=3

[l2-dump-v2.ops]
ui.for original-ref=0 @47:84
  ui.element div @47:84
    ui.text \"body\" @74:78

";

fn original_file(arena: &Allocator) -> Result<NativeTemplateFile<'_>, &'static str> {
    let descriptor = Vue.observe_descriptor(
        arena,
        SOURCE,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let admitted = descriptor.admitted().map_err(|_| "original Descriptor")?;
    let setup = admitted.setup().ok_or("actual setup selection")?;
    let syntax = parse_program_once(
        arena,
        EmbedSource::authored(SOURCE, setup.block().span()).map_err(|_| "original setup source")?,
        ProgramOptions::module(setup.lang()),
    );
    let selected = NativeTemplateComponent::parse_in(arena, admitted)
        .map_err(|_| "selected original component")?
        .ok_or("selected template")?;
    let mut owner = NativeTemplateOwner::new(selected).map_err(|_| "normal original owner")?;
    owner
        .setup_program(syntax.admitted_program().ok_or("once original Program")?)
        .map_err(|_| "actual setup walk")?;
    {
        let mut walk = owner.begin().map_err(|_| "original root walk")?;
        let child = walk
            .selected()
            .children()
            .next()
            .ok_or("original For child")?;
        walk.child(child)
            .map_err(|_| "actual positive For factory")?;
        walk.complete()
            .map_err(|_| "complete actual original root")?;
    }
    let output = owner.finish();
    let file = output
        .view()
        .map_err(|_| "native completion")?
        .file()
        .ok_or("actual File")?;
    assert!(file.is_complete());
    Ok(output)
}

fn original_for<'f, 'a>(file: &'f FileArtifact<'a>) -> Result<&'f OriginalForOp<'a>, &'static str> {
    let [Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
        return Err("real original For root");
    };
    Ok(original)
}

fn assert_original_custody(file: &FileArtifact<'_>) -> Result<(), &'static str> {
    let original = original_for(file)?;
    let head = file
        .for_head_for(original)
        .ok_or("same actual allocation")?;
    assert!(core::ptr::eq(head.file(), file));
    assert!(head.accepts(original));
    assert_eq!(head.id().node().index(), 0);
    let resolution = head.resolution().ok_or("normally retained whole head")?;
    assert_eq!(resolution.input().operand().raw_value(), "item in items");
    assert_eq!(resolution.collection().name, "items");
    let declaration = head
        .value()
        .ok_or("authentic alias BindingId")?
        .template_declaration()
        .ok_or("actual template declaration")?;
    assert!(core::ptr::eq(declaration.file(), file));
    assert_eq!(declaration.declaration().origin(), original.id());
    assert_eq!(
        declaration
            .declaration()
            .original()
            .ok_or("original parameter")?
            .fact()
            .name(),
        "item"
    );
    Ok(())
}

#[test]
fn actual_preorder_visits_each_original_region_once_without_head_expressions()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let output = original_file(&arena)?;
    let file = output.file().ok_or("actual File")?;
    let artifact = file.artifact();
    let mut visited = Vec::new();
    let mut expressions = 0;
    artifact
        .visit_nodes(&mut |id, node| {
            visited.push((id.index(), node.mnemonic(), node.span()));
            node.for_each_expression(&mut |_| expressions += 1);
        })
        .map_err(|_| "complete preorder")?;
    assert_eq!(artifact.node_count(), 3);
    assert_eq!(
        visited,
        [
            (0, "ui.for", OWNER_SPAN),
            (1, "ui.element", OWNER_SPAN),
            (2, "ui.text", BODY_SPAN),
        ]
    );
    assert_eq!(expressions, 0);
    assert_original_custody(file)
}

#[test]
fn actual_dump_round_trip_keeps_only_diagnostic_readback_and_the_real_body()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let output = original_file(&arena)?;
    let file = output.file().ok_or("actual File")?;
    let page = Page::of(&file.artifact().root().ops);
    assert_eq!(page.op_count(), u64::from(file.artifact().node_count()));
    assert_eq!(page.print_to_string(Mode::Full).as_str(), DIAGNOSTIC);
    let reparsed = Page::parse(DIAGNOSTIC).map_err(|_| "diagnostic parse")?;
    assert_eq!(reparsed, page);
    let [DumpOp::OriginalFor(diagnostic)] = reparsed.ops.as_slice() else {
        return Err("numeric diagnostic mirror");
    };
    assert_eq!(diagnostic.node, 0);
    assert_eq!(diagnostic.span, OWNER_SPAN);
    assert_eq!(diagnostic.ops.len(), 1);
    // The mirror exposes no OriginalForId/ForResolution/File capability.
    // Read the original receipt only through the untouched real File.
    assert_original_custody(file)
}

#[test]
fn generic_lowering_retains_dynamic_body_but_has_no_neutral_head_operands()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let output = original_file(&arena)?;
    let file = output.file().ok_or("actual File")?;
    let lowered = lower(&arena, file.artifact().root());
    let program = &lowered.program;
    assert_eq!(
        program.ops.iter().map(|op| op.kind).collect::<Vec<_>>(),
        [OpKind::For, OpKind::InsertNode, OpKind::SetText]
    );
    assert_eq!(
        program.ops.iter().map(|op| op.span).collect::<Vec<_>>(),
        [OWNER_SPAN, OWNER_SPAN, BODY_SPAN]
    );
    assert_eq!(
        lowered
            .partition
            .ops
            .iter()
            .map(|fact| fact.kind)
            .collect::<Vec<_>>(),
        [PartitionKind::Dynamic; 3]
    );
    assert_eq!(program.regions.len(), 3);
    let for_op = &program.ops[0];
    let element = &program.ops[1];
    let text = &program.ops[2];
    assert_eq!(for_op.region, RegionId::ROOT);
    assert_eq!(program.regions[1].parent, Some(RegionId::ROOT));
    assert_eq!(program.regions[1].owner, Some(for_op.id));
    assert_eq!(element.region, program.regions[1].id);
    assert_eq!(program.regions[2].parent, Some(element.region));
    assert_eq!(program.regions[2].owner, Some(element.id));
    assert_eq!(text.region, program.regions[2].id);
    assert_eq!(
        program
            .operands
            .iter()
            .map(|operand| (
                operand.op,
                operand.role,
                operand.value.kind,
                operand.value.text,
                operand.value.span,
            ))
            .collect::<Vec<_>>(),
        [
            (
                element.id,
                OperandRole::Tag,
                ValueKind::Literal,
                "div",
                OWNER_SPAN
            ),
            (
                element.id,
                OperandRole::Namespace,
                ValueKind::Literal,
                "html",
                OWNER_SPAN
            ),
            (
                text.id,
                OperandRole::Text,
                ValueKind::Literal,
                "body",
                BODY_SPAN
            ),
        ]
    );
    assert_eq!(
        program
            .operands
            .iter()
            .filter(|operand| operand.op == for_op.id)
            .count(),
        0
    );
    // The generic validator checks structural tables, not runtime completeness.
    // Actual native product admission must require a genuine original receipt.
    assert_eq!(verify(program), []);
    assert_eq!(lowered.partition.stale(program), None);
    assert_original_custody(file)
}
