//! Literal `useTemplateRef('name')` keys, checked against the template.
//!
//! The Vue toolchain types a call that is not nested in a function by indexing
//! the template's ref registry with its literal key, so a name the template
//! never registers is an error on that key. A call inside a function keeps
//! Vue's own signature and is left alone.
//!
//! The authored call is emitted verbatim, so the index access is placed in
//! front of it on the same generated line: an authored `@ts-expect-error`
//! above the call covers it, and the key maps back to its authored literal.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, CallExpression, Expression};
use oxc_ast_visit::{Visit, walk};
use oxc_span::SourceType;
use oxc_syntax::scope::ScopeFlags;
use vize_carton::String;
use vize_croquis::script_parser::parse_program_for_analysis;

use crate::virtual_ts::VizeMapping;
use crate::virtual_ts::helpers::push_ts_string_literal;

#[derive(Default)]
pub(super) struct TemplateRefKeyChecks {
    /// `(literal start, literal end, key)` in script offsets, in source order.
    keys: Vec<(usize, usize, String)>,
    next: usize,
}

impl TemplateRefKeyChecks {
    /// `registered` is whether this file declares the ref registry at all.
    pub(super) fn collect(script: Option<&str>, registered: bool) -> Self {
        let Some(script) = script.filter(|script| registered && script.contains("useTemplateRef"))
        else {
            return Self::default();
        };
        let allocator = Allocator::default();
        let parsed = parse_program_for_analysis(&allocator, script, SourceType::ts());
        if parsed.panicked {
            return Self::default();
        }
        let mut collector = Collector {
            keys: Vec::new(),
            function_depth: 0,
        };
        collector.visit_program(&parsed.program);
        collector.keys.sort_by_key(|(start, _, _)| *start);
        Self {
            keys: collector.keys,
            next: 0,
        }
    }

    /// Emit the checks of every key authored on the line `[start, end)`.
    pub(super) fn emit_for_line(
        &mut self,
        ts: &mut String,
        mappings: &mut Vec<VizeMapping>,
        (start, end): (usize, usize),
        source_offset: &dyn Fn(usize) -> usize,
    ) {
        while let Some((key_start, key_end, key)) = self.keys.get(self.next)
            && *key_start < end
        {
            self.next += 1;
            if *key_start < start {
                continue;
            }
            ts.push_str("void (null! as __VizeTemplateRefs[");
            let literal_start = ts.len();
            push_ts_string_literal(ts, key.as_str());
            mappings.push(VizeMapping {
                gen_range: literal_start..ts.len(),
                src_range: source_offset(*key_start)..source_offset(*key_end),
                sub_spans: Vec::new(),
            });
            ts.push_str("]); ");
        }
    }
}

struct Collector {
    keys: Vec<(usize, usize, String)>,
    function_depth: usize,
}

impl<'a> Visit<'a> for Collector {
    fn visit_function(&mut self, function: &oxc_ast::ast::Function<'a>, flags: ScopeFlags) {
        self.function_depth += 1;
        walk::walk_function(self, function, flags);
        self.function_depth -= 1;
    }

    fn visit_arrow_function_expression(
        &mut self,
        function: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        self.function_depth += 1;
        walk::walk_arrow_function_expression(self, function);
        self.function_depth -= 1;
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.function_depth == 0
            && matches!(&call.callee, Expression::Identifier(callee) if callee.name == "useTemplateRef")
            && let Some(Argument::StringLiteral(key)) = call.arguments.first()
        {
            self.keys.push((
                key.span.start as usize,
                key.span.end as usize,
                String::from(key.value.as_str()),
            ));
        }
        walk::walk_call_expression(self, call);
    }
}

#[cfg(test)]
mod tests {
    use super::TemplateRefKeyChecks;

    #[test]
    fn only_literal_keys_outside_functions_are_checked() {
        let script = "const a = useTemplateRef('a')\nfunction f() { useTemplateRef('inner') }\nconst g = () => useTemplateRef('arrow')\nuseTemplateRef(name)\nuseTemplateRef<HTMLElement>('typed')\n";
        let checks = TemplateRefKeyChecks::collect(Some(script), true);
        let keys: Vec<(&str, &str)> = checks
            .keys
            .iter()
            .map(|(start, end, key)| (&script[*start..*end], key.as_str()))
            .collect();
        assert_eq!(keys, [("'a'", "a"), ("'typed'", "typed")]);
        assert!(
            TemplateRefKeyChecks::collect(Some(script), false)
                .keys
                .is_empty()
        );
    }
}
