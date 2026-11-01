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
pub(super) mod snapshot;
mod style;
mod template;
use oxc_syntax::symbol::SymbolId;
use tower_lsp::lsp_types::{Location, Position, Range};
use vize_canon::sfc_typecheck::{
    SfcTypeCheckOptions, type_check_sfc, type_check_sfc_with_legacy_vue2,
    type_check_sfc_with_options_api,
};
use vize_canon::virtual_ts::{VizeMapping, mapping::map_generated_range_to_source};
use vize_l0::FxHashSet;

use crate::ide::{IdeContext, offset_to_position};

#[derive(Clone)]
struct Occurrence {
    edit: edits::Kind,
    symbol: SymbolId,
    generated: Span,
    authored: Option<(usize, usize)>,
    declaration: bool,
}

fn fresh_projection(ctx: &IdeContext<'_>) -> Option<snapshot::Projection> {
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
    let mode = if ctx.state.legacy_vue2_enabled() {
        snapshot::Mode::LegacyVue2
    } else if ctx.state.options_api_enabled() {
        snapshot::Mode::OptionsApi
    } else {
        snapshot::Mode::Standard
    };
    let generate = match mode {
        snapshot::Mode::LegacyVue2 => type_check_sfc_with_legacy_vue2,
        snapshot::Mode::OptionsApi => type_check_sfc_with_options_api,
        snapshot::Mode::Standard => type_check_sfc,
    };
    let projection = generate(&ctx.content, &options);
    let code = projection.virtual_ts?;
    Some(snapshot::Projection {
        filename: options.filename,
        mode,
        patterned: options.experimental_patterned_template,
        code,
        mappings: projection.virtual_ts_mappings,
        links: projection.virtual_ts_semantic_links,
    })
}

fn analyze_projection(
    ctx: &IdeContext<'_>,
    projection: &snapshot::Projection,
) -> Option<snapshot::Facts> {
    let code = &projection.code;
    let allocator = Allocator::default();
    let mut parsed = Parser::new(&allocator, code, SourceType::ts()).parse();
    if !parsed.diagnostics.is_empty() {
        parsed = Parser::new(&allocator, code, SourceType::tsx()).parse();
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
                authored: authored_span(&ctx.content, code, &projection.mappings, span),
            })
        })
        .collect();
    let css = style::analyze(ctx, &semantic);
    occurrences.extend(css.occurrences);
    let references = semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::IdentifierReference(id) = node.kind() else {
                return None;
            };
            Some(snapshot::NamedReference {
                name: id.name.as_str().into(),
                reference: id.reference_id.get()?,
            })
        })
        .collect();
    Some(snapshot::Facts {
        scoping: semantic.into_scoping(),
        occurrences,
        references,
        css_free_names: css
            .free_names
            .into_iter()
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    })
}

fn resolve(ctx: &IdeContext<'_>, new_name: Option<&str>) -> Option<Vec<Occurrence>> {
    let entry = resolve_entry(ctx)?;
    select(ctx, new_name, &entry)
}

fn resolve_entry(ctx: &IdeContext<'_>) -> Option<std::sync::Arc<snapshot::Entry>> {
    if crate::ide::template_scope::needs_structural_pattern_navigation(ctx) {
        return None;
    }
    // Type resolution and all external inputs stay fresh. Only the lexical
    // artifact downstream of this complete current projection can be reused.
    let projection = fresh_projection(ctx)?;
    let owner = ctx
        .state
        .documents
        .structural_snapshot_owner(ctx.uri, &ctx.content);
    let entry = if let Some(cached) = owner.as_ref().and_then(|owner| owner.get(&projection)) {
        cached
    } else {
        let facts = analyze_projection(ctx, &projection)?;
        match owner {
            Some(owner) => owner.publish(projection, facts),
            None => std::sync::Arc::new(snapshot::Entry::uncached(
                std::sync::Arc::clone(&ctx.content),
                ctx.uri.clone(),
                projection,
                facts,
            )),
        }
    };
    Some(entry)
}

fn select(
    ctx: &IdeContext<'_>,
    new_name: Option<&str>,
    entry: &snapshot::Entry,
) -> Option<Vec<Occurrence>> {
    let occurrences = &entry.facts.occurrences;
    let scoping = &entry.facts.scoping;
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
        for link in &entry.projection.links {
            let source = symbol_at(occurrences, &link.source_range);
            let target = symbol_at(occurrences, &link.target_range);
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
        && (!guard::rename_is_safe(scoping, &entry.facts.references, &selected, name)
            || (entry.facts.css_free_names.iter().any(|free| free == name)
                && !selected.iter().any(|&id| scoping.symbol_name(id) == name)))
    {
        return None;
    }
    if !candidates.is_subset(&selected) {
        return None;
    }
    Some(
        occurrences
            .iter()
            .filter(|entry| selected.contains(&entry.symbol) && entry.authored.is_some())
            .cloned()
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
    render_rename(ctx, new_name, || resolve(ctx, Some(new_name)))
}

/// Prove the selected lexical/link component contains this exact authored
/// top-level declaration, using the same Entry and full guard as ordinary rename.
#[cfg(feature = "native")]
pub(in crate::ide) fn private_setup_rename(
    ctx: &IdeContext<'_>,
    new_name: &str,
    name: &str,
    declaration: (usize, usize),
) -> Option<Vec<tower_lsp::lsp_types::TextEdit>> {
    let entry = resolve_entry(ctx)?;
    let selected = private_setup_component(ctx, Some(new_name), &entry, name, declaration)?;
    render_rename(ctx, new_name, || Some(selected))
}

#[cfg(feature = "native")]
fn private_setup_component(
    ctx: &IdeContext<'_>,
    new_name: Option<&str>,
    entry: &snapshot::Entry,
    name: &str,
    declaration: (usize, usize),
) -> Option<Vec<Occurrence>> {
    let mut declarations = entry
        .facts
        .occurrences
        .iter()
        .filter(|occurrence| occurrence.declaration && occurrence.authored == Some(declaration))
        .map(|occurrence| occurrence.symbol);
    let symbol = declarations.next()?;
    if declarations.any(|other| other != symbol) || entry.facts.scoping.symbol_name(symbol) != name
    {
        return None;
    }
    let selected = select(ctx, new_name, entry)?;
    // select starts from the actual cursor symbols and closes only explicit
    // generator links. A same-spelling shadow cannot supply this declaration.
    if !selected.iter().any(|occurrence| {
        occurrence.symbol == symbol
            && occurrence.declaration
            && occurrence.authored == Some(declaration)
    }) {
        return None;
    }
    Some(selected)
}

/// One exact half-open query token in the same private declaration component.
/// Every retained span must be plain authored text, including the declaration.
#[cfg(feature = "native")]
pub(in crate::ide) fn private_setup_prepare(
    ctx: &IdeContext<'_>,
    name: &str,
    declaration: (usize, usize),
) -> Option<(Range, Vec<tower_lsp::lsp_types::TextEdit>)> {
    let entry = resolve_entry(ctx)?;
    let selected = private_setup_component(ctx, None, &entry, name, declaration)?;
    if selected.iter().any(|occurrence| {
        entry
            .facts
            .scoping
            .get_resolved_references(occurrence.symbol)
            .any(|reference| reference.is_write())
    }) {
        return None;
    }
    let shorthands = template::shorthands(ctx)?;
    let mut spans = Vec::with_capacity(selected.len());
    for occurrence in selected {
        let span = occurrence.authored?;
        if !matches!(occurrence.edit, edits::Kind::Identifier)
            || shorthands.contains_key(&span)
            || span.0 >= span.1
            || ctx.content.get(span.0..span.1)? != name
        {
            return None;
        }
        let range = location(ctx, span).range;
        if crate::ide::position_to_offset(&ctx.content, range.start.line, range.start.character)?
            != span.0
            || crate::ide::position_to_offset(&ctx.content, range.end.line, range.end.character)?
                != span.1
        {
            return None;
        }
        spans.push(span);
    }
    spans.sort_unstable();
    spans.dedup();
    if !spans.contains(&declaration)
        || spans.windows(2).any(|pair| {
            pair.first()
                .zip(pair.get(1))
                .is_none_or(|(left, right)| left.1 > right.0)
        })
    {
        return None;
    }
    let mut query = spans
        .iter()
        .filter(|&&(start, end)| start <= ctx.offset && ctx.offset < end);
    let range = location(ctx, *query.next()?).range;
    if query.next().is_some() {
        return None;
    }
    Some((
        range,
        spans
            .into_iter()
            .map(|span| tower_lsp::lsp_types::TextEdit {
                range: location(ctx, span).range,
                new_text: name.into(),
            })
            .collect(),
    ))
}

fn render_rename(
    ctx: &IdeContext<'_>,
    new_name: &str,
    resolve_entries: impl FnOnce() -> Option<Vec<Occurrence>>,
) -> Option<Vec<tower_lsp::lsp_types::TextEdit>> {
    let mut changes = std::collections::BTreeMap::new();
    let shorthands = template::shorthands(ctx)?;
    for entry in resolve_entries()? {
        let mut span = entry.authored?;
        let original = ctx.content.get(span.0..span.1)?;
        let text = if let Some(directive) = shorthands.get(&span) {
            span = *directive;
            let original = ctx.content.get(span.0..span.1)?;
            vize_l0::cstr!("{original}=\"{new_name}\"").into()
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
