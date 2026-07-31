//! Syntax-aware reserved template-prop rewriting for `v-bind="expression"`.
//!
//! The general template-expression rewriter predates spread checks and is a
//! deliberately small scanner. A whole spread can contain object methods,
//! comments, TypeScript types, operators, and nested functions, so this path
//! asks OXC for actual unresolved value-reference spans instead of guessing
//! from token adjacency.

use std::ops::Range;

use oxc_ast::ast::{IdentifierReference, ObjectProperty};
use oxc_ast_visit::{Visit, walk::walk_object_property};
use oxc_parser::Parser;
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_span::SourceType;
use vize_carton::{FxHashSet, String, append};
use vize_croquis::{ScopeChain, ScopeId, ScopeKind};

const PARSE_PREFIX: &str = "const __vize_spread = (";
const PARSE_SUFFIX: &str = ");";

#[derive(Debug, PartialEq, Eq)]
pub(super) struct SpreadRewrite {
    pub(super) code: String,
    pub(super) segments: Vec<SpreadRewriteSegment>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct SpreadRewriteSegment {
    pub(super) generated: Range<usize>,
    pub(super) source: Range<usize>,
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
    template_prop_names: &FxHashSet<String>,
    template_scopes: &ScopeChain,
    usage_scope_id: ScopeId,
) -> Option<SpreadRewrite> {
    if !has_reserved_name_candidate(expression, template_prop_names) {
        return None;
    }

    let replacements = collect_replacements(
        expression,
        template_prop_names,
        template_scopes,
        usage_scope_id,
    );
    if replacements.is_empty() {
        return None;
    }
    Some(apply_replacements(expression, &replacements))
}

fn has_reserved_name_candidate(expression: &str, template_prop_names: &FxHashSet<String>) -> bool {
    if template_prop_names.is_empty() {
        return false;
    }
    let bytes = expression.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if !is_identifier_start(bytes[cursor]) {
            cursor += 1;
            continue;
        }
        let start = cursor;
        cursor += 1;
        while cursor < bytes.len() && is_identifier_continue(bytes[cursor]) {
            cursor += 1;
        }
        if template_prop_names.contains(&expression[start..cursor]) {
            return true;
        }
    }
    false
}

fn collect_replacements(
    expression: &str,
    template_prop_names: &FxHashSet<String>,
    template_scopes: &ScopeChain,
    usage_scope_id: ScopeId,
) -> Vec<Replacement> {
    let mut source = String::with_capacity(PARSE_PREFIX.len() + expression.len() + 2);
    source.push_str(PARSE_PREFIX);
    source.push_str(expression);
    source.push_str(PARSE_SUFFIX);

    let allocator = oxc_allocator::Allocator::default();
    let parsed = Parser::new(
        &allocator,
        source.as_str(),
        SourceType::ts().with_module(true),
    )
    .parse();
    if !parsed.diagnostics.is_empty() {
        return Vec::new();
    }
    let semantic = SemanticBuilder::new().build(&parsed.program);
    if !semantic.diagnostics.is_empty() {
        return Vec::new();
    }

    let mut collector = ReferenceCollector {
        expression_len: expression.len(),
        offset: PARSE_PREFIX.len() as u32,
        replacements: Vec::new(),
        scoping: semantic.semantic.scoping(),
        template_prop_names,
        template_scopes,
        usage_scope_id,
    };
    collector.visit_program(&parsed.program);
    collector
        .replacements
        .sort_by_key(|replacement| replacement.source.start);
    collector
        .replacements
        .dedup_by(|right, left| right.source == left.source);
    collector.replacements
}

struct ReferenceCollector<'a> {
    expression_len: usize,
    offset: u32,
    replacements: Vec<Replacement>,
    scoping: &'a Scoping,
    template_prop_names: &'a FxHashSet<String>,
    template_scopes: &'a ScopeChain,
    usage_scope_id: ScopeId,
}

impl ReferenceCollector<'_> {
    fn replacement(&self, ident: &IdentifierReference<'_>, shorthand: bool) -> Option<Replacement> {
        if !self.template_prop_names.contains(ident.name.as_str()) {
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
            || reference.flags().is_type()
            || reference.flags().is_value_as_type()
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
        if matches!(scope.kind, ScopeKind::VFor | ScopeKind::VSlot) && scope.has_binding(name) {
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

fn apply_replacements(expression: &str, replacements: &[Replacement]) -> SpreadRewrite {
    let extra_capacity = replacements.len().saturating_mul(16);
    let mut code = String::with_capacity(expression.len().saturating_add(extra_capacity));
    let mut segments = Vec::with_capacity(replacements.len().saturating_mul(2) + 1);
    let mut source_cursor = 0;

    for replacement in replacements {
        push_source_segment(
            expression,
            source_cursor..replacement.source.start,
            &mut code,
            &mut segments,
        );
        let generated_start = code.len();
        let name = &expression[replacement.source.clone()];
        if replacement.shorthand {
            append!(code, "{name}: props[\"{name}\"]");
        } else {
            append!(code, "props[\"{name}\"]");
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
    SpreadRewrite { code, segments }
}

fn push_source_segment(
    expression: &str,
    source: Range<usize>,
    code: &mut String,
    segments: &mut Vec<SpreadRewriteSegment>,
) {
    if source.is_empty() {
        return;
    }
    let generated_start = code.len();
    code.push_str(&expression[source.clone()]);
    segments.push(SpreadRewriteSegment {
        generated: generated_start..code.len(),
        source,
    });
}

const fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$')
}

const fn is_identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn props(names: &[&str]) -> FxHashSet<String> {
        names.iter().copied().map(Into::into).collect()
    }

    fn rewrite(expression: &str, names: &FxHashSet<String>) -> Option<SpreadRewrite> {
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
