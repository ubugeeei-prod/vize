//! vue/require-prop-comment
//!
//! Each declared prop needs an immediately preceding JSDoc comment. Inline
//! type-literal members and same-file interfaces are checked. An unresolved
//! imported type is skipped.

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, CallExpression, Declaration, ExportDefaultDeclarationKind,
    Expression, ObjectPropertyKind, Program, PropertyKey, Statement, TSSignature, TSType,
    TSTypeName,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};
use vize_l0::{String, cstr};

static META: RuleMeta = RuleMeta {
    name: "vue/require-prop-comment",
    description: "Require a JSDoc comment on each declared prop",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/require-prop-comment`. The default comment form is JSDoc.
#[derive(Default)]
pub struct RequirePropComment;

impl Rule for RequirePropComment {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let findings = {
            let Some(descriptor) = ctx.sfc_descriptor() else {
                return;
            };
            let Some(scripts) = ctx.sfc_scripts() else {
                return;
            };
            let mut findings = Vec::new();
            if let (Some(block), Some(parsed)) = (&descriptor.script, scripts.script()) {
                findings.extend(collect_block(
                    &parsed.program,
                    block.content.as_ref(),
                    block.loc.start as u32,
                ));
            }
            if let (Some(block), Some(parsed)) = (&descriptor.script_setup, scripts.script_setup())
            {
                findings.extend(collect_block(
                    &parsed.program,
                    block.content.as_ref(),
                    block.loc.start as u32,
                ));
            }
            findings
        };
        for (message, start, end) in findings {
            ctx.report_in_sfc(LintDiagnostic::warn(META.name, message, start, end));
        }
    }
}

fn collect_block(program: &Program<'_>, source: &str, offset: u32) -> Vec<(String, u32, u32)> {
    let mut visitor = PropVisitor {
        program,
        props: Vec::new(),
    };
    visitor.visit_program(program);
    visitor
        .props
        .into_iter()
        .filter(|prop| !has_jsdoc(source, &program.comments, prop.anchor))
        .map(|prop| {
            (
                cstr!(
                    "The \"{}\" property should have a JSDoc comment.",
                    prop.name
                ),
                offset + prop.span.start,
                offset + prop.span.end,
            )
        })
        .collect()
}

struct Prop {
    name: String,
    span: Span,
    anchor: u32,
}

struct PropVisitor<'a> {
    program: &'a Program<'a>,
    props: Vec<Prop>,
}

impl<'a> Visit<'a> for PropVisitor<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        match callee_name(&call.callee) {
            Some("defineProps") => self.define_props(call),
            Some("defineComponent") => self.component_arguments(call),
            _ => {}
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_export_default_declaration(
        &mut self,
        declaration: &oxc_ast::ast::ExportDefaultDeclaration<'a>,
    ) {
        if let ExportDefaultDeclarationKind::ObjectExpression(object) = &declaration.declaration {
            self.props_option(object);
        }
        walk::walk_export_default_declaration(self, declaration);
    }
}

impl<'a> PropVisitor<'a> {
    fn define_props(&mut self, call: &CallExpression<'a>) {
        if let Some(types) = &call.type_arguments
            && let Some(ty) = types.params.first()
        {
            if let Some(props) = self.props_from_type(ty, 4) {
                self.props.extend(props);
            }
            return;
        }
        if let Some(argument) = call.arguments.first() {
            self.runtime_argument(argument);
        }
    }

    fn component_arguments(&mut self, call: &CallExpression<'a>) {
        if let Some(Argument::ObjectExpression(object)) = call.arguments.first() {
            self.props_option(object);
        }
    }

    fn props_option(&mut self, object: &oxc_ast::ast::ObjectExpression<'a>) {
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            if property_name(&property.key) != Some("props") {
                continue;
            }
            match &property.value {
                Expression::ObjectExpression(props) => {
                    for prop in &props.properties {
                        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
                            continue;
                        };
                        self.note_key(&prop.key, prop.span.start);
                    }
                }
                Expression::ArrayExpression(array) => self.array_names(array),
                _ => {}
            }
        }
    }

    fn runtime_argument(&mut self, argument: &Argument<'a>) {
        match argument {
            Argument::ObjectExpression(object) => {
                for property in &object.properties {
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        continue;
                    };
                    self.note_key(&property.key, property.span.start);
                }
            }
            Argument::ArrayExpression(array) => self.array_names(array),
            _ => {}
        }
    }

    fn array_names(&mut self, array: &oxc_ast::ast::ArrayExpression<'a>) {
        for element in &array.elements {
            let ArrayExpressionElement::StringLiteral(literal) = element else {
                continue;
            };
            self.note(&literal.value, literal.span, literal.span.start);
        }
    }

    fn props_from_type(&self, ty: &TSType<'a>, depth: u8) -> Option<Vec<Prop>> {
        if depth == 0 {
            return None;
        }
        match ty {
            TSType::TSTypeLiteral(literal) => Some(self.members(&literal.members)),
            TSType::TSParenthesizedType(inner) => {
                self.props_from_type(&inner.type_annotation, depth)
            }
            TSType::TSTypeReference(reference) => {
                let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
                    return None;
                };
                self.lookup(identifier.name.as_str(), depth - 1)
            }
            _ => None,
        }
    }

    fn members(&self, members: &[TSSignature<'a>]) -> Vec<Prop> {
        let mut props = Vec::new();
        for member in members {
            match member {
                TSSignature::TSPropertySignature(signature) => {
                    if let Some(prop) = named(&signature.key, signature.span.start) {
                        props.push(prop);
                    }
                }
                TSSignature::TSMethodSignature(signature) => {
                    if let Some(prop) = named(&signature.key, signature.span.start) {
                        props.push(prop);
                    }
                }
                _ => {}
            }
        }
        props
    }

    fn lookup(&self, name: &str, depth: u8) -> Option<Vec<Prop>> {
        for statement in &self.program.body {
            if let Some(props) = self.declaration_props(statement, name, depth) {
                return Some(props);
            }
        }
        None
    }

    fn declaration_props(
        &self,
        statement: &Statement<'a>,
        name: &str,
        depth: u8,
    ) -> Option<Vec<Prop>> {
        match statement {
            Statement::TSInterfaceDeclaration(interface) if interface.id.name == name => {
                Some(self.members(&interface.body.body))
            }
            Statement::TSTypeAliasDeclaration(alias) if alias.id.name == name => {
                self.props_from_type(&alias.type_annotation, depth)
            }
            Statement::ExportNamedDeclaration(export) => {
                export
                    .declaration
                    .as_ref()
                    .and_then(|declaration| match declaration {
                        Declaration::TSInterfaceDeclaration(interface)
                            if interface.id.name == name =>
                        {
                            Some(self.members(&interface.body.body))
                        }
                        Declaration::TSTypeAliasDeclaration(alias) if alias.id.name == name => {
                            self.props_from_type(&alias.type_annotation, depth)
                        }
                        _ => None,
                    })
            }
            _ => None,
        }
    }

    fn note_key(&mut self, key: &PropertyKey<'a>, anchor: u32) {
        if let Some(prop) = named(key, anchor) {
            self.note(&prop.name, prop.span, prop.anchor);
        }
    }

    fn note(&mut self, name: &str, span: Span, anchor: u32) {
        if name.is_empty() || self.props.iter().any(|prop| prop.span.start == span.start) {
            return;
        }
        self.props.push(Prop {
            name: String::from(name),
            span,
            anchor,
        });
    }
}

fn named(key: &PropertyKey<'_>, anchor: u32) -> Option<Prop> {
    let name = match key {
        PropertyKey::StaticIdentifier(identifier) => identifier.name.as_str(),
        PropertyKey::StringLiteral(literal) => literal.value.as_str(),
        _ => return None,
    };
    let span = key.span();
    Some(Prop {
        name: String::from(name),
        span,
        anchor: anchor.min(span.start),
    })
}

fn has_jsdoc(source: &str, comments: &[oxc_ast::ast::Comment], start: u32) -> bool {
    let Some(comment) = comments
        .iter()
        .rev()
        .find(|comment| comment.span.end <= start)
    else {
        return false;
    };
    let gap = source
        .get(comment.span.end as usize..start as usize)
        .unwrap_or("");
    if gap.chars().any(|ch| !ch.is_whitespace()) || !comment.is_block() {
        return false;
    }
    let content = comment.content_span();
    source.as_bytes().get(content.start as usize).copied() == Some(b'*')
}

fn callee_name<'a>(callee: &'a Expression<'a>) -> Option<&'a str> {
    match callee.get_inner_expression() {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
