//! The bindings pug's build-time code makes, scoped as JavaScript
//! scopes them: `var` to the enclosing function (the template, an `each`,
//! a mixin, a mixin call's `block`), `let` / `const` to the enclosing
//! block.
//!
//! Frames live in an arena and point at their parent, so a mixin (and a
//! call's `block`) closes over the frames where it was written: it reads
//! their bindings as they are when it runs, even after the block that
//! made them has ended. A frame no closure captured is dropped when it
//! ends.

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

/// A frame in the arena.
pub(super) type FrameId = usize;

#[derive(Debug)]
struct Frame {
    /// A function body (`var` lands here) rather than a block.
    function: bool,
    parent: Option<FrameId>,
    /// A closure reads this frame; it outlives its block.
    captured: bool,
    names: StdVec<(String, Kind, Value)>,
}

/// The bindings visible to the code being evaluated.
#[derive(Debug)]
pub(super) struct Scope {
    frames: StdVec<Frame>,
    current: FrameId,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            frames: alloc::vec![Frame {
                function: true,
                parent: None,
                captured: true,
                names: StdVec::new(),
            }],
            current: 0,
        }
    }
}

impl Scope {
    /// Enter a block (`if`, a loop body) or a function (`each`).
    pub(super) fn push(&mut self, function: bool) {
        self.open(function, self.current);
    }

    /// Leave the innermost block or function.
    pub(super) fn pop(&mut self) {
        let parent = self.frame(self.current).and_then(|frame| frame.parent);
        self.close(parent.unwrap_or(0));
    }

    /// The current frame, kept alive for a closure that will run later.
    pub(super) fn capture(&mut self) -> FrameId {
        let mut at = Some(self.current);
        while let Some(frame) = at.and_then(|id| self.frames.get_mut(id)) {
            if frame.captured {
                break;
            }
            frame.captured = true;
            at = frame.parent;
        }
        self.current
    }

    /// Call a closure over `frame`: a fresh function frame under it.
    /// Returns the frame to [`Self::leave`] back to.
    pub(super) fn enter(&mut self, frame: FrameId) -> FrameId {
        let caller = self.current;
        self.open(true, frame);
        caller
    }

    /// Return from [`Self::enter`].
    pub(super) fn leave(&mut self, caller: FrameId) {
        self.close(caller);
    }

    fn open(&mut self, function: bool, parent: FrameId) {
        self.frames.push(Frame {
            function,
            parent: Some(parent),
            captured: false,
            names: StdVec::new(),
        });
        self.current = self.frames.len() - 1;
    }

    fn close(&mut self, to: FrameId) {
        let done = self.current;
        let droppable =
            done + 1 == self.frames.len() && self.frame(done).is_some_and(|frame| !frame.captured);
        if droppable && done != 0 {
            self.frames.pop();
        }
        self.current = to.min(self.frames.len().saturating_sub(1));
    }

    fn frame(&self, id: FrameId) -> Option<&Frame> {
        self.frames.get(id)
    }

    /// The frames from the current one outwards.
    fn chain(&self) -> impl Iterator<Item = (FrameId, &Frame)> {
        let mut at = Some(self.current);
        core::iter::from_fn(move || {
            let id = at?;
            let frame = self.frame(id)?;
            at = frame.parent;
            Some((id, frame))
        })
    }

    pub(super) fn lookup(&self, name: &str) -> Option<&Value> {
        self.chain().find_map(|(_, frame)| {
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
            self.chain()
                .find(|(_, frame)| frame.function)
                .map(|(id, _)| id)
        } else {
            Some(self.current)
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

#[cfg(test)]
mod tests {
    use super::Scope;
    use crate::lower::pug::value::Value;

    #[test]
    fn closures_read_their_frames_after_the_block_ends() {
        let mut scope = Scope::default();
        scope.push(false);
        scope.declare_const("x", Value::Number(1.0)).unwrap();
        let captured = scope.capture();
        scope.pop();
        assert_eq!(scope.lookup("x"), None);
        let caller = scope.enter(captured);
        assert_eq!(scope.lookup("x"), Some(&Value::Number(1.0)));
        scope.declare_var("y", Value::Null).unwrap();
        scope.leave(caller);
        assert_eq!(scope.lookup("y"), None);
        // An uncaptured frame is dropped when it ends.
        scope.push(false);
        scope.pop();
        assert_eq!(scope.frames.len(), 2);
    }
}
