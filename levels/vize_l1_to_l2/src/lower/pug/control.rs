//! pug's control flow run at build time, as `pug-code-gen@3.0.4` runs
//! it: `if` / `unless` with their `else if` / `else` chain, and
//! `each` / `for` (`in`, with an optional `else`, or `of`). Branches pug
//! skips still have their expressions syntax-checked ([`super::check`]).

use alloc::vec::Vec as StdVec;

use vize_l0::{Span, String, cstr};
use vize_l1::pug::{PugNode, PugRefusal, PugRefused};

use super::emit::Emitter;
use super::eval;
use super::heads::{Branch, Each, branch, each};
use super::value::Value;

fn is_else<'n, 'a>(node: &'n PugNode<'a>) -> Option<&'n PugRefused<'a>> {
    match node {
        PugNode::Refused(refused)
            if refused.refusal == PugRefusal::Conditional
                && branch(refused.head.text) == Ok(Branch::Else) =>
        {
            Some(refused)
        }
        _ => None,
    }
}

impl<'t> Emitter<'t> {
    /// Run a conditional chain or a loop headed by `head`; returns how
    /// many of the following siblings (`else if` / `else`) it consumed.
    pub(super) fn control(&mut self, head: &'t PugRefused<'t>, rest: &'t [PugNode<'t>]) -> usize {
        match head.refusal {
            PugRefusal::Each => self.each(head, rest),
            _ => self.conditional(head, rest),
        }
    }

    fn conditional(&mut self, first: &'t PugRefused<'t>, rest: &'t [PugNode<'t>]) -> usize {
        let head_span = self.span(&first.head);
        let (test, negate) = match branch(first.head.text) {
            Ok(Branch::If { test, negate }) => (test, negate),
            Ok(Branch::ElseIf(_)) => {
                self.error(head_span, String::from("unexpected token \"else-if\""));
                return 0;
            }
            Ok(Branch::Else) => {
                self.error(head_span, String::from("unexpected token \"else\""));
                return 0;
            }
            Err(message) => {
                self.error(head_span, String::from(message));
                return 0;
            }
        };
        let mut chain = alloc::vec![(first, Some((test, negate)))];
        for node in rest {
            let PugNode::Refused(next) = node else {
                break;
            };
            if next.refusal != PugRefusal::Conditional {
                break;
            }
            match branch(next.head.text) {
                Ok(Branch::ElseIf(test)) => chain.push((next, Some((test, false)))),
                Ok(Branch::Else) => {
                    chain.push((next, None));
                    break;
                }
                _ => break,
            }
        }
        let consumed = chain.len() - 1;
        let mut taken = None;
        for (index, (_, test)) in chain.iter().enumerate() {
            let Some((test, negate)) = test else {
                taken = Some(index);
                break;
            };
            match eval::expression(&self.scope, test) {
                Ok(value) if value.value.truthy() != *negate => {
                    taken = Some(index);
                    break;
                }
                Ok(_) => {}
                Err(reason) => {
                    let keyword = if *negate { "unless" } else { "if" };
                    let span = self.text_span(test);
                    self.error(span, super::refusal::condition(keyword, &reason));
                    return consumed;
                }
            }
        }
        for (index, (node, test)) in chain.iter().enumerate() {
            if Some(index) == taken {
                self.scope.push(false);
                self.nodes(&node.block);
                self.scope.pop();
            } else {
                if taken.is_some_and(|taken| index > taken)
                    && let Some((test, _)) = test
                {
                    self.check_expression(test);
                }
                self.check(&node.block);
            }
        }
        consumed
    }

    fn each(&mut self, head: &'t PugRefused<'t>, rest: &'t [PugNode<'t>]) -> usize {
        let head_span = self.span(&head.head);
        let parsed = match each(head.head.text) {
            Ok(parsed) => parsed,
            Err(message) => {
                self.error(head_span, String::from(message));
                return 0;
            }
        };
        if head.block.is_empty() {
            self.error(head_span, String::from(super::refusal::EACH_BLOCK));
            return 0;
        }
        match parsed {
            Each::In {
                value,
                key,
                subject,
            } => {
                let alternate = rest.first().and_then(is_else);
                if alternate.is_some_and(|alternate| alternate.block.is_empty()) {
                    self.error(head_span, String::from(super::refusal::EACH_BLOCK));
                }
                self.each_in(head, value, key, subject, alternate);
                usize::from(alternate.is_some())
            }
            Each::Of {
                names,
                pair,
                subject,
            } => {
                self.each_of(head, &names, pair, subject);
                0
            }
        }
    }

    fn subject(&mut self, subject: &str) -> Option<Value> {
        match eval::expression(&self.scope, subject) {
            Ok(value) => Some(value.value),
            Err(reason) => {
                let span = self.text_span(subject);
                self.error(span, super::refusal::each(&reason));
                None
            }
        }
    }

    fn each_in(
        &mut self,
        head: &'t PugRefused<'t>,
        value: &str,
        key: Option<&str>,
        subject: &str,
        alternate: Option<&'t PugRefused<'t>>,
    ) {
        if key == Some(value) {
            let span = self.span(&head.head);
            self.error(span, String::from(super::refusal::EACH_SAME_NAMES));
            return;
        }
        let Some(object) = self.subject(subject) else {
            return;
        };
        let entries: StdVec<(Value, Value)> = match object {
            Value::Array(items) => items
                .into_iter()
                .enumerate()
                .map(|(index, item)| (Value::Number(index as f64), item))
                .collect(),
            Value::Str(text) => {
                let mut entries = StdVec::new();
                for (index, unit) in text.encode_utf16().enumerate() {
                    let Some(ch) = char::from_u32(u32::from(unit)) else {
                        let span = self.text_span(subject);
                        let reason = String::from("it splits a surrogate pair");
                        self.error(span, super::refusal::each(&reason));
                        return;
                    };
                    let mut buffer = [0; 4];
                    entries.push((
                        Value::Number(index as f64),
                        Value::str(ch.encode_utf8(&mut buffer)),
                    ));
                }
                entries
            }
            Value::Object(entries) => entries
                .into_iter()
                .map(|(key, item)| (Value::Str(key), item))
                .collect(),
            // A function's `length` (a call's `block` has none) is a number.
            Value::Number(_) | Value::Bool(_) | Value::Function(_) => StdVec::new(),
            Value::Null | Value::Undefined => {
                let span = self.text_span(subject);
                let reason = cstr!("`{}` is {}", subject.trim(), object.to_js_string());
                self.error(span, super::refusal::each(&reason));
                return;
            }
        };
        // `each` runs in its own function: `value` and `key` are `var`s.
        self.scope.push(true);
        if entries.is_empty() {
            self.check(&head.block);
        }
        for (index, item) in entries.iter() {
            let bound = self
                .scope
                .declare_var(value, item.clone())
                .and_then(|()| match key {
                    Some(key) => self.scope.declare_var(key, index.clone()),
                    None => Ok(()),
                });
            if let Err(reason) = bound {
                let span = self.span(&head.head);
                self.error(span, super::refusal::each(&reason));
                break;
            }
            self.scope.push(false);
            self.nodes(&head.block);
            self.scope.pop();
        }
        if let Some(alternate) = alternate {
            if entries.is_empty() {
                self.scope.push(false);
                self.nodes(&alternate.block);
                self.scope.pop();
            } else {
                self.check(&alternate.block);
            }
        }
        self.scope.pop();
    }

    fn each_of(&mut self, head: &'t PugRefused<'t>, names: &[&str], pair: bool, subject: &str) {
        let Some(object) = self.subject(subject) else {
            return;
        };
        let items: StdVec<Value> = match object {
            Value::Array(items) => items,
            Value::Str(text) => text
                .chars()
                .map(|ch| Value::str(ch.encode_utf8(&mut [0; 4])))
                .collect(),
            other => {
                let span = self.text_span(subject);
                let reason = cstr!("`{}` ({}) is not iterable", subject.trim(), other.type_of());
                self.error(span, super::refusal::each(&reason));
                return;
            }
        };
        if items.is_empty() {
            self.check(&head.block);
        }
        for item in items {
            let values = if pair {
                let parts = match item {
                    Value::Array(parts) => parts,
                    Value::Str(text) => text
                        .chars()
                        .map(|ch| Value::str(ch.encode_utf8(&mut [0; 4])))
                        .collect(),
                    other => {
                        let span = self.text_span(subject);
                        let reason = cstr!("an item ({}) is not iterable", other.type_of());
                        self.error(span, super::refusal::each(&reason));
                        return;
                    }
                };
                (0..2)
                    .map(|index| parts.get(index).cloned().unwrap_or(Value::Undefined))
                    .collect()
            } else {
                alloc::vec![item]
            };
            // `for (const value of …)`: a fresh block per iteration.
            self.scope.push(false);
            for (name, value) in names.iter().zip(values) {
                if let Err(reason) = self.scope.declare_const(name, value) {
                    let span = self.span(&head.head);
                    self.error(span, super::refusal::each(&reason));
                }
            }
            self.nodes(&head.block);
            self.scope.pop();
        }
    }

    /// The span of `text`, a slice of the authored source.
    pub(super) fn text_span(&self, text: &str) -> Span {
        if let Some(origin) = self.origin(text.as_ptr() as usize) {
            return origin;
        }
        let start = (text.as_ptr() as usize).saturating_sub(self.source_start()) as u32;
        Span::new(start, start + text.len() as u32)
    }
}

/// The expression inside a `#{…}` / `!{…}` token; `None` when unclosed.
pub(super) fn interpolated(token: &str) -> Option<&str> {
    token.get(2..)?.strip_suffix('}')
}
