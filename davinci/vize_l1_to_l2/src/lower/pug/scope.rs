//! The bindings pug's build-time code makes, scoped as JavaScript
//! scopes them: `var` to the enclosing function (the template, an `each`),
//! `let` / `const` to the enclosing block.

use alloc::vec::Vec as StdVec;

use vize_l0::{String, cstr};

use super::eval::Refusal;
use super::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Var,
    Let,
    Const,
}

#[derive(Debug)]
struct Frame {
    /// A function body (`var` lands here) rather than a block.
    function: bool,
    names: StdVec<(String, Kind, Value)>,
}

/// The bindings visible to the code being evaluated.
#[derive(Debug)]
pub(super) struct Scope {
    frames: StdVec<Frame>,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            frames: alloc::vec![Frame {
                function: true,
                names: StdVec::new(),
            }],
        }
    }
}

impl Scope {
    /// Enter a block (`if`, a loop body) or a function (`each`).
    pub(super) fn push(&mut self, function: bool) {
        self.frames.push(Frame {
            function,
            names: StdVec::new(),
        });
    }

    pub(super) fn pop(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    pub(super) fn lookup(&self, name: &str) -> Option<&Value> {
        self.frames.iter().rev().find_map(|frame| {
            frame
                .names
                .iter()
                .find(|(bound, ..)| bound == name)
                .map(|(.., value)| value)
        })
    }

    /// Bind `name` as JavaScript would at its declaration.
    pub(super) fn declare_var(&mut self, name: &str, value: Value) -> Result<(), Refusal> {
        self.declare(name, Kind::Var, Some(value))
    }

    /// Bind `name` as a block-scoped constant (`for (const name of …)`).
    pub(super) fn declare_const(&mut self, name: &str, value: Value) -> Result<(), Refusal> {
        self.declare(name, Kind::Const, Some(value))
    }

    pub(super) fn declare(
        &mut self,
        name: &str,
        kind: Kind,
        value: Option<Value>,
    ) -> Result<(), Refusal> {
        if matches!(name, "undefined" | "NaN" | "Infinity") {
            return Err(cstr!("`{name}` cannot be rebound"));
        }
        let index = if kind == Kind::Var {
            self.frames.iter().rposition(|frame| frame.function)
        } else {
            self.frames.len().checked_sub(1)
        };
        let Some(frame) = index.and_then(|index| self.frames.get_mut(index)) else {
            return Err(String::from("there is no scope to bind in"));
        };
        match frame.names.iter_mut().find(|(bound, ..)| bound == name) {
            Some((_, Kind::Var, slot)) if kind == Kind::Var => {
                if let Some(value) = value {
                    *slot = value;
                }
            }
            Some(_) => {
                return Err(cstr!("`{name}` is declared twice in one scope"));
            }
            None => frame
                .names
                .push((String::from(name), kind, value.unwrap_or(Value::Undefined))),
        }
        Ok(())
    }
}
