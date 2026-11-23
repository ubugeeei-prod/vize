//! Compare resident graph reads for original Vue documents and editor revisions
//! against a fresh lowering of the bytes selected in that revision.
use std::{fs, path::Path};
use vize_incremental::{
    BlockKind, ParsedSfc, ResidentDatabase, SourceFile, block_query::BlockView,
    descriptor::parse_outcome,
};
use vize_l0::{Allocator, Lang, Span};
use vize_l1_to_l2::native::lower_component_native;
use vize_l3::{
    L3Ir,
    op::OpId,
    query::{Direction, EdgeKinds, GraphError, OpGraph, ReachQuery, check_reachability},
};
use vize_resident_graph::{TemplateGraphError, TemplateQueries, TemplateReachBatch};

#[derive(Debug, Default, PartialEq, Eq)]
struct Counts {
    unavailable: usize,
    construction_errors: usize,
    incomplete: usize,
    holes: usize,
    diagnostics: usize,
    accepted: usize,
    projected: usize,
    checked_answers: usize,
    witnessed: usize,
}

fn request(start: OpId, direction: Direction) -> ReachQuery {
    ReachQuery {
        start,
        direction,
        kinds: EdgeKinds::ALL,
    }
}

fn position(block: BlockView<'_>, source: &str, relative: Span) -> Span {
    let base = block.start();
    let expected = Span::new(
        base.checked_add(relative.start).unwrap(),
        base.checked_add(relative.end).unwrap(),
    );
    let bytes = block
        .source()
        .text
        .get(relative.start as usize..relative.end as usize)
        .unwrap();
    assert_eq!(
        source.get(expected.start as usize..expected.end as usize),
        Some(bytes)
    );
    assert_eq!(block.authored_span(relative), Some(expected));
    expected
}

fn checked_graph(
    source: &str,
    block: BlockView<'_>,
    lang: Lang,
    ir: &L3Ir<'_>,
    batch: TemplateReachBatch<'_>,
    counts: &mut Counts,
) {
    let original = OpGraph::new(ir).unwrap();
    let sites = batch.source_sites();
    for op in &ir.ops {
        let span = position(block, source, op.span);
        assert_eq!(batch.authored_span(op.id), Ok(span));
        let (projected, text) = sites.get(op.id).unwrap();
        assert_eq!(projected, span);
        assert_eq!(
            Some(text),
            source.get(span.start as usize..span.end as usize)
        );
        let relative = op.span.start as usize..op.span.end as usize;
        assert!(std::ptr::eq(text, &block.source().text[relative]));
    }
    counts.projected += ir.ops.len();
    // A start outside the source graph is still invalid on a structurally complete template.
    let missing = OpId::new(u32::MAX);
    assert!(original.operation(missing).is_none());
    let absent = request(missing, Direction::Forward);
    let expected = TemplateGraphError::Graph(GraphError::UnknownOp(missing));
    assert_eq!(sites.get(missing).as_ref().err(), Some(&expected));
    assert_eq!(block.reachable(lang, absent).err(), Some(&expected));
    assert_eq!(batch.reachable(absent).err(), Some(expected));

    if ir.ops.is_empty() {
        return;
    }
    let mut previous = None;
    for index in [0, ir.ops.len() / 2, ir.ops.len() - 1] {
        if previous == Some(index) {
            continue;
        }
        previous = Some(index);
        for direction in [Direction::Forward, Direction::Reverse] {
            let query = request(ir.ops[index].id, direction);
            let saved = block.reachable(lang, query).unwrap();
            let streamed = batch.reachable(query).unwrap();
            assert_eq!(saved.relative(), streamed.relative());
            let steps = saved.relative().steps();
            assert_eq!(
                batch.check_singleton(query),
                check_reachability(ir, query, &steps[..1])
            );
            assert_eq!(check_reachability(ir, query, steps), Ok(()));
            assert_eq!(steps, original.reach_with(query, |_| {}).unwrap());
            assert_eq!(saved.authored_spans().len(), steps.len());
            for (step, row) in steps.iter().zip(saved.authored_spans()) {
                let span = original.operation(step.op).unwrap().span;
                assert_eq!(row, Ok((step.op, position(block, source, span))));
            }
            counts.checked_answers += 1;
            counts.witnessed += steps.len();
        }
    }
}

fn selected(source: &str, block: BlockView<'_>, lang: Lang, counts: &mut Counts) {
    let batch = block.reach_batch(lang);
    let query = request(OpId::new(0), Direction::Forward);
    let verify_refusal = |error: &TemplateGraphError, batch: Option<&TemplateGraphError>| {
        assert_eq!(batch, Some(error));
        assert_eq!(block.reachable(lang, query).err(), Some(error));
    };
    if block.template_input().is_none() {
        verify_refusal(&TemplateGraphError::UnsupportedBlock, batch.err());
        counts.unavailable += 1;
        return;
    }
    let text = block.source().text.as_str();
    position(
        block,
        source,
        Span::new(0, u32::try_from(text.len()).unwrap()),
    );
    let arena = Allocator::default();
    let native = match lower_component_native(&arena, text, lang) {
        Ok(native) => native,
        Err(rejected) => {
            verify_refusal(
                &TemplateGraphError::Construction(rejected.error),
                batch.err(),
            );
            counts.construction_errors += 1;
            return;
        }
    };
    if !native.is_supported() || !native.diagnostics.is_empty() {
        for hole in &native.holes {
            position(block, source, hole.span);
        }
        for diagnostic in &native.diagnostics {
            position(block, source, diagnostic.span);
            for part in &diagnostic.parts {
                position(block, source, part.span);
            }
            if let Some(chain) = diagnostic.witness_chain() {
                for link in chain.links() {
                    position(block, source, link.span);
                }
            }
        }
        counts.incomplete += 1;
        counts.holes += native.holes.len();
        counts.diagnostics += native.diagnostics.len();
        verify_refusal(
            &TemplateGraphError::Incomplete {
                holes: native.holes,
                diagnostics: native.diagnostics,
            },
            batch.err(),
        );
        return;
    }
    counts.accepted += 1;
    let native = vize_l2_to_l3::lower(&arena, native.artifact.root());
    checked_graph(source, block, lang, native.l3ir(), batch.unwrap(), counts);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct TemplateSite {
    start: u32,
    len: usize,
    inline: bool,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Revision {
    template: Option<TemplateSite>,
    counts: Counts,
    descriptor_refused: bool,
}

fn revision(db: &ResidentDatabase, file: SourceFile, path: &str, text: &str) -> Revision {
    let mut revision = Revision::default();
    let blocks = db.document_blocks(file);
    let original = match parse_outcome(path, text) {
        ParsedSfc::Failed(error) => {
            assert_eq!(blocks.err(), Some(&error), "{path}");
            revision.descriptor_refused = true;
            return revision;
        }
        ParsedSfc::Descriptor(original) => original,
    };
    let blocks = blocks.unwrap();
    let mut templates = blocks.iter().filter(|b| b.kind() == &BlockKind::Template);
    let Some(template) = &original.template else {
        assert!(templates.next().is_none(), "{path}");
        return revision;
    };
    let block = templates.next().unwrap();
    assert!(templates.next().is_none(), "{path}");
    assert_eq!(block.start() as usize, template.loc.start, "{path}");
    assert_eq!(
        block.source().text.as_str(),
        template.content.as_ref(),
        "{path}"
    );
    let inline = !template.attrs.contains_key("src")
        && template
            .lang
            .as_deref()
            .is_none_or(|s| s.is_empty() || s.eq_ignore_ascii_case("html"));
    assert_eq!(block.template_input().is_some(), inline, "{path}");
    revision.template = Some(TemplateSite {
        start: block.start(),
        len: template.content.len(),
        inline,
    });
    for lang in [Lang::Js, Lang::Ts] {
        selected(text, block, lang, &mut revision.counts);
    }
    revision
}

#[test]
#[ignore = "provide the NUL-separated tracked or upstream VIZE_IR_GRAPH_INPUTS"]
fn graph_answers_across_original_source_edits() {
    let manifest = fs::read(std::env::var_os("VIZE_IR_GRAPH_INPUTS").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let mut totals = Counts::default();
    let (mut files, mut invalid_utf8, mut damaged, mut refused_descriptors) = (0, 0, 0, 0);
    for entry in manifest.split(|&b| b == 0).filter(|p| !p.is_empty()) {
        let path = std::str::from_utf8(entry).unwrap();
        let bytes = fs::read(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"));
        files += 1;
        let Ok(text) = std::str::from_utf8(&bytes) else {
            invalid_utf8 += 1;
            continue;
        };
        let mut db = ResidentDatabase::default();
        let file = db.open(path, text);
        let original = revision(&db, file, path, text);
        refused_descriptors += usize::from(original.descriptor_refused);
        drop(db.take_accounting());
        const PREFIX: &str = "<!--編集中🌸-->\n";
        let mut moved = text.to_owned();
        let after_bom = text
            .strip_prefix('\u{feff}')
            .map_or(0, |_| '\u{feff}'.len_utf8());
        moved.insert_str(after_bom, PREFIX);
        db.edit(file, &moved);
        let moved_revision = revision(&db, file, path, &moved);
        assert_eq!(original.counts, moved_revision.counts, "{path}");
        let shifted = original.template.map(|site| TemplateSite {
            start: site.start + PREFIX.len() as u32,
            ..site
        });
        assert_eq!(shifted, moved_revision.template, "{path}");
        for (ingredient, counts) in db.take_accounting() {
            if ingredient.contains("template_graph") || ingredient.contains("template_reach") {
                assert_eq!(counts.executed, 0, "{path} {ingredient}");
            }
        }
        // A self-closing template has no authored body to edit.
        if let Some(site) = original.template.filter(|site| site.inline && site.len > 0) {
            let mut broken = text.to_owned();
            broken.insert_str(site.start as usize, "{{(}}");
            db.edit(file, &broken);
            let current = revision(&db, file, path, &broken);
            assert!(current.counts.incomplete >= 2, "{path}");
            // Typing a valid expression of the same byte length must invalidate
            // its previous holes even if the rest of this template is still broken.
            let expression = site.start as usize + 2;
            broken.replace_range(expression..expression + 1, "0");
            db.edit(file, &broken);
            revision(&db, file, path, &broken);
            db.edit(file, text);
            assert_eq!(original, revision(&db, file, path, text), "{path}");
            damaged += 1;
        }
        let count = original.counts;
        totals.unavailable += count.unavailable;
        totals.construction_errors += count.construction_errors;
        totals.incomplete += count.incomplete;
        totals.holes += count.holes;
        totals.diagnostics += count.diagnostics;
        totals.accepted += count.accepted;
        totals.projected += count.projected;
        totals.checked_answers += count.checked_answers;
        totals.witnessed += count.witnessed;
    }
    assert!(damaged > 0 && totals.accepted > 0 && totals.incomplete > 0 && totals.witnessed > 0);
    println!(
        "files={files} invalid_utf8={invalid_utf8} refused_descriptors={refused_descriptors} damaged_and_restored={damaged} original={totals:?}"
    );
}
