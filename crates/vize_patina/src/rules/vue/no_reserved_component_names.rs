//! vue/no-reserved-component-names
//!
//! Disallow the use of reserved names as component names.
//!
//! HTML element names, SVG element names, and Vue built-in component names
//! should not be used as component names.
//!
//! This rule checks explicit component-name declarations (`name` option or
//! `defineOptions({ name })`), NOT names inferred from filenames and NOT names
//! of other components used in the template. This matches the behavior of
//! eslint-plugin-vue. Using `<Transition>` or `<KeepAlive>` in a template is
//! perfectly valid — they are Vue built-in components being used correctly.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   name: 'button'
//! }
//! ```
//!
//! ### Valid
//! ```vue
//! <!-- Button.vue -->
//! <script setup></script>
//! <template><div /></template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::script::script_source_type;
use oxc_allocator::Allocator as OxcAllocator;
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, ExportDefaultDeclarationKind, Expression,
    ObjectExpression, ObjectPropertyKind, Program, PropertyKey, Statement, StringLiteral,
};
use oxc_parser::Parser;
use vize_carton::is_html_tag;
use vize_carton::{FxHashMap, String};
use vize_croquis::builtins::is_builtin_component;

static META: RuleMeta = RuleMeta {
    name: "vue/no-reserved-component-names",
    description: "Disallow the use of reserved names as component names",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Reserved names that cannot be used (specific edge cases)
const RESERVED_NAMES: &[&str] = &[
    "annotation-xml",
    "color-profile",
    "font-face",
    "font-face-src",
    "font-face-uri",
    "font-face-format",
    "font-face-name",
    "missing-glyph",
];

/// Disallow reserved component names
pub struct NoReservedComponentNames {
    /// Also disallow HTML element names
    pub disallow_html: bool,
    /// Also disallow Vue built-ins
    pub disallow_vue_builtins: bool,
}

impl Default for NoReservedComponentNames {
    fn default() -> Self {
        Self {
            disallow_html: true,
            disallow_vue_builtins: true,
        }
    }
}

impl Rule for NoReservedComponentNames {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let findings = {
            let Some(descriptor) = ctx.sfc_descriptor() else {
                return;
            };
            let mut findings = Vec::new();

            if let Some(script) = descriptor.script.as_ref() {
                self.collect_script_block_findings(
                    script.content.as_ref(),
                    script.loc.start,
                    &mut findings,
                );
            }
            if let Some(script_setup) = descriptor.script_setup.as_ref() {
                self.collect_script_setup_block_findings(
                    script_setup.content.as_ref(),
                    script_setup.loc.start,
                    &mut findings,
                );
            }

            findings
        };

        for finding in findings {
            ctx.error_at_with_help(
                ctx.t_fmt(
                    "vue/no-reserved-component-names.message",
                    &[("name", finding.name.as_str())],
                ),
                ByteRange {
                    start: finding.start,
                    end: finding.end,
                },
                ctx.t("vue/no-reserved-component-names.help"),
            );
        }
    }
}

impl NoReservedComponentNames {
    fn collect_script_block_findings(
        &self,
        source: &str,
        offset: usize,
        findings: &mut Vec<ComponentNameFinding>,
    ) {
        // A finding here requires a default-exported component options object
        // carrying a `name` property. Skip the oxc parse entirely when either
        // token is absent so the common case (no Options API `name`) stays a
        // cheap byte scan instead of a full parse per file.
        let bytes = source.as_bytes();
        if memchr::memmem::find(bytes, b"export default").is_none()
            || memchr::memmem::find(bytes, b"name").is_none()
        {
            return;
        }

        let allocator = OxcAllocator::default();
        let parsed = Parser::new(&allocator, source, script_source_type()).parse();
        if parsed.panicked || !parsed.errors.is_empty() {
            return;
        }

        if let Some(options) = find_component_options(&parsed.program)
            && let Some(name) = name_string_literal(options)
        {
            self.collect_name_finding(name, offset, findings);
        }
    }

    fn collect_script_setup_block_findings(
        &self,
        source: &str,
        offset: usize,
        findings: &mut Vec<ComponentNameFinding>,
    ) {
        // The only `<script setup>` source of an explicit component name is
        // `defineOptions({ name })`. Without that call there is nothing to
        // flag, so avoid parsing files that never reference it.
        if memchr::memmem::find(source.as_bytes(), b"defineOptions").is_none() {
            return;
        }

        let allocator = OxcAllocator::default();
        let parsed = Parser::new(&allocator, source, script_source_type()).parse();
        if parsed.panicked || !parsed.errors.is_empty() {
            return;
        }

        if let Some(name) = define_options_name(&parsed.program) {
            self.collect_name_finding(name, offset, findings);
        }
    }

    fn collect_name_finding(
        &self,
        name: &StringLiteral<'_>,
        offset: usize,
        findings: &mut Vec<ComponentNameFinding>,
    ) {
        let value = name.value.as_str();
        if !self.is_reserved_component_name(value) {
            return;
        }

        findings.push(ComponentNameFinding {
            name: String::from(value),
            start: offset as u32 + name.span.start,
            end: offset as u32 + name.span.end,
        });
    }

    fn is_reserved_component_name(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        RESERVED_NAMES.contains(&name_lower.as_str())
            || (self.disallow_html && is_html_tag(name))
            || (self.disallow_vue_builtins
                && (is_builtin_component(&name_lower) || is_builtin_component(name)))
    }
}

struct ComponentNameFinding {
    name: String,
    start: u32,
    end: u32,
}

fn name_string_literal<'a>(options: &'a ObjectExpression<'a>) -> Option<&'a StringLiteral<'a>> {
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if !matches!(property_key_name(&property.key), Some("name")) {
            continue;
        }
        if let Expression::StringLiteral(literal) = &property.value {
            return Some(literal);
        }
    }
    None
}

fn define_options_name<'a>(program: &'a Program<'a>) -> Option<&'a StringLiteral<'a>> {
    for statement in program.body.iter() {
        let Statement::ExpressionStatement(expression) = statement else {
            continue;
        };
        let Expression::CallExpression(call) = &expression.expression else {
            continue;
        };
        let Expression::Identifier(callee) = &call.callee else {
            continue;
        };
        if !matches!(callee.name.as_str(), "defineOptions") {
            continue;
        }
        if let Some(Argument::ObjectExpression(object)) = call.arguments.first() {
            return name_string_literal(object);
        }
    }
    None
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

fn find_component_options<'a>(program: &'a Program<'a>) -> Option<&'a ObjectExpression<'a>> {
    let mut bindings: FxHashMap<&'a str, &'a ObjectExpression<'a>> = FxHashMap::default();

    for statement in program.body.iter() {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            if let BindingPattern::BindingIdentifier(id) = &declarator.id
                && let Some(object) = options_from_expression(init, &bindings)
            {
                bindings.insert(id.name.as_str(), object);
            }
        }
    }

    for statement in program.body.iter() {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        if let Some(object) = options_from_export(&export.declaration, &bindings) {
            return Some(object);
        }
    }

    None
}

fn options_from_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::CallExpression(call) => options_from_call(call, bindings),
        ExportDefaultDeclarationKind::Identifier(identifier) => {
            bindings.get(identifier.name.as_str()).copied()
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(paren) => {
            options_from_expression(&paren.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            options_from_expression(&ts_as.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            options_from_expression(&ts_satisfies.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            options_from_expression(&ts_non_null.expression, bindings)
        }
        _ => None,
    }
}

fn options_from_expression<'a>(
    expression: &'a Expression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::CallExpression(call) => options_from_call(call, bindings),
        Expression::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        Expression::ParenthesizedExpression(paren) => {
            options_from_expression(&paren.expression, bindings)
        }
        Expression::TSAsExpression(ts_as) => options_from_expression(&ts_as.expression, bindings),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            options_from_expression(&ts_satisfies.expression, bindings)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            options_from_expression(&ts_non_null.expression, bindings)
        }
        _ => None,
    }
}

fn options_from_call<'a>(
    call: &'a CallExpression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if !matches!(callee.name.as_str(), "defineComponent" | "_defineComponent") {
        return None;
    }
    match call.arguments.first()? {
        Argument::ObjectExpression(object) => Some(object),
        Argument::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        argument => argument
            .as_expression()
            .and_then(|expression| options_from_expression(expression, bindings)),
    }
}

#[cfg(test)]
mod tests;
