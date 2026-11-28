//! Stack headroom for recursive descent.
//!
//! Nesting depth in the source is call depth in the parser, and running out of
//! stack aborts the process instead of returning a diagnostic. Callers that own
//! their thread (the Vize CLI and LSP) reserve large stacks, but bindings such
//! as N-API run on the host's thread and get whatever stack it has.
//!
//! Every recursion cycle of the grammar passes through
//! [`ParserImpl::with_stack_headroom`]. In the common case that is one compare
//! of the current frame address against a window computed when the parser was
//! created. Only when the frame is outside that window — the stack is within
//! [`RED_ZONE`] of its end, or another stack segment is running — does the cold
//! path ask `stacker` for the remaining stack and continue the descent on a
//! fresh [`SEGMENT_SIZE`] heap segment when needed. Nesting then costs heap
//! instead of stack; no grammar, diagnostic, or depth refusal is involved.

use crate::{ParserConfig as Config, ParserImpl};

/// Stack that must remain before a step proceeds in place. One grammar cycle
/// uses a few KiB of frames between two checks; this leaves ample slack.
const RED_ZONE: usize = 512 * 1024;

/// Size of the heap segment a descent continues on once the red zone is reached.
const SEGMENT_SIZE: usize = 4 * 1024 * 1024;

/// Frame addresses above the one seen at creation that still count as inside
/// the window, for parser frames that sit slightly higher than that point.
const ABOVE_SLACK: usize = 64 * 1024;

/// Frame addresses `[floor, floor + span)` at which a step may proceed in place.
#[derive(Clone, Copy)]
pub(crate) struct StackWindow {
    floor: usize,
    span: usize,
}

impl StackWindow {
    /// The window of the stack running now.
    pub(crate) fn current() -> Self {
        let here = frame_address();
        match stacker::remaining_stack() {
            Some(remaining) => {
                let floor = here.saturating_sub(remaining).saturating_add(RED_ZONE);
                Self { floor, span: here.saturating_add(ABOVE_SLACK).saturating_sub(floor) }
            }
            // The stack end is unknown (e.g. wasm32): descend in place, as
            // `stacker` itself does there.
            None => Self { floor: 0, span: usize::MAX },
        }
    }

    #[inline(always)]
    fn contains(self, address: usize) -> bool {
        address.wrapping_sub(self.floor) < self.span
    }
}

/// An address inside the caller's frame.
#[inline(always)]
fn frame_address() -> usize {
    let marker = 0u8;
    std::ptr::addr_of!(marker).addr()
}

impl<'a, C: Config> ParserImpl<'a, C> {
    /// Run one recursive step of the descent with at least [`RED_ZONE`] of
    /// stack left.
    #[inline(always)]
    pub(crate) fn with_stack_headroom<R>(&mut self, step: impl FnOnce(&mut Self) -> R) -> R {
        if self.stack.contains(frame_address()) {
            return step(self);
        }
        self.with_stack_headroom_cold(step)
    }

    #[cold]
    #[inline(never)]
    fn with_stack_headroom_cold<R>(&mut self, step: impl FnOnce(&mut Self) -> R) -> R {
        let outer = self.stack;
        let result = if stacker::remaining_stack().is_some_and(|remaining| remaining < RED_ZONE) {
            stacker::grow(SEGMENT_SIZE, || {
                self.stack = StackWindow::current();
                step(self)
            })
        } else {
            self.stack = StackWindow::current();
            step(self)
        };
        self.stack = outer;
        result
    }
}

#[cfg(test)]
mod tests;
