//! Svelte 5 runes in the instance script and in template expressions.
//!
//! `let x = $state(v)`, `let y = $derived(e)` and `let { p } = $props()` are
//! the top-level declarations this proof of concept reads. Every later
//! reference becomes a runtime signal access: `$.get(x)`, `$.set(x, v)`,
//! `$.update(x)` and `$$props.p`. A rune name that is shadowed, a rune call
//! anywhere else, and an assignment to a derived value or a prop are refused.

use oxc_ast::ast::{
    Argument, AssignmentExpression, AssignmentOperator, AssignmentTarget, BindingIdentifier,
    BindingPattern, Expression, IdentifierReference, ObjectProperty, SimpleAssignmentTarget,
    Statement, UpdateExpression, UpdateOperator, VariableDeclaration,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use poc_support::Error;
use vize_l0::{Allocator, Span, String, cstr};

const RUNE_CALLS: [&str; 3] = ["$state", "$derived", "$props"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rune {
    State,
    Derived,
    Prop,
    /// A block parameter that the runtime passes as a signal (`{#each}` items).
    Signal,
}

/// The rune bindings declared by one instance script.
#[derive(Default)]
pub(crate) struct Runes {
    names: Vec<(String, Rune)>,
}

impl Runes {
    pub(crate) fn declares(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    fn get(&self, name: &str) -> Option<Rune> {
        self.names
            .iter()
            .find(|(declared, _)| declared == name)
            .map(|(_, rune)| *rune)
    }
}

/// Read the runes of `script` (authored at file offset `base`) and return it
/// rewritten for the Svelte client runtime.
pub(crate) fn script(script: &str, base: u32) -> Result<(Runes, String), Error> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::mjs()).parse();
    if let Some(error) = parsed.diagnostics.iter().next() {
        return Err(Error::new(cstr!("{error}"), Span::new(base, base)));
    }
    let mut runes = Runes::default();
    for statement in &parsed.program.body {
        if let Some((kind, pattern, _)) = declaration(statement) {
            collect(&mut runes, kind, pattern).map_err(|error| shift(error, base))?;
        }
    }
    let mut rewrite = Rewrite::new(&runes, &[]);
    for statement in &parsed.program.body {
        match declaration(statement) {
            Some((Rune::Prop, _, _)) => {
                // The whole line goes: its indentation, the statement and its break.
                let span = statement.span();
                let line = script.get(..span.start as usize).unwrap_or_default();
                let indentation = line.len() - line.trim_end_matches([' ', '\t']).len();
                let start = span.start - u32::try_from(indentation).unwrap_or(0);
                let end = match script.as_bytes().get(span.end as usize) {
                    Some(b'\n') => span.end + 1,
                    _ => span.end,
                };
                rewrite.edit(Span::new(start, end), String::default());
            }
            Some((rune, _, call)) => {
                let Some(argument) = call.arguments.first().and_then(Argument::as_expression)
                else {
                    return Err(Error::new(
                        "a rune needs one argument",
                        shift_span(call.span, base),
                    ));
                };
                let callee = cstr!(
                    "{}",
                    if rune == Rune::State {
                        "$.state("
                    } else {
                        "$.derived(() => "
                    }
                );
                rewrite.edit(
                    Span::new(call.callee.span().start, argument.span().start),
                    callee,
                );
                rewrite.visit_expression(argument);
            }
            None => rewrite.visit_statement(statement),
        }
    }
    let text = rewrite.apply(script).map_err(|error| shift(error, base))?;
    Ok((runes, text))
}

/// Rewrite one template expression. `signals` are enclosing block parameters.
pub(crate) fn expression(
    text: &str,
    span: Span,
    runes: &Runes,
    signals: &[&str],
) -> Result<String, Error> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, SourceType::mjs())
        .parse_expression()
        .map_err(|_| Error::new("unsupported expression", span))?;
    let mut rewrite = Rewrite::new(runes, signals);
    rewrite.visit_expression(&parsed);
    rewrite
        .apply(text)
        .map_err(|error| Error::new(error.message, span))
}

type RuneDeclaration<'s, 'a> = (
    Rune,
    &'s BindingPattern<'a>,
    &'s oxc_ast::ast::CallExpression<'a>,
);

fn declaration<'s, 'a>(statement: &'s Statement<'a>) -> Option<RuneDeclaration<'s, 'a>> {
    let Statement::VariableDeclaration(declaration) = statement else {
        return None;
    };
    let VariableDeclaration { declarations, .. } = &**declaration;
    let [declarator] = declarations.as_slice() else {
        return None;
    };
    let Some(Expression::CallExpression(call)) = &declarator.init else {
        return None;
    };
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    let rune = match callee.name.as_str() {
        "$state" => Rune::State,
        "$derived" => Rune::Derived,
        "$props" => Rune::Prop,
        _ => return None,
    };
    Some((rune, &declarator.id, call))
}

fn collect(runes: &mut Runes, rune: Rune, pattern: &BindingPattern<'_>) -> Result<(), Error> {
    match (rune, pattern) {
        (Rune::Prop, BindingPattern::ObjectPattern(object)) if object.rest.is_none() => {
            for property in &object.properties {
                let BindingPattern::BindingIdentifier(name) = &property.value else {
                    return Err(Error::new(
                        "unsupported prop pattern",
                        shift_span(property.span, 0),
                    ));
                };
                runes
                    .names
                    .push((String::from(name.name.as_str()), Rune::Prop));
            }
            Ok(())
        }
        (Rune::State | Rune::Derived, BindingPattern::BindingIdentifier(name)) => {
            runes.names.push((String::from(name.name.as_str()), rune));
            Ok(())
        }
        _ => Err(Error::new(
            "unsupported rune declaration",
            shift_span(pattern.span(), 0),
        )),
    }
}

struct Rewrite<'r> {
    runes: &'r Runes,
    signals: &'r [&'r str],
    edits: Vec<(Span, String)>,
    error: Option<Error>,
}

impl<'r> Rewrite<'r> {
    fn new(runes: &'r Runes, signals: &'r [&'r str]) -> Self {
        Self {
            runes,
            signals,
            edits: Vec::new(),
            error: None,
        }
    }

    fn rune(&self, name: &str) -> Option<Rune> {
        if self.signals.contains(&name) {
            Some(Rune::Signal)
        } else {
            self.runes.get(name)
        }
    }

    fn edit(&mut self, span: Span, text: String) {
        self.edits.push((span, text));
    }

    fn fail(&mut self, message: &str, span: oxc_span::Span) {
        self.error
            .get_or_insert_with(|| Error::new(message, shift_span(span, 0)));
    }

    fn apply(mut self, text: &str) -> Result<String, Error> {
        if let Some(error) = self.error {
            return Err(error);
        }
        self.edits.sort_by_key(|(span, _)| (span.start, span.end));
        let mut out = String::with_capacity(text.len());
        let mut cursor = 0;
        for (span, replacement) in &self.edits {
            let start = span.start as usize;
            let kept = text
                .get(cursor..start)
                .ok_or_else(|| Error::new("overlapping rewrite", *span))?;
            out.push_str(kept);
            out.push_str(replacement);
            cursor = span.end as usize;
        }
        out.push_str(text.get(cursor..).unwrap_or_default());
        Ok(out)
    }
}

fn read(name: &str, rune: Rune) -> String {
    match rune {
        Rune::Prop => cstr!("$$props.{name}"),
        _ => cstr!("$.get({name})"),
    }
}

impl<'a> Visit<'a> for Rewrite<'_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        let name = it.name.as_str();
        if RUNE_CALLS.contains(&name) {
            self.fail("a rune outside a top-level declaration", it.span);
        } else if let Some(rune) = self.rune(name) {
            self.edit(shift_span(it.span, 0), read(name, rune));
        }
    }

    fn visit_object_property(&mut self, it: &ObjectProperty<'a>) {
        if let (true, Expression::Identifier(value)) = (it.shorthand, &it.value) {
            let name = value.name.as_str();
            if let Some(rune) = self.rune(name) {
                let text = cstr!("{name}: {}", read(name, rune));
                self.edit(shift_span(value.span, 0), text);
                return;
            }
        }
        walk::walk_object_property(self, it);
    }

    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        let AssignmentTarget::AssignmentTargetIdentifier(target) = &it.left else {
            return walk::walk_assignment_expression(self, it);
        };
        let name = target.name.as_str();
        match self.rune(name) {
            None => walk::walk_assignment_expression(self, it),
            Some(Rune::State) => {
                let right = it.right.span();
                let prefix = match it.operator {
                    AssignmentOperator::Assign => cstr!("$.set({name}, "),
                    operator => cstr!(
                        "$.set({name}, $.get({name}) {} ",
                        operator.as_str().trim_end_matches('=')
                    ),
                };
                self.edit(Span::new(it.span.start, right.start), prefix);
                self.visit_expression(&it.right);
                self.edit(Span::new(right.end, it.span.end), String::from(")"));
            }
            Some(_) => self.fail("only a $state binding is assignable", it.span),
        }
    }

    fn visit_update_expression(&mut self, it: &UpdateExpression<'a>) {
        let SimpleAssignmentTarget::AssignmentTargetIdentifier(target) = &it.argument else {
            return walk::walk_update_expression(self, it);
        };
        let name = target.name.as_str();
        match self.rune(name) {
            None => walk::walk_update_expression(self, it),
            Some(Rune::State) => {
                let call = if it.prefix {
                    "$.update_pre"
                } else {
                    "$.update"
                };
                let delta = match it.operator {
                    UpdateOperator::Increment => "",
                    UpdateOperator::Decrement => ", -1",
                };
                self.edit(shift_span(it.span, 0), cstr!("{call}({name}{delta})"));
            }
            Some(_) => self.fail("only a $state binding is assignable", it.span),
        }
    }

    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        if self.rune(it.name.as_str()).is_some() {
            self.fail("a declaration shadows a rune binding", it.span);
        }
    }
}

fn shift_span(span: oxc_span::Span, base: u32) -> Span {
    Span::new(span.start + base, span.end + base)
}

fn shift(error: Error, base: u32) -> Error {
    Error::new(
        error.message,
        Span::new(error.span.start + base, error.span.end + base),
    )
}

#[cfg(test)]
mod tests {
    use super::{Runes, expression, script};
    use vize_l0::Span;

    fn runes(source: &str) -> Runes {
        script(source, 0)
            .map(|(runes, _)| runes)
            .unwrap_or_default()
    }

    #[test]
    fn rune_declarations_and_references_become_signal_accesses() {
        let source = "let { title } = $props();\nlet count = $state(0);\nlet big = $derived(count > 3);\nfunction inc() { count += 1; count++; }\n";
        let rewritten = script(source, 0).map(|(_, text)| text);
        assert_eq!(
            rewritten.as_deref(),
            Ok(
                "let count = $.state(0);\nlet big = $.derived(() => $.get(count) > 3);\nfunction inc() { $.set(count, $.get(count) + 1); $.update(count); }\n"
            )
        );
        let runes = runes(source);
        let at = Span::new(0, 0);
        assert_eq!(
            expression("{ title, n: big }", at, &runes, &[]).as_deref(),
            Ok("{ title: $$props.title, n: $.get(big) }")
        );
        assert_eq!(
            expression("item + count", at, &runes, &["item"]).as_deref(),
            Ok("$.get(item) + $.get(count)")
        );
    }

    #[test]
    fn shadowing_and_derived_writes_are_refused() {
        let runes = runes("let count = $state(0);\nlet big = $derived(count);\n");
        let at = Span::new(0, 0);
        assert!(expression("(count) => count", at, &runes, &[]).is_err());
        assert!(expression("big = 1", at, &runes, &[]).is_err());
        assert!(expression("$state(1)", at, &runes, &[]).is_err());
    }
}
