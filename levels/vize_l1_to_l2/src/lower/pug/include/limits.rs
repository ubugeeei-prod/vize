//! Per-compilation bounds apply to total work, including cached expansions.
use vize_l0::Span;

pub(super) const MAX_INCLUDES: usize = 4096;
/// Maximum total source bytes read by one compilation's includes.
pub const MAX_INCLUDE_BYTES: usize = 16 * 1024 * 1024;
const MAX_OUTPUT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Default)]
pub(in crate::lower::pug) struct Budget {
    loads: usize,
    expansions: usize,
    source_bytes: usize,
    output_bytes: usize,
    pub(in crate::lower::pug) halted: bool,
    reported: bool,
}

impl Budget {
    pub(super) fn load(&mut self) -> bool {
        self.loads += 1;
        self.allow(self.loads <= MAX_INCLUDES)
    }

    pub(super) fn source(&mut self, bytes: usize) -> bool {
        let remaining = MAX_INCLUDE_BYTES.saturating_sub(self.source_bytes);
        if bytes > remaining {
            return self.allow(false);
        }
        self.source_bytes += bytes;
        self.allow(true)
    }

    pub(super) fn expand(&mut self) -> bool {
        self.expansions += 1;
        self.allow(self.expansions <= MAX_INCLUDES)
    }

    pub(super) fn output(&mut self, bytes: usize) -> bool {
        if self.loads == 0 {
            return true;
        }
        let remaining = MAX_OUTPUT_BYTES.saturating_sub(self.output_bytes);
        if bytes > remaining {
            return self.allow(false);
        }
        self.output_bytes += bytes;
        self.allow(true)
    }

    fn allow(&mut self, allowed: bool) -> bool {
        self.halted |= !allowed;
        !self.halted
    }

    pub(super) fn report(&mut self) -> bool {
        if self.reported {
            return false;
        }
        self.reported = true;
        true
    }
}

/// Reject rooted/drive/UNC forms on every host before a reader can interpret them.
pub(super) fn absolute(path: &str) -> bool {
    let bytes = path.as_bytes();
    path.starts_with(['/', '\\'])
        || (bytes.first().is_some_and(u8::is_ascii_alphabetic) && bytes.get(1) == Some(&b':'))
}

impl super::super::emit::Emitter<'_> {
    pub(in crate::lower::pug) fn include_budget_error(&mut self, span: Span) {
        if self.includes.budget.report() {
            self.error(
                span,
                vize_l0::String::from(super::super::refusal::INCLUDE_BUDGET),
            );
        }
    }

    pub(in crate::lower::pug) fn include_output(&mut self, bytes: usize, span: Span) -> bool {
        if self.includes.budget.output(bytes) {
            true
        } else {
            self.include_budget_error(span);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_and_output_bounds_are_independent_and_overflow_safe() {
        let mut budget = Budget::default();
        assert!(budget.load());
        assert!(budget.source(MAX_INCLUDE_BYTES));
        assert!(!budget.source(usize::MAX));
        let mut budget = Budget::default();
        assert!(budget.load());
        assert!(budget.output(MAX_OUTPUT_BYTES));
        assert!(!budget.output(usize::MAX));
    }

    #[test]
    fn templates_without_includes_keep_their_existing_output_policy() {
        assert!(Budget::default().output(usize::MAX));
    }
}
