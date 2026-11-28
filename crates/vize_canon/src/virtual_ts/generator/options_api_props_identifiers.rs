//! Options API identifier `props:` support for virtual TypeScript emission.

use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Expression, Statement, VariableDeclarationKind};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{FxHashSet, String};

use super::options_api::{
    component_options_from_program, extends_base_assertion, option_expression_property,
};
use super::options_api_support::is_safe_value_identifier;

/// Type assertions spliced after authored Options API expressions, as
/// `(offset, text)` in offset order: `as const` on the initializer of an
/// identifier `props:` option, and the cast of an `extends:` base imported
/// from another SFC (see `extends_base_assertion`).
pub(super) struct PropsConstAssertions {
    offsets: Vec<(usize, String)>,
    index: usize,
}

impl PropsConstAssertions {
    pub(super) fn new(script: &str, options_api: bool) -> Self {
        let offsets = if options_api {
            find_assertion_offsets(script)
        } else {
            Vec::new()
        };
        Self { offsets, index: 0 }
    }

    pub(super) fn splice_line(&mut self, line: &str, line_start: usize) -> Option<String> {
        let offsets = self.take_line_offsets(line_start, line_start + line.len());
        if offsets.is_empty() {
            return None;
        }
        let mut output = String::default();
        let mut copied_until = 0usize;
        for (offset, assertion) in offsets {
            let column = offset - line_start;
            output.push_str(line.get(copied_until..column)?);
            output.push_str(assertion);
            copied_until = column;
        }
        output.push_str(line.get(copied_until..)?);
        Some(output)
    }

    /// Consume insertions owned by one authored line before other projections
    /// change its byte offsets.
    pub(super) fn take_line_offsets(
        &mut self,
        line_start: usize,
        line_end: usize,
    ) -> &[(usize, String)] {
        while self
            .offsets
            .get(self.index)
            .is_some_and(|&(offset, _)| offset <= line_start)
        {
            self.index += 1;
        }

        let first = self.index;
        while let Some(&(offset, _)) = self.offsets.get(self.index) {
            if offset > line_end {
                break;
            }
            self.index += 1;
        }
        self.offsets.get(first..self.index).unwrap_or_default()
    }

    pub(super) fn splice_output_line<'a>(
        &mut self,
        output_line: &mut std::borrow::Cow<'a, str>,
        line_start: usize,
    ) {
        if let Some(spliced) = self.splice_line(output_line.as_ref(), line_start) {
            *output_line = std::borrow::Cow::Owned(spliced.into());
        }
    }
}

fn find_assertion_offsets(script: &str) -> Vec<(usize, String)> {
    if !script.contains("export") || !script.contains("default") {
        return Vec::new();
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    if parsed.panicked {
        return Vec::new();
    }

    let Some(options) = component_options_from_program(&parsed.program) else {
        return Vec::new();
    };
    let mut offsets: Vec<(usize, String)> = extends_base_assertion(&parsed.program, options)
        .into_iter()
        .collect();
    let Some(props) = option_expression_property(options, "props") else {
        return offsets;
    };

    let mut prop_bindings = FxHashSet::default();
    collect_props_identifier_names(props, &mut prop_bindings);
    if prop_bindings.is_empty() {
        return offsets;
    }

    for statement in parsed.program.body.iter() {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        if declaration.kind != VariableDeclarationKind::Const {
            continue;
        }
        for declarator in declaration.declarations.iter() {
            let BindingPattern::BindingIdentifier(id) = &declarator.id else {
                continue;
            };
            if !prop_bindings.contains(id.name.as_str()) {
                continue;
            }
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            if let Some(offset) = const_assertion_offset_for_props_initializer(init) {
                offsets.push((offset, String::from(" as const")));
            }
        }
    }
    offsets.sort_unstable();
    offsets.dedup();
    offsets
}

fn collect_props_identifier_names<'a>(
    expression: &'a Expression<'a>,
    names: &mut FxHashSet<&'a str>,
) {
    match expression {
        Expression::Identifier(identifier)
            if is_safe_value_identifier(identifier.name.as_str()) =>
        {
            names.insert(identifier.name.as_str());
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            collect_props_identifier_names(&parenthesized.expression, names);
        }
        Expression::TSAsExpression(ts_as) => {
            collect_props_identifier_names(&ts_as.expression, names)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            collect_props_identifier_names(&ts_satisfies.expression, names);
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            collect_props_identifier_names(&ts_non_null.expression, names);
        }
        _ => {}
    }
}

fn const_assertion_offset_for_props_initializer(expression: &Expression<'_>) -> Option<usize> {
    match expression {
        Expression::ObjectExpression(object) => Some(object.span().end as usize),
        Expression::ParenthesizedExpression(parenthesized) => {
            const_assertion_offset_for_props_initializer(&parenthesized.expression)
        }
        _ => None,
    }
}
