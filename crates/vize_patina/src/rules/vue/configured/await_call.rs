//! `vue/no-restricted-call-after-await`.
//!
//! Like eslint-plugin-vue, only calls made by a `setup()` function itself
//! count. `<script setup>` restores the instance after a top-level `await`,
//! and nested functions run later, so neither is reported.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{
    Argument, AwaitExpression, CallExpression, Expression, FunctionBody,
    ImportDeclarationSpecifier, ObjectExpression, ObjectPropertyKind, Program, Statement,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_await_expression, walk_call_expression},
};
use oxc_syntax::scope::ScopeFlags;
use vize_l0::config::{CallAfterAwaitOptions, CallAfterAwaitSpec};
use vize_l0::{CompactString, String, cstr};

use super::await_ast::{callee_name, callee_path, object_of_export, property_key};
use super::{each_program, report};

static META: RuleMeta = RuleMeta {
    name: "vue/no-restricted-call-after-await",
    description: "disallow asynchronously called restricted methods",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-restricted-call-after-await`.
#[derive(Default)]
pub struct NoRestrictedCallAfterAwait {
    restrictions: Vec<CallAfterAwaitSpec>,
}

impl NoRestrictedCallAfterAwait {
    /// Build the rule from `linter.ruleOptions["vue/no-restricted-call-after-await"]`.
    pub(crate) fn from_options(options: &CallAfterAwaitOptions) -> Self {
        Self {
            restrictions: options.restrictions(),
        }
    }
}

impl Rule for NoRestrictedCallAfterAwait {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        if self.restrictions.is_empty() {
            return;
        }
        let mut findings = Vec::new();
        each_program(ctx, |program, offset| {
            for (span, message) in scan_program(program, &self.restrictions) {
                findings.push((message, offset + span.start, offset + span.end));
            }
        });
        report(ctx, META.name, findings);
    }
}

enum Binding {
    Named {
        local: CompactString,
        imported: CompactString,
    },
    Default {
        local: CompactString,
    },
    Namespace {
        local: CompactString,
    },
}

fn scan_program(
    program: &Program<'_>,
    restrictions: &[CallAfterAwaitSpec],
) -> Vec<(oxc_span::Span, CompactString)> {
    let bindings = imports(program);
    let mut components = Components {
        restrictions,
        bindings: &bindings,
        findings: Vec::new(),
    };
    components.visit_program(program);
    components.findings
}

fn imports(program: &Program<'_>) -> Vec<(String, Vec<Binding>)> {
    let mut grouped: Vec<(String, Vec<Binding>)> = Vec::new();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.import_kind.is_type() {
            continue;
        }
        let module = String::from(import.source.value.as_str());
        let slot = grouped
            .iter_mut()
            .find(|(name, _)| name == module)
            .map(|(_, bindings)| bindings);
        let bindings = if let Some(slot) = slot {
            slot
        } else {
            grouped.push((module, Vec::new()));
            let Some((_, bindings)) = grouped.last_mut() else {
                continue;
            };
            bindings
        };
        for specifier in import.specifiers.iter().flatten() {
            if let Some(binding) = binding(specifier) {
                bindings.push(binding);
            }
        }
    }
    grouped
}

fn binding(specifier: &ImportDeclarationSpecifier<'_>) -> Option<Binding> {
    match specifier {
        ImportDeclarationSpecifier::ImportSpecifier(specifier)
            if !specifier.import_kind.is_type() =>
        {
            Some(Binding::Named {
                local: CompactString::from(specifier.local.name.as_str()),
                imported: CompactString::from(specifier.imported.name().as_str()),
            })
        }
        ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => Some(Binding::Default {
            local: CompactString::from(specifier.local.name.as_str()),
        }),
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
            Some(Binding::Namespace {
                local: CompactString::from(specifier.local.name.as_str()),
            })
        }
        _ => None,
    }
}

struct Components<'a> {
    restrictions: &'a [CallAfterAwaitSpec],
    bindings: &'a [(String, Vec<Binding>)],
    findings: Vec<(oxc_span::Span, CompactString)>,
}

impl<'a> Visit<'a> for Components<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if matches!(
            callee_name(&call.callee),
            Some("defineComponent" | "defineNuxtComponent")
        ) && let Some(Argument::ObjectExpression(options)) = call.arguments.first()
        {
            self.scan_options(options);
        }
        oxc_ast_visit::walk::walk_call_expression(self, call);
    }

    fn visit_export_default_declaration(
        &mut self,
        declaration: &oxc_ast::ast::ExportDefaultDeclaration<'a>,
    ) {
        if let Some(options) = object_of_export(&declaration.declaration) {
            self.scan_options(options);
        }
        oxc_ast_visit::walk::walk_export_default_declaration(self, declaration);
    }
}

impl Components<'_> {
    fn scan_options(&mut self, options: &ObjectExpression<'_>) {
        for property in &options.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            if property_key(&property.key) != Some("setup") {
                continue;
            }
            match &property.value {
                Expression::FunctionExpression(function) => {
                    if let Some(body) = &function.body {
                        self.scan_setup(body);
                    }
                }
                Expression::ArrowFunctionExpression(arrow) => self.scan_setup(&arrow.body),
                _ => {}
            }
        }
    }

    fn scan_setup(&mut self, body: &FunctionBody<'_>) {
        let mut scan = Scan {
            restrictions: self.restrictions,
            bindings: self.bindings,
            after_await: false,
            findings: Vec::new(),
        };
        for statement in &body.statements {
            scan.visit_statement(statement);
        }
        self.findings.append(&mut scan.findings);
    }
}

struct Scan<'a> {
    restrictions: &'a [CallAfterAwaitSpec],
    bindings: &'a [(String, Vec<Binding>)],
    after_await: bool,
    findings: Vec<(oxc_span::Span, CompactString)>,
}

impl<'a> Visit<'a> for Scan<'a> {
    fn visit_function(&mut self, _it: &oxc_ast::ast::Function<'a>, _flags: ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _it: &oxc_ast::ast::ArrowFunctionExpression<'a>) {
    }

    fn visit_class(&mut self, _it: &oxc_ast::ast::Class<'a>) {}

    fn visit_await_expression(&mut self, it: &AwaitExpression<'a>) {
        self.after_await = true;
        walk_await_expression(self, it);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.after_await {
            self.check(call);
        }
        walk_call_expression(self, call);
    }
}

impl Scan<'_> {
    fn check(&mut self, call: &CallExpression<'_>) {
        let Some((root, members)) = callee_path(&call.callee) else {
            return;
        };
        for restriction in self.restrictions {
            let Some(bindings) = self
                .bindings
                .iter()
                .find(|(module, _)| module == restriction.module)
                .map(|(_, bindings)| bindings)
            else {
                continue;
            };
            let segments = restriction.segments();
            if !path_matches(bindings, root, &members, &segments) {
                continue;
            }
            let message = restriction.message.clone().unwrap_or_else(|| {
                let path = segments
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(".");
                cstr!(
                    "`import(\"{}\").{path}` is forbidden after an `await` expression.",
                    restriction.module
                )
            });
            self.findings.push((call.span, message));
            return;
        }
    }
}

fn path_matches(bindings: &[Binding], root: &str, members: &[&str], segments: &[String]) -> bool {
    for binding in bindings {
        let matched = match binding {
            Binding::Named { local, imported } if local == root && !segments.is_empty() => {
                segments
                    .first()
                    .is_some_and(|first| imported.as_str() == first.as_str())
                    && members
                        .iter()
                        .copied()
                        .eq(segments.iter().skip(1).map(String::as_str))
            }
            Binding::Namespace { local } if local == root => members
                .iter()
                .copied()
                .eq(segments.iter().map(String::as_str)),
            Binding::Default { local }
                if local == root && segments.first().map(String::as_str) == Some("default") =>
            {
                members
                    .iter()
                    .copied()
                    .eq(segments.iter().skip(1).map(String::as_str))
            }
            _ => false,
        };
        if matched {
            return true;
        }
    }
    false
}
