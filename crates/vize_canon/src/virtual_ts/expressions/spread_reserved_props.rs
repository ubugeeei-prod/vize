//! Syntax-aware reserved template-prop rewriting for `v-bind="expression"`.
//!
//! The general template-expression rewriter predates spread checks and is a
//! deliberately small scanner. A whole spread can contain object methods,
//! comments, TypeScript types, operators, and nested functions, so this path
//! asks OXC for actual unresolved value-reference spans instead of guessing
//! from token adjacency.

use super::reserved_props::{is_identifier_continue, is_identifier_start};
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use std::ops::Range;

mod references;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use references::ReferenceCollector;
use vize_carton::{FxHashSet, String, append};
use vize_croquis::{ScopeChain, ScopeId};

const PARSE_PREFIX: &str = "const __vize_spread = (";
const PARSE_SUFFIX: &str = ");";

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BindingRewrite {
    pub(crate) code: String,
    pub(crate) segments: Vec<SpreadRewriteSegment>,
    /// Relative receiver-only anchors. They carry diagnostics, never symbol identity.
    pub(crate) diagnostic_prefixes: Vec<SpreadRewriteSegment>,
}

pub(super) type SpreadRewrite = BindingRewrite;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SpreadRewriteSegment {
    pub(crate) generated: Range<usize>,
    pub(crate) source: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Replacement {
    source: Range<usize>,
    /// OXC's actual normalized identifier, separate from its authored spelling.
    name: String,
    /// A normalized value binding actually visible at this particular read.
    receiver_shadowed: bool,
    shorthand: bool,
    authored_callee: bool,
}

/// Parsed relative references and actual semantic names. No AST or allocator
/// reference crosses this boundary; final receiver spelling is selected later.
#[derive(Debug, Default)]
pub(crate) struct ParsedInstanceReferences {
    replacements: Vec<Replacement>,
    pub(crate) normalized_names: FxHashSet<String>,
}

impl ParsedInstanceReferences {
    pub(crate) fn has_instance_references(&self) -> bool {
        !self.replacements.is_empty()
    }

    pub(crate) fn receiver_is_shadowed(&self) -> bool {
        self.replacements.iter().any(|row| row.receiver_shadowed)
    }
}

/// Returns `None` without constructing a parser or output buffer when the
/// expression contains no reserved template-prop spelling at all.
pub(super) fn rewrite_reserved_spread_references(
    expression: &str,
    template_binding_access: &TemplateBindingAccess,
    template_scopes: &ScopeChain,
    usage_scope_id: ScopeId,
) -> Option<SpreadRewrite> {
    if !has_reserved_name_candidate(expression, template_binding_access) {
        return None;
    }

    let parsed = collect_replacements(
        expression,
        template_binding_access,
        template_scopes,
        usage_scope_id,
        false,
    );
    let replacements = parsed.replacements;
    if replacements.is_empty() {
        return None;
    }
    Some(apply_replacements(
        expression,
        &replacements,
        template_binding_access,
    ))
}

pub(crate) fn prepare_instance_references(
    expression: &str,
    bindings: &TemplateBindingAccess,
) -> ParsedInstanceReferences {
    let Some(summary) = bindings.instance_summary() else {
        return ParsedInstanceReferences::default();
    };
    collect_replacements(
        expression,
        bindings,
        &summary.scopes,
        bindings.scope_id(),
        true,
    )
}

pub(crate) fn materialize_instance_references(
    expression: &str,
    parsed: ParsedInstanceReferences,
    bindings: &TemplateBindingAccess,
) -> Option<BindingRewrite> {
    (!parsed.replacements.is_empty())
        .then(|| apply_replacements(expression, &parsed.replacements, bindings))
}

fn has_reserved_name_candidate(
    expression: &str,
    template_binding_access: &TemplateBindingAccess,
) -> bool {
    if template_binding_access.is_empty() {
        return false;
    }
    let bytes = expression.as_bytes();
    let mut cursor = 0;
    while let Some(&byte) = bytes.get(cursor) {
        if !is_identifier_start(byte) {
            cursor += 1;
            continue;
        }
        let start = cursor;
        cursor += 1;
        while bytes
            .get(cursor)
            .copied()
            .is_some_and(is_identifier_continue)
        {
            cursor += 1;
        }
        if template_binding_access.contains(expression.get(start..cursor).unwrap_or_default()) {
            return true;
        }
    }
    false
}

fn collect_replacements(
    expression: &str,
    template_binding_access: &TemplateBindingAccess,
    template_scopes: &ScopeChain,
    usage_scope_id: ScopeId,
    only_instance: bool,
) -> ParsedInstanceReferences {
    let expression_result = collect_with_wrapper(
        expression,
        template_binding_access,
        template_scopes,
        usage_scope_id,
        only_instance,
        (PARSE_PREFIX, PARSE_SUFFIX),
    );
    if let Some(result) = expression_result {
        return result;
    }
    // Event handlers can contain several statements, local types and returns.
    // This is a second grammar attempt, never a spelling-based fallback.
    if only_instance {
        return collect_with_wrapper(
            expression,
            template_binding_access,
            template_scopes,
            usage_scope_id,
            true,
            ("function __vize_handler() {\n", "\n}"),
        )
        .unwrap_or_default();
    }
    ParsedInstanceReferences::default()
}

fn collect_with_wrapper(
    expression: &str,
    template_binding_access: &TemplateBindingAccess,
    template_scopes: &ScopeChain,
    usage_scope_id: ScopeId,
    only_instance: bool,
    wrapper: (&str, &str),
) -> Option<ParsedInstanceReferences> {
    let (prefix, suffix) = wrapper;
    if only_instance {
        vize_carton::profiler::global_profiler()
            .record_counter("canon.instance_global_rewrite.source_buffers", 1);
    }
    let mut source = String::with_capacity(prefix.len() + expression.len() + suffix.len());
    source.push_str(prefix);
    source.push_str(expression);
    source.push_str(suffix);

    if only_instance {
        vize_carton::profiler::global_profiler()
            .record_counter("canon.instance_global_rewrite.parse_attempts", 1);
    }

    let allocator = oxc_allocator::Allocator::default();
    let parsed = Parser::new(
        &allocator,
        source.as_str(),
        SourceType::ts().with_module(true),
    )
    .parse();
    if !parsed.diagnostics.is_empty() {
        return None;
    }
    let semantic = SemanticBuilder::new().build(&parsed.program);
    if !semantic.diagnostics.is_empty() {
        return None;
    }

    let mut collector = ReferenceCollector {
        expression_len: expression.len(),
        offset: prefix.len() as u32,
        replacements: Vec::new(),
        scoping: semantic.semantic.scoping(),
        template_binding_access,
        template_scopes,
        usage_scope_id,
        only_instance,
        callees: FxHashSet::default(),
    };
    collector.visit_program(&parsed.program);
    collector
        .replacements
        .sort_by_key(|replacement| replacement.source.start);
    collector
        .replacements
        .dedup_by(|right, left| right.source == left.source);
    let scoping = semantic.semantic.scoping();
    let mut normalized_names = FxHashSet::default();
    if only_instance {
        // Symbols include escaped binders; unresolved names include authored
        // value/type-query reads that a new generated alias must never bind.
        normalized_names.extend(
            scoping
                .symbol_ids()
                .map(|id| String::from(scoping.symbol_name(id))),
        );
        normalized_names.extend(
            scoping
                .root_unresolved_references()
                .keys()
                .map(|name| String::from(name.as_str())),
        );
    }
    Some(ParsedInstanceReferences {
        replacements: collector.replacements,
        normalized_names,
    })
}

fn apply_replacements(
    expression: &str,
    replacements: &[Replacement],
    bindings: &TemplateBindingAccess,
) -> SpreadRewrite {
    let extra_capacity = replacements.len().saturating_mul(16);
    let mut code = String::with_capacity(expression.len().saturating_add(extra_capacity));
    let mut segments = Vec::with_capacity(replacements.len().saturating_mul(2) + 1);
    let mut diagnostic_prefixes = Vec::new();
    let mut source_cursor = 0;

    for replacement in replacements {
        push_source_segment(
            expression,
            source_cursor..replacement.source.start,
            &mut code,
            &mut segments,
        );
        if replacement.authored_callee {
            let Some((name, receiver)) = expression
                .get(replacement.source.clone())
                .zip(bindings.authored_callee_receiver())
            else {
                source_cursor = replacement.source.start;
                continue;
            };
            let receiver_start = code.len();
            append!(
                code,
                "({receiver} as unknown as (\"{key}\" extends keyof typeof {receiver} ? typeof {receiver} : {{ \"{key}\": any }})).",
                key = replacement.name
            );
            let key_start = code.len();
            diagnostic_prefixes.push(SpreadRewriteSegment {
                generated: receiver_start..key_start,
                source: replacement.source.clone(),
            });
            code.push_str(name);
            segments.push(SpreadRewriteSegment {
                generated: key_start..code.len(),
                source: replacement.source.clone(),
            });
            source_cursor = replacement.source.end;
            continue;
        }
        let Some((name, receiver)) = expression
            .get(replacement.source.clone())
            .and_then(|name| Some((name, bindings.receiver(replacement.name.as_str())?)))
        else {
            // Keep the authored text: the next source segment copies it.
            source_cursor = replacement.source.start;
            continue;
        };
        let generated_start = code.len();
        if bindings.is_instance_global(replacement.name.as_str()) {
            if replacement.shorthand {
                code.push_str(name);
                segments.push(SpreadRewriteSegment {
                    generated: generated_start..code.len(),
                    source: replacement.source.clone(),
                });
                code.push_str(": ");
            }
            let receiver_start = code.len();
            code.push_str(receiver);
            code.push('.');
            let key_start = code.len();
            diagnostic_prefixes.push(SpreadRewriteSegment {
                generated: receiver_start..key_start,
                source: replacement.source.clone(),
            });
            code.push_str(name);
            segments.push(SpreadRewriteSegment {
                generated: key_start..code.len(),
                source: replacement.source.clone(),
            });
            source_cursor = replacement.source.end;
            continue;
        } else if replacement.shorthand {
            append!(code, "{name}: {receiver}[\"{name}\"]");
        } else {
            append!(code, "{receiver}[\"{name}\"]");
        }
        segments.push(SpreadRewriteSegment {
            generated: generated_start..code.len(),
            source: replacement.source.clone(),
        });
        source_cursor = replacement.source.end;
    }
    push_source_segment(
        expression,
        source_cursor..expression.len(),
        &mut code,
        &mut segments,
    );
    SpreadRewrite {
        code,
        segments,
        diagnostic_prefixes,
    }
}

fn push_source_segment(
    expression: &str,
    source: Range<usize>,
    code: &mut String,
    segments: &mut Vec<SpreadRewriteSegment>,
) {
    let Some(text) = expression
        .get(source.clone())
        .filter(|text| !text.is_empty())
    else {
        return;
    };
    let generated_start = code.len();
    code.push_str(text);
    segments.push(SpreadRewriteSegment {
        generated: generated_start..code.len(),
        source,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn props(names: &[&str]) -> TemplateBindingAccess<'static> {
        TemplateBindingAccess::from_props(names.iter().copied().map(Into::into).collect())
    }

    fn rewrite(expression: &str, names: &TemplateBindingAccess) -> Option<SpreadRewrite> {
        rewrite_reserved_spread_references(expression, names, &ScopeChain::new(), ScopeId::ROOT)
    }

    #[test]
    fn no_reserved_spelling_takes_the_parser_free_fast_path() {
        let names = props(&["as"]);
        assert!(!has_reserved_name_candidate("{ value: bag.count }", &names));
        assert_eq!(rewrite("{ value: bag.count }", &names), None);
    }

    #[test]
    fn rewrites_only_unbound_value_references() {
        let expression = "{ as, tag: as, member: item.as, computed: item[as], template: `${as}`, cast: count as /* as */ number, kind: typeof count, local: ((as) => as)('x') }";
        let rewritten = rewrite(expression, &props(&["as", "typeof"]))
            .expect("unbound references should be rewritten");

        assert_eq!(
            rewritten.code,
            "{ as: props[\"as\"], tag: props[\"as\"], member: item.as, computed: item[props[\"as\"]], template: `${props[\"as\"]}`, cast: count as /* as */ number, kind: typeof count, local: ((as) => as)('x') }",
        );
    }

    #[test]
    fn preserves_commented_generic_method_accessor_generator_and_async_keys() {
        let expression = "{ /* as */ as /* key */ <T>(value: T) { return value }, get as() { return 'x' }, set as(value) {}, *as<T>() { yield 1 }, async as<T>() {}, async *as<T>() { yield 1 } }";
        assert_eq!(rewrite(expression, &props(&["as"])), None);
    }

    #[test]
    fn preserves_reserved_operators_comments_and_type_references() {
        let expression = "{ type: typeof value, member: key in value, check: value instanceof Type, made: new Type(), cast: value as typeof as, run: async function* () { await value; yield value } }";
        let names = props(&["typeof", "in", "instanceof", "new", "as", "await", "yield"]);
        assert_eq!(rewrite(expression, &names), None);
    }

    #[test]
    fn rewrite_segments_keep_offsets_after_an_expanded_shorthand_exact() {
        let expression = "{ as, value: bag.missing }";
        let rewritten =
            rewrite(expression, &props(&["as"])).expect("shorthand should be rewritten");
        let source_missing = expression.find("missing").unwrap();
        let generated_missing = rewritten.code.find("missing").unwrap();
        let segment = rewritten
            .segments
            .iter()
            .find(|segment| segment.generated.contains(&generated_missing))
            .expect("missing must have an unchanged mapping segment");

        assert_eq!(
            generated_missing - segment.generated.start,
            source_missing - segment.source.start,
        );
    }
}
