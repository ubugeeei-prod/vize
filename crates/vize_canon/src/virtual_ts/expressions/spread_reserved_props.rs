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

use oxc_ast::ast::{IdentifierReference, ObjectProperty};
use oxc_ast_visit::{Visit, walk::walk_object_property};
use oxc_parser::Parser;
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_span::SourceType;
use vize_carton::{String, append};
use vize_croquis::{ScopeChain, ScopeId, ScopeKind};

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
    shorthand: bool,
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

    let replacements = collect_replacements(
        expression,
        template_binding_access,
        template_scopes,
        usage_scope_id,
        false,
    );
    if replacements.is_empty() {
        return None;
    }
    Some(apply_replacements(
        expression,
        &replacements,
        template_binding_access,
    ))
}

pub(super) fn rewrite_instance_references(
    expression: &str,
    bindings: &TemplateBindingAccess,
) -> Option<BindingRewrite> {
    let summary = bindings.instance_summary()?;
    let replacements = collect_replacements(
        expression,
        bindings,
        &summary.scopes,
        bindings.scope_id(),
        true,
    );
    (!replacements.is_empty()).then(|| apply_replacements(expression, &replacements, bindings))
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
) -> Vec<Replacement> {
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
    Vec::new()
}

fn collect_with_wrapper(
    expression: &str,
    template_binding_access: &TemplateBindingAccess,
    template_scopes: &ScopeChain,
    usage_scope_id: ScopeId,
    only_instance: bool,
    wrapper: (&str, &str),
) -> Option<Vec<Replacement>> {
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
    };
    collector.visit_program(&parsed.program);
    collector
        .replacements
        .sort_by_key(|replacement| replacement.source.start);
    collector
        .replacements
        .dedup_by(|right, left| right.source == left.source);
    Some(collector.replacements)
}

struct ReferenceCollector<'a> {
    expression_len: usize,
    offset: u32,
    replacements: Vec<Replacement>,
    scoping: &'a Scoping,
    template_binding_access: &'a TemplateBindingAccess<'a>,
    template_scopes: &'a ScopeChain,
    usage_scope_id: ScopeId,
    only_instance: bool,
}

impl ReferenceCollector<'_> {
    fn replacement(&self, ident: &IdentifierReference<'_>, shorthand: bool) -> Option<Replacement> {
        if !self.template_binding_access.contains(ident.name.as_str()) {
            return None;
        }
        let instance = self
            .template_binding_access
            .is_instance_global(ident.name.as_str());
        if self.only_instance && !instance {
            return None;
        }
        if is_visible_template_binding(
            self.template_scopes,
            self.usage_scope_id,
            ident.name.as_str(),
        ) {
            return None;
        }
        let reference_id = ident.reference_id.get()?;
        let reference = self.scoping.get_reference(reference_id);
        if reference.symbol_id().is_some()
            || (reference.flags().is_type() && !reference.flags().is_value_as_type())
            || (reference.flags().is_value_as_type() && !instance)
        {
            return None;
        }

        let start = ident.span.start.checked_sub(self.offset)? as usize;
        let end = ident.span.end.checked_sub(self.offset)? as usize;
        (start < end && end <= self.expression_len).then_some(Replacement {
            source: start..end,
            shorthand,
        })
    }
}

fn is_visible_template_binding(scopes: &ScopeChain, start: ScopeId, name: &str) -> bool {
    let mut current = Some(start);
    while let Some(id) = current {
        let Some(scope) = scopes.get_scope(id) else {
            return false;
        };
        if matches!(
            scope.kind,
            ScopeKind::VFor | ScopeKind::VSlot | ScopeKind::EventHandler
        ) && scope.has_binding(name)
        {
            return true;
        }
        current = scope.parent();
    }
    false
}

impl<'a> Visit<'a> for ReferenceCollector<'_> {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if let Some(replacement) = self.replacement(ident, false) {
            self.replacements.push(replacement);
        }
    }

    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        if property.shorthand
            && let oxc_ast::ast::Expression::Identifier(ident) = &property.value
            && let Some(replacement) = self.replacement(ident, true)
        {
            self.replacements.push(replacement);
            return;
        }
        walk_object_property(self, property);
    }
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
        let Some((name, receiver)) = expression
            .get(replacement.source.clone())
            .and_then(|name| Some((name, bindings.receiver(name)?)))
        else {
            // Keep the authored text: the next source segment copies it.
            source_cursor = replacement.source.start;
            continue;
        };
        let generated_start = code.len();
        if bindings.is_instance_global(name) {
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
