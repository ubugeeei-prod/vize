//! Compatibility rules whose evidence belongs to the SFC or script surface.

mod content;
mod events;
mod options;
mod sort_keys;

pub use content::NoContentOutsideBlock;
pub use events::NoShadowNativeEvents;
pub use options::NoPropsShadow;
pub use sort_keys::SortKeys;

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Expression, ObjectExpression, ObjectPropertyKind,
    Program, PropertyKey, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};
use vize_l0::cstr;

macro_rules! meta {
    ($name:literal, $description:literal) => {
        RuleMeta {
            name: $name,
            description: $description,
            category: RuleCategory::Recommended,
            fixable: false,
            default_severity: Severity::Warning,
        }
    };
}

static ENFORCE_STYLE_ATTRIBUTE_META: RuleMeta = meta!(
    "vue/enforce-style-attribute",
    "Enforce allowed style-block attributes"
);
static NEXT_TICK_STYLE_META: RuleMeta = meta!(
    "vue/next-tick-style",
    "Enforce Promise-style nextTick usage"
);
static NO_DEFINE_MODEL_DEFAULT_META: RuleMeta = meta!(
    "vue/no-define-model-default",
    "Disallow default values in defineModel"
);
static REQUIRE_MACRO_VARIABLE_NAME_META: RuleMeta = meta!(
    "vue/require-macro-variable-name",
    "Require conventional macro variable names"
);

pub(crate) fn rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(EnforceStyleAttribute::default()),
        Box::new(NextTickStyle),
        Box::new(NoContentOutsideBlock),
        Box::new(NoDefineModelDefault),
        Box::new(NoPropsShadow),
        Box::new(NoShadowNativeEvents),
        Box::new(RequireMacroVariableName),
        Box::new(SortKeys::default()),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Authored style-block modes accepted by `vue/enforce-style-attribute`.
pub enum StyleAttribute {
    /// A plain `<style>` block without `scoped` or `module`.
    Plain,
    /// A `<style scoped>` block.
    Scoped,
    /// A `<style module>` block.
    Module,
}

/// `vue/enforce-style-attribute`; upstream defaults to `scoped` only.
pub struct EnforceStyleAttribute {
    allow_plain: bool,
    allow_scoped: bool,
    allow_module: bool,
}

impl Default for EnforceStyleAttribute {
    fn default() -> Self {
        Self {
            allow_plain: false,
            allow_scoped: true,
            allow_module: false,
        }
    }
}

impl EnforceStyleAttribute {
    /// Build the rule from the style modes accepted by the project.
    pub fn new(allowed: &[StyleAttribute]) -> Self {
        Self {
            allow_plain: allowed.contains(&StyleAttribute::Plain),
            allow_scoped: allowed.contains(&StyleAttribute::Scoped),
            allow_module: allowed.contains(&StyleAttribute::Module),
        }
    }
}

impl Rule for EnforceStyleAttribute {
    fn meta(&self) -> &'static RuleMeta {
        &ENFORCE_STYLE_ATTRIBUTE_META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let finding = ctx.sfc_descriptor().and_then(|descriptor| {
            descriptor.styles.iter().find_map(|style| {
                let allowed = if style.scoped {
                    self.allow_scoped
                } else if style.module.is_some() {
                    self.allow_module
                } else {
                    self.allow_plain
                };
                (!allowed).then_some((style.loc.tag_start as u32, style.loc.start as u32))
            })
        });
        if let Some((start, end)) = finding {
            ctx.report_in_sfc(
                LintDiagnostic::warn(
                    ENFORCE_STYLE_ATTRIBUTE_META.name,
                    "This style block uses a disallowed style attribute mode",
                    start,
                    end,
                )
                .with_help("Use one of the style modes allowed by this rule's configuration"),
            );
        }
    }
}

/// `vue/no-define-model-default`.
pub struct NoDefineModelDefault;

impl Rule for NoDefineModelDefault {
    fn meta(&self) -> &'static RuleMeta {
        &NO_DEFINE_MODEL_DEFAULT_META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let findings = setup_program(ctx, |program, offset, _| {
            let mut visitor = DefineModelDefaults {
                offset,
                findings: Vec::new(),
            };
            visitor.visit_program(program);
            visitor.findings
        });
        for finding in findings {
            ctx.warn_at_with_help(
                "Unexpected default value in `defineModel`",
                finding,
                "Keep the parent and child model values synchronized by applying the default in the parent",
            );
        }
    }
}

struct DefineModelDefaults {
    offset: u32,
    findings: Vec<ByteRange>,
}

impl<'a> Visit<'a> for DefineModelDefaults {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if callee_name(call) == Some("defineModel")
            && let Some(object) = call.arguments.iter().rev().find_map(object_argument)
            && let Some(span) = property_span(object, "default")
        {
            self.findings.push(absolute_range(self.offset, span));
        }
        walk::walk_call_expression(self, call);
    }
}

/// `vue/require-macro-variable-name` with the upstream defaults.
pub struct RequireMacroVariableName;

impl Rule for RequireMacroVariableName {
    fn meta(&self) -> &'static RuleMeta {
        &REQUIRE_MACRO_VARIABLE_NAME_META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let findings = setup_program(ctx, |program, offset, _| {
            let mut visitor = MacroVariableNames {
                offset,
                findings: Vec::new(),
            };
            visitor.visit_program(program);
            visitor.findings
        });
        for (macro_name, expected, range) in findings {
            ctx.warn_at_with_help(
                cstr!(
                    "The variable name of `{}` must be `{}`",
                    macro_name,
                    expected
                ),
                range,
                cstr!("Rename this binding to `{}`", expected),
            );
        }
    }
}

struct MacroVariableNames {
    offset: u32,
    findings: Vec<(&'static str, &'static str, ByteRange)>,
}

impl<'a> Visit<'a> for MacroVariableNames {
    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        let BindingPattern::BindingIdentifier(binding) = &declarator.id else {
            walk::walk_variable_declarator(self, declarator);
            return;
        };
        let Some(initializer) = declarator.init.as_ref() else {
            return;
        };
        let Some((macro_name, expected)) = macro_binding(initializer) else {
            walk::walk_variable_declarator(self, declarator);
            return;
        };
        if binding.name.as_str() != expected {
            self.findings.push((
                macro_name,
                expected,
                absolute_range(self.offset, binding.span),
            ));
        }
        walk::walk_variable_declarator(self, declarator);
    }
}

fn macro_binding(expression: &Expression<'_>) -> Option<(&'static str, &'static str)> {
    let Expression::CallExpression(call) = expression.get_inner_expression() else {
        return None;
    };
    let name = callee_name(call)?;
    let name = if name == "withDefaults" {
        call.arguments
            .first()
            .and_then(Argument::as_expression)
            .and_then(|argument| match argument.get_inner_expression() {
                Expression::CallExpression(inner) => callee_name(inner),
                _ => None,
            })?
    } else {
        name
    };
    match name {
        "defineProps" => Some(("defineProps", "props")),
        "defineEmits" => Some(("defineEmits", "emit")),
        "defineSlots" => Some(("defineSlots", "slots")),
        "useSlots" => Some(("useSlots", "slots")),
        "useAttrs" => Some(("useAttrs", "attrs")),
        _ => None,
    }
}

/// `vue/next-tick-style`; the upstream default is Promise style.
pub struct NextTickStyle;

impl Rule for NextTickStyle {
    fn meta(&self) -> &'static RuleMeta {
        &NEXT_TICK_STYLE_META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let mut findings = Vec::new();
        collect_script_programs(ctx, |program, offset, source| {
            let mut visitor = NextTickCalls {
                offset,
                source,
                findings: Vec::new(),
            };
            visitor.visit_program(program);
            findings.extend(visitor.findings);
        });
        for finding in findings {
            ctx.warn_at_with_help(
                "Use the Promise returned by `nextTick`",
                finding,
                "Await `nextTick()` or chain `.then(...)` instead of passing a callback",
            );
        }
    }
}

struct NextTickCalls<'s> {
    offset: u32,
    source: &'s str,
    findings: Vec<ByteRange>,
}

impl<'a> Visit<'a> for NextTickCalls<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_next_tick_call(call)
            && (!call.arguments.is_empty() || !promise_is_consumed(self.source, call.span))
        {
            self.findings
                .push(absolute_range(self.offset, call.callee.span()));
        }
        walk::walk_call_expression(self, call);
    }
}

fn is_next_tick_call(call: &CallExpression<'_>) -> bool {
    match call.callee.get_inner_expression() {
        Expression::Identifier(identifier) => identifier.name == "nextTick",
        Expression::StaticMemberExpression(member) => {
            member.property.name == "$nextTick"
                || (member.property.name == "nextTick"
                    && matches!(
                        member.object.get_inner_expression(),
                        Expression::Identifier(identifier) if identifier.name == "Vue"
                    ))
        }
        _ => false,
    }
}

fn promise_is_consumed(source: &str, span: Span) -> bool {
    source
        .get(..span.start as usize)
        .is_some_and(|prefix| prefix.trim_end().ends_with("await"))
        || source
            .get(span.end as usize..)
            .is_some_and(|suffix| suffix.trim_start().starts_with(".then"))
}

pub(super) fn setup_program<T>(
    ctx: &LintContext<'_>,
    collect: impl FnOnce(&Program<'_>, u32, &str) -> T,
) -> T
where
    T: Default,
{
    let Some(descriptor) = ctx.sfc_descriptor() else {
        return T::default();
    };
    let Some(block) = descriptor.script_setup.as_ref() else {
        return T::default();
    };
    let Some(parsed) = ctx
        .sfc_scripts()
        .and_then(|scripts| scripts.script_setup())
        .filter(|parsed| !parsed.panicked && parsed.diagnostics.is_empty())
    else {
        return T::default();
    };
    collect(
        &parsed.program,
        block.loc.start as u32,
        block.content.as_ref(),
    )
}

pub(super) fn collect_script_programs(
    ctx: &LintContext<'_>,
    mut collect: impl FnMut(&Program<'_>, u32, &str),
) {
    let (Some(descriptor), Some(scripts)) = (ctx.sfc_descriptor(), ctx.sfc_scripts()) else {
        return;
    };
    if let (Some(block), Some(parsed)) = (descriptor.script.as_ref(), scripts.script())
        && !parsed.panicked
        && parsed.diagnostics.is_empty()
    {
        collect(
            &parsed.program,
            block.loc.start as u32,
            block.content.as_ref(),
        );
    }
    if let (Some(block), Some(parsed)) = (descriptor.script_setup.as_ref(), scripts.script_setup())
        && !parsed.panicked
        && parsed.diagnostics.is_empty()
    {
        collect(
            &parsed.program,
            block.loc.start as u32,
            block.content.as_ref(),
        );
    }
}

pub(super) fn callee_name<'a>(call: &'a CallExpression<'a>) -> Option<&'a str> {
    match call.callee.get_inner_expression() {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        Expression::StaticMemberExpression(member) => Some(member.property.name.as_str()),
        _ => None,
    }
}

pub(super) fn object_argument<'a>(argument: &'a Argument<'a>) -> Option<&'a ObjectExpression<'a>> {
    match argument.as_expression()?.get_inner_expression() {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

pub(super) fn property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::Identifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

fn property_span(object: &ObjectExpression<'_>, name: &str) -> Option<Span> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        (property_name(&property.key) == Some(name)).then(|| property.span())
    })
}

pub(super) fn absolute_range(offset: u32, span: Span) -> ByteRange {
    ByteRange {
        start: offset + span.start,
        end: offset + span.end,
    }
}

#[cfg(test)]
mod tests;
