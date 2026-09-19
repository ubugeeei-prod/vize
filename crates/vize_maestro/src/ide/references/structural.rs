//! Checker-free navigation over Canon's projection and OXC's lexical symbols.
//! Only exact authored identifier spans participate; unresolved properties and
//! generated helpers cannot turn into spelling-based edits.

use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{SourceType, Span};

mod edits;
mod guard;
mod style;
mod template;
use oxc_syntax::symbol::SymbolId;
use tower_lsp::lsp_types::{Location, Position, Range};
use vize_canon::sfc_typecheck::{
    SfcTypeCheckOptions, type_check_sfc, type_check_sfc_with_legacy_vue2,
    type_check_sfc_with_options_api,
};
use vize_canon::virtual_ts::{VizeMapping, mapping::map_generated_range_to_source};
use vize_s0::FxHashSet;

use crate::ide::{IdeContext, offset_to_position};

struct Occurrence {
    edit: edits::Kind,
    symbol: SymbolId,
    generated: Span,
    authored: Option<(usize, usize)>,
    declaration: bool,
}

fn resolve(ctx: &IdeContext<'_>, new_name: Option<&str>) -> Option<Vec<Occurrence>> {
    if crate::ide::template_scope::needs_structural_pattern_navigation(ctx) {
        return None;
    }
    let options = SfcTypeCheckOptions {
        include_virtual_ts: true,
        experimental_patterned_template: ctx.state.patterned_template_enabled(),
        check_props: false,
        check_emits: false,
        check_template_bindings: false,
        check_reactivity: false,
        check_setup_context: false,
        check_invalid_exports: false,
        check_fallthrough_attrs: false,
        ..SfcTypeCheckOptions::new(ctx.uri.path())
    };
    let generate = if ctx.state.legacy_vue2_enabled() {
        type_check_sfc_with_legacy_vue2
    } else if ctx.state.options_api_enabled() {
        type_check_sfc_with_options_api
    } else {
        type_check_sfc
    };
    let projection = generate(&ctx.content, &options);
    let code = projection.virtual_ts?;
    let allocator = Allocator::default();
    let mut parsed = Parser::new(&allocator, &code, SourceType::ts()).parse();
    if !parsed.diagnostics.is_empty() {
        parsed = Parser::new(&allocator, &code, SourceType::tsx()).parse();
    }
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program)
        .semantic;
    let scoping = semantic.scoping();
    let mut occurrences: Vec<_> = semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let (symbol, span, declaration) = match node.kind() {
                AstKind::BindingIdentifier(id) => (id.symbol_id.get()?, id.span, true),
                AstKind::IdentifierReference(id) => (
                    scoping.get_reference(id.reference_id.get()?).symbol_id()?,
                    id.span,
                    false,
                ),
                _ => return None,
            };
            Some(Occurrence {
                edit: edits::kind(&semantic, node.id()),
                symbol,
                generated: span,
                declaration,
                authored: authored_span(&ctx.content, &code, &projection.virtual_ts_mappings, span),
            })
        })
        .collect();
    let css = style::analyze(ctx, &semantic);
    occurrences.extend(css.occurrences);
    let candidates: FxHashSet<_> = occurrences
        .iter()
        .filter(|entry| {
            entry
                .authored
                .is_some_and(|(start, end)| start <= ctx.offset && ctx.offset <= end)
        })
        .map(|entry| entry.symbol)
        .collect();
    let mut selected = FxHashSet::default();
    selected.insert(*candidates.iter().next()?);
    // Aliases introduced by template unwrapping carry explicit generator links.
    // Never infer one from an identifier's spelling or a synthetic name prefix.
    loop {
        let before = selected.len();
        for link in &projection.virtual_ts_semantic_links {
            let source = symbol_at(&occurrences, &link.source_range);
            let target = symbol_at(&occurrences, &link.target_range);
            if let (Some(source), Some(target)) = (source, target)
                && (selected.contains(&source) || selected.contains(&target))
            {
                selected.insert(source);
                selected.insert(target);
            }
        }
        if before == selected.len() {
            break;
        }
    }
    if let Some(name) = new_name
        && (!guard::rename_is_safe(&semantic, &selected, name)
            || (css.free_names.contains(name)
                && !selected.iter().any(|&id| scoping.symbol_name(id) == name)))
    {
        return None;
    }
    if !candidates.is_subset(&selected) {
        return None;
    }
    Some(
        occurrences
            .into_iter()
            .filter(|entry| selected.contains(&entry.symbol) && entry.authored.is_some())
            .collect(),
    )
}

pub(super) fn references(ctx: &IdeContext<'_>, include_declaration: bool) -> Option<Vec<Location>> {
    let entries = resolve(ctx, None)?;
    let declarations: FxHashSet<_> = entries
        .iter()
        .filter(|entry| entry.declaration)
        .filter_map(|entry| entry.authored)
        .collect();
    let mut spans: Vec<_> = entries
        .into_iter()
        .filter_map(|entry| entry.authored)
        .filter(|span| include_declaration || !declarations.contains(span))
        .collect();
    spans.sort_unstable();
    spans.dedup();
    Some(spans.into_iter().map(|span| location(ctx, span)).collect())
}

pub(in crate::ide) fn rename(
    ctx: &IdeContext<'_>,
    new_name: &str,
) -> Option<Vec<tower_lsp::lsp_types::TextEdit>> {
    let mut changes = std::collections::BTreeMap::new();
    let shorthands = template::shorthands(ctx)?;
    for entry in resolve(ctx, Some(new_name))? {
        let mut span = entry.authored?;
        let original = ctx.content.get(span.0..span.1)?;
        let text = if let Some(directive) = shorthands.get(&span) {
            span = *directive;
            let original = ctx.content.get(span.0..span.1)?;
            vize_s0::cstr!("{original}=\"{new_name}\"").into()
        } else {
            edits::render(entry.edit, original, new_name)
        };
        if let Some(previous) = changes.insert(span, text.clone())
            && previous != text
        {
            return None;
        }
    }
    Some(
        changes
            .into_iter()
            .map(|(span, new_text)| tower_lsp::lsp_types::TextEdit {
                range: location(ctx, span).range,
                new_text,
            })
            .collect(),
    )
}

pub(in crate::ide) fn prepare_rename(ctx: &IdeContext<'_>) -> Option<Range> {
    resolve(ctx, None)?
        .iter()
        .filter_map(|entry| entry.authored)
        .find(|&(start, end)| start <= ctx.offset && ctx.offset <= end)
        .map(|span| location(ctx, span).range)
}

fn location(ctx: &IdeContext<'_>, (start, end): (usize, usize)) -> Location {
    let (line, character) = offset_to_position(&ctx.content, start);
    let (end_line, end_character) = offset_to_position(&ctx.content, end);
    Location {
        uri: ctx.uri.clone(),
        range: Range {
            start: Position { line, character },
            end: Position {
                line: end_line,
                character: end_character,
            },
        },
    }
}

fn symbol_at(entries: &[Occurrence], range: &std::ops::Range<usize>) -> Option<SymbolId> {
    entries
        .iter()
        .find(|entry| {
            entry.generated.start as usize == range.start
                && entry.generated.end as usize == range.end
        })
        .map(|entry| entry.symbol)
}

fn authored_span(
    source: &str,
    code: &str,
    mappings: &[VizeMapping],
    span: Span,
) -> Option<(usize, usize)> {
    let (start, end) = (span.start as usize, span.end as usize);
    let (authored_start, authored_end) = map_generated_range_to_source(mappings, start, end)?;
    (source.get(authored_start..authored_end)? == code.get(start..end)?)
        .then_some((authored_start, authored_end))
}
