//! Keep nested parameter echoes inside the unmapped prop-inference copy.
//!
//! Directives go inside parameter parentheses, never before an expression or
//! into a body. Body tokens and existing line breaks remain intact, including
//! return/ASI, line comments, JSX and template literal contents.

use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Expression, FormalParameters};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::String;

/// Ignores an inference-only nested callback parameter.
const IGNORE: &str = "\n// @ts-ignore\n";

pub(super) fn rewrite(value: &str) -> Option<String> {
    // A new directive must not consume an authored expect-error or turn it
    // into an unused directive. This unsupported copy retains the old bytes.
    if value.contains("@ts-expect-error") || !value.contains(['\n', '\r', '\u{2028}', '\u{2029}']) {
        return None;
    }
    let allocator = Allocator::default();
    let expression = Parser::new(&allocator, value, SourceType::ts())
        .parse_expression()
        .or_else(|_| Parser::new(&allocator, value, SourceType::tsx()).parse_expression())
        .ok()?;
    let mut visitor = Parameters {
        value,
        depth: 0,
        edits: Vec::new(),
    };
    visitor.visit_expression(&expression);
    if visitor.edits.is_empty() {
        return None;
    }
    visitor.edits.sort_unstable_by_key(|edit| edit.0);
    let mut text = String::with_capacity(value.len());
    let mut copied = 0;
    for (offset, insertion) in visitor.edits {
        text.push_str(value.get(copied..offset)?);
        text.push_str(insertion);
        copied = offset;
    }
    text.push_str(value.get(copied..)?);
    Some(text)
}

struct Parameters<'s> {
    value: &'s str,
    depth: usize,
    edits: Vec<(usize, &'static str)>,
}

impl Parameters<'_> {
    fn collect(&mut self, params: &FormalParameters<'_>) {
        if self.depth == 0 {
            return;
        }
        let Some(source) = self
            .value
            .get(params.span.start as usize..params.span.end as usize)
        else {
            return;
        };
        let parenthesized = source.starts_with('(') && source.ends_with(')');
        for parameter in &params.items {
            let BindingPattern::BindingIdentifier(binding) = &parameter.pattern else {
                continue;
            };
            if parameter.type_annotation.is_some()
                || parameter.initializer.is_some()
                || !parameter.decorators.is_empty()
            {
                continue;
            }
            let start = binding.span.start as usize;
            let end = binding.span.end as usize;
            // The caller's existing prop directive already covers line zero.
            // A trailing newline alone must not change a one-line callback.
            if !self
                .value
                .get(..start)
                .is_some_and(|prefix| prefix.contains(['\n', '\r', '\u{2028}', '\u{2029}']))
            {
                continue;
            }
            // An unparenthesized arrow admits exactly one plain identifier.
            // No guessed range or recovery result can mint punctuation.
            if !parenthesized
                && (params.items.len() != 1 || params.rest.is_some() || params.span != binding.span)
            {
                continue;
            }
            self.edits.push((
                start,
                if parenthesized {
                    IGNORE
                } else {
                    // Inference-only nested callback parameter.
                    "(\n// @ts-ignore\n"
                },
            ));
            if !parenthesized {
                self.edits.push((end, ")"));
            }
        }
    }
}

impl<'a> Visit<'a> for Parameters<'_> {
    fn visit_expression(&mut self, expression: &Expression<'a>) {
        let params = match expression {
            Expression::ArrowFunctionExpression(arrow) => &arrow.params,
            Expression::FunctionExpression(function) => &function.params,
            _ => {
                walk::walk_expression(self, expression);
                return;
            }
        };
        self.collect(params);
        self.depth += 1;
        walk::walk_expression(self, expression);
        self.depth -= 1;
    }
}

#[cfg(test)]
mod tests;
