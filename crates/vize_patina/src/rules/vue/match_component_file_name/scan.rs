//! Component `name` sites in one SFC script block.
//!
//! A definition increments the file-wide count even when it has no static
//! name. `*.component(name, ...)` compares `name` without counting on its own.
//! Reports are dropped by the caller when the count is greater than one.

use super::casing::names_match;
use crate::rules::script::script_source_type;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, CallExpression, ExportDefaultDeclaration, ExportDefaultDeclarationKind, Expression,
    NewExpression, ObjectExpression, ObjectPropertyKind, PropertyKey,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_export_default_declaration, walk_new_expression},
};
use oxc_parser::Parser;
use oxc_span::Span;
use vize_atelier_sfc::SfcScriptBlock;

#[derive(Default)]
pub(super) struct Scan {
    pub(super) count: u32,
    pub(super) misses: Vec<Miss>,
}

pub(super) struct Miss {
    pub(super) name: String,
    pub(super) start: u32,
    pub(super) end: u32,
}

pub(super) fn scan_block(block: &SfcScriptBlock<'_>, setup: bool, stem: &str, scan: &mut Scan) {
    if block.src.is_some() || block.content.is_empty() {
        return;
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, block.content.as_ref(), script_source_type()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return;
    }
    let mut scanner = Scanner {
        setup,
        stem,
        offset: block.loc.start as u32,
        scan,
    };
    scanner.visit_program(&parsed.program);
}

struct Scanner<'scan> {
    setup: bool,
    stem: &'scan str,
    offset: u32,
    scan: &'scan mut Scan,
}

impl Scanner<'_> {
    fn count_object(&mut self, object: &ObjectExpression<'_>) {
        self.scan.count += 1;
        self.verify_name_property(object);
    }

    fn verify_name_property(&mut self, object: &ObjectExpression<'_>) {
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            if !is_name_key(property.computed, &property.key) {
                continue;
            }
            if let Some((name, span)) = static_name(&property.value) {
                self.consider(&name, span);
            }
            return;
        }
    }

    fn consider(&mut self, name: &str, span: Span) {
        if names_match(name, self.stem) {
            return;
        }
        self.scan.misses.push(Miss {
            name: name.to_string(),
            start: self.offset + span.start,
            end: self.offset + span.end,
        });
    }

    fn note_identifier_call(&mut self, name: &str, call: &CallExpression<'_>) {
        if self.setup && name == "defineOptions" {
            self.scan.count += 1;
            if let Some(argument) = call.arguments.first()
                && let Some(object) = object_argument(argument)
            {
                self.verify_name_property(object);
            }
            return;
        }
        if matches!(
            name,
            "defineComponent" | "defineNuxtComponent" | "createApp" | "component"
        ) && let Some(object) = call.arguments.last().and_then(object_argument)
        {
            self.count_object(object);
        }
    }

    fn note_member_call(&mut self, call: &CallExpression<'_>) {
        let Some(member) = call.callee.get_inner_expression().as_member_expression() else {
            return;
        };
        let Some(prop) = member.static_property_name() else {
            return;
        };
        let Some(object_name) = identifier_name(member.object()) else {
            return;
        };
        let defines_component = if object_name == "Vue" {
            matches!(prop, "component" | "mixin" | "extend")
        } else {
            matches!(prop, "component" | "mixin")
        };
        if prop == "component" && call.arguments.len() == 2 {
            self.verify_argument(call.arguments.first());
        }
        if defines_component && let Some(object) = call.arguments.last().and_then(object_argument) {
            self.count_object(object);
        }
    }

    fn verify_argument(&mut self, argument: Option<&Argument<'_>>) {
        let Some(argument) = argument else {
            return;
        };
        if let Some((name, span)) = argument
            .as_expression()
            .and_then(|expression| static_name(expression))
        {
            self.consider(&name, span);
        }
    }
}

impl<'a> Visit<'a> for Scanner<'_> {
    fn visit_export_default_declaration(&mut self, it: &ExportDefaultDeclaration<'a>) {
        if !self.setup
            && let ExportDefaultDeclarationKind::ObjectExpression(object) = &it.declaration
        {
            self.count_object(object);
        }
        walk_export_default_declaration(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        match it.callee.get_inner_expression() {
            Expression::Identifier(identifier) => {
                self.note_identifier_call(identifier.name.as_str(), it);
            }
            _ => self.note_member_call(it),
        }
        walk_call_expression(self, it);
    }

    fn visit_new_expression(&mut self, it: &NewExpression<'a>) {
        if identifier_name(&it.callee) == Some("Vue")
            && let Some(object) = it.arguments.first().and_then(object_argument)
        {
            self.count_object(object);
        }
        walk_new_expression(self, it);
    }
}

fn object_argument<'a>(argument: &'a Argument<'a>) -> Option<&'a ObjectExpression<'a>> {
    match argument.as_expression()?.get_inner_expression() {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

fn static_name(expression: &Expression<'_>) -> Option<(String, Span)> {
    match expression.get_inner_expression() {
        Expression::StringLiteral(literal) => {
            Some((literal.value.as_str().to_string(), literal.span))
        }
        Expression::TemplateLiteral(template) => {
            let value = template.single_quasi()?;
            Some((value.as_str().to_string(), template.span))
        }
        _ => None,
    }
}

fn identifier_name<'a>(expression: &'a Expression<'a>) -> Option<&'a str> {
    match expression.get_inner_expression() {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn is_name_key(computed: bool, key: &PropertyKey<'_>) -> bool {
    match key {
        PropertyKey::StaticIdentifier(identifier) if !computed => identifier.name == "name",
        PropertyKey::StringLiteral(literal) => literal.value == "name",
        PropertyKey::TemplateLiteral(template) => template
            .single_quasi()
            .is_some_and(|value| value.as_str() == "name"),
        _ => false,
    }
}
