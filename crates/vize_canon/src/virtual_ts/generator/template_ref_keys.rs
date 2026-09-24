//! Literal `useTemplateRef('name')` keys, checked against the template.
//!
//! The Vue toolchain types a call that is not nested in a function by indexing
//! the template's ref registry with its literal key, so a name the template
//! never registers is an error on that key. A call inside a function keeps
//! Vue's own signature and is left alone.
//!
//! The authored call is emitted verbatim, so the index access is placed in
//! front of the statement that holds it, on the generated line the statement
//! starts on: an authored `@ts-expect-error` above the statement covers it, and
//! the key maps back to its authored literal. The call itself may span lines
//! (`useTemplateRef<{\n…\n}>('key')`), so the line of the key is not a place a
//! statement can be inserted at; a statement that shares its first line with
//! other code is left unchecked for the same reason.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, CallExpression, Expression, Statement};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, SourceType};
use oxc_syntax::scope::ScopeFlags;
use vize_carton::String;
use vize_croquis::script_parser::parse_program_for_analysis;

use crate::virtual_ts::VizeMapping;
use crate::virtual_ts::helpers::push_ts_string_literal;

#[derive(Default)]
pub(super) struct TemplateRefKeyChecks {
    /// In script offsets, ordered by the statement they are checked in front of.
    keys: Vec<TemplateRefKey>,
    next: usize,
}

struct TemplateRefKey {
    /// Where the top-level statement holding the call starts.
    statement_start: usize,
    /// The authored string literal.
    literal: (usize, usize),
    key: String,
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
            script,
            keys: Vec::new(),
            function_depth: 0,
            statement_depth: 0,
            statement_start: 0,
        };
        collector.visit_program(&parsed.program);
        collector
            .keys
            .sort_by_key(|key| (key.statement_start, key.literal.0));
        Self {
            keys: collector.keys,
            next: 0,
        }
    }

    /// Emit the checks of every statement that starts on the line `[start, end)`.
    pub(super) fn emit_for_line(
        &mut self,
        ts: &mut String,
        mappings: &mut Vec<VizeMapping>,
        (start, end): (usize, usize),
        source_offset: &dyn Fn(usize) -> usize,
    ) {
        while let Some(key) = self.keys.get(self.next)
            && key.statement_start < end
        {
            self.next += 1;
            if key.statement_start < start {
                continue;
            }
            ts.push_str("void (null! as __VizeTemplateRefs[");
            let literal_start = ts.len();
            push_ts_string_literal(ts, key.key.as_str());
            mappings.push(VizeMapping {
                gen_range: literal_start..ts.len(),
                src_range: source_offset(key.literal.0)..source_offset(key.literal.1),
                sub_spans: Vec::new(),
            });
            ts.push_str("]); ");
        }
    }
}

struct Collector<'s> {
    script: &'s str,
    keys: Vec<TemplateRefKey>,
    function_depth: usize,
    statement_depth: usize,
    /// Where the top-level statement being visited starts.
    statement_start: usize,
}

impl Collector<'_> {
    /// Whether nothing but indentation precedes the current statement on its
    /// line, so that the start of the line is a statement boundary.
    fn statement_opens_its_line(&self) -> bool {
        let Some(before) = self.script.get(..self.statement_start) else {
            return false;
        };
        let line = before.rsplit_once('\n').map_or(before, |(_, line)| line);
        line.trim().is_empty()
    }
}

impl<'a> Visit<'a> for Collector<'_> {
    fn visit_statement(&mut self, statement: &Statement<'a>) {
        if self.statement_depth == 0 {
            self.statement_start = statement.span().start as usize;
        }
        self.statement_depth += 1;
        walk::walk_statement(self, statement);
        self.statement_depth -= 1;
    }

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
            && self.statement_opens_its_line()
        {
            self.keys.push(TemplateRefKey {
                statement_start: self.statement_start,
                literal: (key.span.start as usize, key.span.end as usize),
                key: String::from(key.value.as_str()),
            });
        }
        walk::walk_call_expression(self, call);
    }
}

#[expect(clippy::string_slice, reason = "tests assert by panicking")]
#[cfg(test)]
mod tests {
    use super::TemplateRefKeyChecks;
    use vize_carton::String;

    fn authored<'a>(checks: &'a TemplateRefKeyChecks, script: &'a str) -> Vec<(&'a str, &'a str)> {
        checks
            .keys
            .iter()
            .map(|key| (&script[key.literal.0..key.literal.1], key.key.as_str()))
            .collect()
    }

    /// The generated script, with the checks in front of the lines they belong to.
    fn generate(script: &str) -> String {
        let mut checks = TemplateRefKeyChecks::collect(Some(script), true);
        let mut ts = String::default();
        let mut mappings = Vec::new();
        let mut line_start = 0;
        for line in script.split_inclusive('\n') {
            let line_end = line_start + line.len();
            checks.emit_for_line(&mut ts, &mut mappings, (line_start, line_end), &|offset| {
                offset
            });
            ts.push_str(line);
            line_start = line_end;
        }
        for mapping in &mappings {
            assert_eq!(
                ts[mapping.gen_range.clone()].replace('"', "'"),
                script[mapping.src_range.clone()]
            );
        }
        ts
    }

    #[test]
    fn only_literal_keys_outside_functions_are_checked() {
        let script = "const a = useTemplateRef('a')\nfunction f() { useTemplateRef('inner') }\nconst g = () => useTemplateRef('arrow')\nuseTemplateRef(name)\nuseTemplateRef<HTMLElement>('typed')\n";
        let checks = TemplateRefKeyChecks::collect(Some(script), true);
        assert_eq!(
            authored(&checks, script),
            [("'a'", "a"), ("'typed'", "typed")]
        );
        assert!(
            TemplateRefKeyChecks::collect(Some(script), false)
                .keys
                .is_empty()
        );
    }

    #[test]
    fn a_call_that_spans_lines_is_checked_in_front_of_its_statement() {
        assert_eq!(
            generate(
                "const modal = useTemplateRef<{\n  close: () => void\n}>('modal')\nif (ok) {\n  useTemplateRef('nested')\n}\n"
            ),
            "void (null! as __VizeTemplateRefs[\"modal\"]); const modal = useTemplateRef<{\n  close: () => void\n}>('modal')\nvoid (null! as __VizeTemplateRefs[\"nested\"]); if (ok) {\n  useTemplateRef('nested')\n}\n"
        );
    }

    #[test]
    fn a_statement_that_shares_its_first_line_is_left_unchecked() {
        let script = "const a = [\n  1]; const b = useTemplateRef('b')\n  const c = useTemplateRef('c'), d = useTemplateRef('d')\n";
        assert_eq!(
            generate(script),
            "const a = [\n  1]; const b = useTemplateRef('b')\nvoid (null! as __VizeTemplateRefs[\"c\"]); void (null! as __VizeTemplateRefs[\"d\"]);   const c = useTemplateRef('c'), d = useTemplateRef('d')\n"
        );
    }
}
