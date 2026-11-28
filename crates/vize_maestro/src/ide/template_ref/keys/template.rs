//! Key spellings in an HTML template: `ref="box"`, `:ref="'box'"`, and
//! `$refs.box` / `$refs['box']` in expressions.

use oxc_allocator::Allocator;
use oxc_ast::ast::{ComputedMemberExpression, Expression, StaticMemberExpression};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};

use super::{Site, Spellings, every_refs_is_known, verbatim_key};

pub(super) fn collect(content: &str, base: usize, spellings: &mut Spellings) {
    let allocator = vize_l0::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, content);
    if errors.iter().any(|error| !error.is_recoverable()) {
        spellings.complete = false;
    }
    Collector {
        content,
        base,
        spellings,
    }
    .children(&root.children);
}

struct Collector<'s> {
    content: &'s str,
    base: usize,
    spellings: &'s mut Spellings,
}

impl Collector<'_> {
    fn children(&mut self, children: &[TemplateChildNode<'_>]) {
        for child in children {
            match child {
                TemplateChildNode::Element(element) => {
                    for prop in &element.props {
                        self.prop(prop);
                    }
                    self.children(&element.children);
                }
                TemplateChildNode::Interpolation(interpolation) => {
                    self.expression(&interpolation.content);
                }
                _ => {}
            }
        }
    }

    fn prop(&mut self, prop: &PropNode<'_>) {
        match prop {
            PropNode::Attribute(attribute) if attribute.name == "ref" => {
                let Some(value) = attribute.value.as_ref() else {
                    self.spellings.complete = false;
                    return;
                };
                let (start, end) = (value.loc.span.start as usize, value.loc.span.end as usize);
                match unquoted(self.content, start, end) {
                    // An entity-encoded key has no single span to rewrite.
                    Some((start, end)) if self.content.get(start..end) == Some(value.content) => {
                        self.push(value.content, start, end, Site::Attribute);
                    }
                    _ => self.spellings.complete = false,
                }
            }
            PropNode::Attribute(_) => {}
            PropNode::Directive(directive) => {
                let arg = match &directive.arg {
                    Some(ExpressionNode::Simple(arg)) => Some(arg),
                    _ => None,
                };
                if directive.name == "bind" && arg.is_some_and(|arg| arg.content == "ref") {
                    // A string literal is one key; anything else may be any key.
                    if !arg.is_some_and(|arg| arg.is_static)
                        || !directive
                            .exp
                            .as_ref()
                            .is_some_and(|exp| self.bound_literal(exp))
                    {
                        self.spellings.complete = false;
                    }
                    return;
                }
                if let Some(dynamic) = directive.arg.as_ref()
                    && arg.is_none_or(|arg| !arg.is_static)
                {
                    self.expression(dynamic);
                }
                if let Some(exp) = directive.exp.as_ref() {
                    self.expression(exp);
                }
            }
        }
    }

    /// `:ref="'box'"`: the key between the literal's quotes.
    fn bound_literal(&mut self, exp: &ExpressionNode<'_>) -> bool {
        let ExpressionNode::Simple(exp) = exp else {
            return false;
        };
        let start = exp.loc.span.start as usize;
        if self
            .content
            .get(start..start + exp.content.len())
            .is_none_or(|source| source != exp.content)
        {
            return false;
        }
        let trimmed = exp.content.trim_start();
        let start = start + exp.content.len() - trimmed.len();
        let trimmed = trimmed.trim_end();
        let Some(quote @ ('\'' | '"')) = trimmed.chars().next() else {
            return false;
        };
        let Some(key) = trimmed
            .strip_prefix(quote)
            .and_then(|rest| rest.strip_suffix(quote))
        else {
            return false;
        };
        if key.contains([quote, '\\', '\n', '\r']) {
            return false;
        }
        let Some((start, end)) = verbatim_key(self.content, start, start + trimmed.len(), key)
        else {
            return false;
        };
        self.push(key, start, end, Site::Attribute);
        true
    }

    /// `$refs.box` and `$refs['box']` in an expression that mentions `$refs`.
    fn expression(&mut self, exp: &ExpressionNode<'_>) {
        let ExpressionNode::Simple(exp) = exp else {
            return;
        };
        if !exp.content.contains("$refs") {
            return;
        }
        let start = exp.loc.span.start as usize;
        if self.content.get(start..start + exp.content.len()) != Some(exp.content) {
            self.spellings.complete = false;
            return;
        }
        let allocator = Allocator::default();
        let mut reads = Reads {
            content: exp.content,
            found: Vec::new(),
            refs: Vec::new(),
            escaped: false,
        };
        // A handler may be a statement list rather than one expression, and
        // an expression parse stops after the first statement.
        match Parser::new(&allocator, exp.content, SourceType::ts()).parse_expression() {
            Ok(expression) if expression.span().end as usize >= exp.content.trim_end().len() => {
                reads.visit_expression(&expression);
            }
            _ => {
                let parsed = Parser::new(&allocator, exp.content, SourceType::ts()).parse();
                if parsed.panicked || !parsed.diagnostics.is_empty() {
                    self.spellings.complete = false;
                    return;
                }
                reads.visit_program(&parsed.program);
            }
        }
        if !reads.complete() {
            self.spellings.complete = false;
        }
        for (key, key_start, key_end, site) in reads.found {
            self.push(key, start + key_start, start + key_end, site);
        }
    }

    fn push(&mut self, key: &str, start: usize, end: usize, site: Site) {
        self.spellings
            .push(key, self.base + start, self.base + end, site);
    }
}

/// `$refs` reads in one template expression.
struct Reads<'s> {
    content: &'s str,
    /// Key, span in the expression, and site.
    found: Vec<(&'s str, usize, usize, Site)>,
    /// Offsets of the `$refs` names that start a known spelling.
    refs: Vec<usize>,
    escaped: bool,
}

impl Reads<'_> {
    fn complete(&self) -> bool {
        !self.escaped && every_refs_is_known(self.content, &self.refs)
    }
}

/// The offset of `$refs` when `object` is the bare `$refs`.
fn refs(object: &Expression<'_>) -> Option<usize> {
    match object {
        Expression::Identifier(name) if name.name == "$refs" => Some(name.span.start as usize),
        _ => None,
    }
}

impl<'a, 's> Visit<'a> for Reads<'s> {
    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if let Some(refs) = refs(&member.object) {
            let span = member.property.span;
            let (start, end) = (span.start as usize, span.end as usize);
            if let Some(key) = self.content.get(start..end) {
                self.refs.push(refs);
                self.found.push((key, start, end, Site::Member));
                return;
            }
        }
        walk::walk_static_member_expression(self, member);
    }

    fn visit_computed_member_expression(&mut self, member: &ComputedMemberExpression<'a>) {
        if let Some(refs) = refs(&member.object)
            && let Expression::StringLiteral(literal) = &member.expression
        {
            let (start, end) = (literal.span.start as usize, literal.span.end as usize);
            match verbatim_key(self.content, start, end, literal.value.as_str()) {
                Some((start, end)) => {
                    if let Some(key) = self.content.get(start..end) {
                        self.refs.push(refs);
                        self.found.push((key, start, end, Site::Index));
                    }
                }
                None => self.escaped = true,
            }
            return;
        }
        walk::walk_computed_member_expression(self, member);
    }
}

/// The span inside an attribute value's quotes.
fn unquoted(content: &str, start: usize, end: usize) -> Option<(usize, usize)> {
    let raw = content.get(start..end)?;
    match raw.as_bytes().first() {
        Some(&quote @ (b'"' | b'\''))
            if raw.len() >= 2 && raw.as_bytes().last() == Some(&quote) =>
        {
            Some((start + 1, end - 1))
        }
        Some(b'"' | b'\'') => None,
        _ => Some((start, end)),
    }
}
