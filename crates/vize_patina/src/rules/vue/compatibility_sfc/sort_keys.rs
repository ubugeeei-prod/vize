//! `vue/sort-keys` compatibility rule.
//!
//! Every object literal of the script blocks is checked, as in eslint-plugin-vue,
//! except a component options object itself, the children of `model` and the
//! grandchildren of `computed`, `directives`, `inject`, `props` and `watch`.

mod natural;

use super::absolute_range;
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{
    Argument, CallExpression, ExportDefaultDeclaration, Expression, ImportDeclarationSpecifier,
    NewExpression, ObjectExpression, ObjectPropertyKind, Program, PropertyKey, Statement,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};
use vize_l0::{FxHashSet, String, cstr};

static META: RuleMeta = RuleMeta {
    name: "vue/sort-keys",
    description: "Require sorted object keys, compatible with component option order",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

const IGNORE_CHILDREN_OF: &[&str] = &["model"];
const IGNORE_GRANDCHILDREN_OF: &[&str] = &["computed", "directives", "inject", "props", "watch"];

/// Require object keys in the configured order.
pub struct SortKeys {
    descending: bool,
    case_sensitive: bool,
    natural: bool,
}

impl Default for SortKeys {
    fn default() -> Self {
        Self {
            descending: false,
            case_sensitive: true,
            natural: false,
        }
    }
}

impl SortKeys {
    /// Build the rule from its ordering, case and natural-sort policy.
    pub fn new(descending: bool, case_sensitive: bool, natural: bool) -> Self {
        Self {
            descending,
            case_sensitive,
            natural,
        }
    }

    fn in_order(&self, previous: &str, current: &str) -> bool {
        let (left, right) = if self.descending {
            (current, previous)
        } else {
            (previous, current)
        };
        let (left, right): (std::borrow::Cow<'_, str>, std::borrow::Cow<'_, str>) =
            if self.case_sensitive {
                (left.into(), right.into())
            } else {
                (left.to_lowercase().into(), right.to_lowercase().into())
            };
        if self.natural {
            natural::compare(&left, &right).is_le()
        } else {
            utf16_cmp(&left, &right).is_le()
        }
    }
}

/// JavaScript compares strings by UTF-16 code unit.
fn utf16_cmp(left: &str, right: &str) -> std::cmp::Ordering {
    left.encode_utf16().cmp(right.encode_utf16())
}

impl Rule for SortKeys {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let (Some(descriptor), Some(scripts)) = (ctx.sfc_descriptor(), ctx.sfc_scripts()) else {
            return;
        };
        let blocks = [
            (descriptor.script.as_ref(), scripts.script(), false),
            (
                descriptor.script_setup.as_ref(),
                scripts.script_setup(),
                true,
            ),
        ];
        let mut findings = Vec::new();
        for (block, parsed, setup) in blocks {
            let (Some(block), Some(parsed)) = (block, parsed) else {
                continue;
            };
            if parsed.panicked || !parsed.diagnostics.is_empty() {
                continue;
            }
            let mut visitor = SortedKeys::new(self, &parsed.program, block.loc.start as u32, setup);
            visitor.visit_program(&parsed.program);
            findings.extend(visitor.findings);
        }
        for (current, previous, range) in findings {
            ctx.warn_at_with_help(
                cstr!("`{}` should be before `{}`", current, previous),
                range,
                "Keep object keys in the configured order",
            );
        }
    }
}

/// Where an object literal sits relative to a component options object.
#[derive(Clone, Default)]
struct VueState {
    is_vue_object: bool,
    within: bool,
    ignore: bool,
    property_name: String,
    chain_level: u32,
}

struct SortedKeys<'r> {
    rule: &'r SortKeys,
    offset: u32,
    /// Spans of the object literals that are component options.
    vue_objects: FxHashSet<(u32, u32)>,
    /// The object that is the value of the property being visited, with the
    /// state of the object that owns that property.
    chain: Option<((u32, u32), VueState, String)>,
    /// Imported names of `vue`'s `component`, `createApp` and `defineComponent`.
    vue_imports: FxHashSet<String>,
    other_imports: FxHashSet<String>,
    setup: bool,
    marked_lines: Vec<u32>,
    source: &'r str,
    findings: Vec<(String, String, ByteRange)>,
}

impl<'r> SortedKeys<'r> {
    fn new(rule: &'r SortKeys, program: &'r Program<'_>, offset: u32, setup: bool) -> Self {
        let mut vue_imports = FxHashSet::default();
        let mut other_imports = FxHashSet::default();
        for statement in &program.body {
            let Statement::ImportDeclaration(import) = statement else {
                continue;
            };
            let from_vue = import.source.value.as_str() == "vue";
            for specifier in import.specifiers.iter().flatten() {
                let local = match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => &specifier.local,
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        &specifier.local
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        &specifier.local
                    }
                };
                let names = if from_vue {
                    &mut vue_imports
                } else {
                    &mut other_imports
                };
                names.insert(String::from(local.name.as_str()));
            }
        }
        // `// @vue/component` marks the object that starts on the next line.
        let marked_lines = program
            .comments
            .iter()
            .filter(|comment| {
                program
                    .source_text
                    .get(comment.span.start as usize..comment.span.end as usize)
                    .is_some_and(|text| text.contains("@vue/component"))
            })
            .map(|comment| line_of(program.source_text, comment.span.end))
            .collect();
        Self {
            rule,
            offset,
            vue_objects: FxHashSet::default(),
            chain: None,
            vue_imports,
            other_imports,
            setup,
            marked_lines,
            source: program.source_text,
            findings: Vec::new(),
        }
    }

    fn mark_vue_object(&mut self, expression: Option<&Expression<'_>>) {
        if let Some(Expression::ObjectExpression(object)) = expression.map(skip_ts_as) {
            self.vue_objects.insert(key(object.span));
        }
    }

    fn is_vue_callee(&self, call: &CallExpression<'_>) -> bool {
        match call.callee.without_parentheses() {
            Expression::Identifier(callee) => match callee.name.as_str() {
                "defineNuxtComponent" => true,
                name @ ("component" | "createApp" | "defineComponent") => {
                    self.vue_imports.contains(name) || !self.other_imports.contains(name)
                }
                _ => false,
            },
            callee => callee.as_member_expression().is_some_and(|member| {
                let Expression::Identifier(object) = skip_ts_as(member.object()) else {
                    return false;
                };
                match member.static_property_name() {
                    Some("component" | "mixin") => true,
                    Some("extend") => object.name == "Vue",
                    _ => false,
                }
            }),
        }
    }

    fn check_order(&mut self, object: &ObjectExpression<'_>) {
        if object.properties.len() < 2 {
            return;
        }
        let mut previous: Option<String> = None;
        for entry in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = entry else {
                previous = None;
                continue;
            };
            let Some(current) = sort_name(&property.key, property.computed) else {
                continue;
            };
            if let Some(before) = &previous
                && !self.rule.in_order(before, &current)
            {
                self.findings.push((
                    current.clone(),
                    before.clone(),
                    absolute_range(self.offset, property.key.span()),
                ));
            }
            previous = Some(current);
        }
    }

    fn state_of(&mut self, object: &ObjectExpression<'_>) -> VueState {
        let previous_line = line_of(self.source, object.span.start).checked_sub(1);
        let is_vue_object = self.vue_objects.contains(&key(object.span))
            || previous_line.is_some_and(|line| self.marked_lines.contains(&line));
        if is_vue_object {
            return VueState {
                is_vue_object,
                within: true,
                ignore: true,
                ..VueState::default()
            };
        }
        if self
            .chain
            .as_ref()
            .is_none_or(|(span, ..)| *span != key(object.span))
        {
            return VueState::default();
        }
        let Some((_, upper, property_name)) = self.chain.take() else {
            return VueState::default();
        };
        let (property_name, chain_level) = if upper.is_vue_object {
            (property_name, 1)
        } else {
            (upper.property_name, upper.chain_level.saturating_add(1))
        };
        let ignore = chain_level == 1 && IGNORE_CHILDREN_OF.contains(&property_name.as_str())
            || chain_level == 2 && IGNORE_GRANDCHILDREN_OF.contains(&property_name.as_str());
        VueState {
            is_vue_object: false,
            within: true,
            ignore,
            property_name,
            chain_level,
        }
    }
}

impl<'a> Visit<'a> for SortedKeys<'_> {
    fn visit_export_default_declaration(&mut self, declaration: &ExportDefaultDeclaration<'a>) {
        if !self.setup {
            self.mark_vue_object(
                declaration
                    .declaration
                    .as_expression()
                    .and_then(|expression| {
                        matches!(
                            expression.without_parentheses(),
                            Expression::ObjectExpression(_)
                        )
                        .then_some(expression)
                    }),
            );
        }
        walk::walk_export_default_declaration(self, declaration);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.is_vue_callee(call) {
            self.mark_vue_object(call.arguments.last().and_then(Argument::as_expression));
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_new_expression(&mut self, call: &NewExpression<'a>) {
        if matches!(call.callee.without_parentheses(),
            Expression::Identifier(callee) if callee.name == "Vue")
        {
            self.mark_vue_object(call.arguments.first().and_then(Argument::as_expression));
        }
        walk::walk_new_expression(self, call);
    }

    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        let state = self.state_of(object);
        if !state.ignore {
            self.check_order(object);
        }
        for entry in &object.properties {
            match entry {
                ObjectPropertyKind::ObjectProperty(property) => {
                    self.chain = match skip_ts_as(&property.value) {
                        Expression::ObjectExpression(child) if state.within => Some((
                            key(child.span),
                            state.clone(),
                            static_name(&property.key, property.computed).unwrap_or_default(),
                        )),
                        _ => None,
                    };
                    self.visit_object_property(property);
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    self.chain = None;
                    self.visit_spread_element(spread);
                }
            }
        }
        self.chain = None;
    }
}

fn key(span: Span) -> (u32, u32) {
    (span.start, span.end)
}

fn skip_ts_as<'b, 'a>(expression: &'b Expression<'a>) -> &'b Expression<'a> {
    let mut expression = expression.without_parentheses();
    while let Expression::TSAsExpression(inner) = expression {
        expression = inner.expression.without_parentheses();
    }
    expression
}

fn line_of(source: &str, offset: u32) -> u32 {
    let end = (offset as usize).min(source.len());
    let newlines = source.get(..end).map_or(0, |before| {
        before.bytes().filter(|byte| *byte == b'\n').count()
    });
    u32::try_from(newlines).unwrap_or(u32::MAX)
}

/// `getStaticPropertyName` of a property.
fn static_name(key: &PropertyKey<'_>, computed: bool) -> Option<String> {
    use oxc_syntax::number::ToJsString;
    match key {
        PropertyKey::StaticIdentifier(identifier) if !computed => {
            Some(String::from(identifier.name.as_str()))
        }
        PropertyKey::StringLiteral(literal) => Some(String::from(literal.value.as_str())),
        PropertyKey::NumericLiteral(literal) if literal.value == 0.0 => Some(String::from("0")),
        PropertyKey::NumericLiteral(literal) => Some(String::from(literal.value.to_js_string())),
        PropertyKey::BigIntLiteral(literal) => Some(String::from(literal.value.as_str())),
        PropertyKey::BooleanLiteral(literal) => {
            Some(String::from(if literal.value { "true" } else { "false" }))
        }
        PropertyKey::NullLiteral(_) => Some(String::from("null")),
        PropertyKey::TemplateLiteral(template) if template.expressions.is_empty() => template
            .quasis
            .first()
            .and_then(|quasi| quasi.value.cooked.as_ref())
            .map(|cooked| String::from(cooked.as_str())),
        _ => None,
    }
}

/// The name a key sorts by: its static name, or the name of a computed identifier.
fn sort_name(key: &PropertyKey<'_>, computed: bool) -> Option<String> {
    static_name(key, computed).or_else(|| match key {
        PropertyKey::Identifier(identifier) => Some(String::from(identifier.name.as_str())),
        _ => None,
    })
}
