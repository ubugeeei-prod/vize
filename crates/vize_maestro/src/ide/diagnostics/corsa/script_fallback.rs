//! Last-resort positioning for script diagnostics the source map cannot place.
//!
//! When the byte-range mapping finds nothing for a script-side diagnostic, the
//! collector guesses a position by line arithmetic: virtual line minus the
//! start of user code, replayed from the authored `<script>` start. That
//! assumes the virtual script lines up with the authored one one-for-one,
//! which generated code (`__VizeThis` members, hoisted helpers) breaks — the
//! guess then points past the authored EOF and the editor renders a diagnostic
//! on a line that does not exist (#3299).
//!
//! `vize check` drops unmappable diagnostics outright, so the guess is
//! accepted only while it lands inside the authored document. A wrong range is
//! worse than a missing duplicate: the underlying defect is still reported at
//! its authored range by the diagnostic that did map.

/// Inputs for the guessed script-diagnostic range, all in zero-based lines.
#[derive(Clone, Copy)]
pub(super) struct ScriptFallback {
    /// First virtual line belonging to user code.
    pub(super) user_code_start_line: u32,
    /// Authored line the `<script>` block's content starts on (one-based).
    pub(super) sfc_script_start_line: u32,
    /// Import lines the virtual generator dropped ahead of user code.
    pub(super) skipped_import_lines: u32,
    /// Lines in the authored document; a guess outside it is rejected.
    pub(super) authored_line_count: u32,
}

impl ScriptFallback {
    /// Guess the authored range for a virtual diagnostic, or `None` when the
    /// guess falls outside the authored document (or ahead of user code).
    pub(super) fn guess_range(
        &self,
        virtual_start_line: u32,
        virtual_end_line: u32,
    ) -> Option<(u32, u32)> {
        if virtual_start_line < self.user_code_start_line {
            return None;
        }

        let script_base = self.sfc_script_start_line.saturating_sub(1);
        let start = script_base
            + virtual_start_line.saturating_sub(self.user_code_start_line)
            + self.skipped_import_lines;
        let end = script_base
            + virtual_end_line.saturating_sub(self.user_code_start_line)
            + self.skipped_import_lines;

        if start >= self.authored_line_count || end >= self.authored_line_count {
            return None;
        }

        Some((start, end))
    }
}

#[cfg(test)]
mod tests {
    use super::ScriptFallback;

    const FALLBACK: ScriptFallback = ScriptFallback {
        user_code_start_line: 10,
        sfc_script_start_line: 3,
        skipped_import_lines: 1,
        authored_line_count: 20,
    };

    #[test]
    fn guesses_a_range_inside_the_document() {
        // virtual line 12 -> script base 2 + offset 2 + 1 skipped import = 5
        assert_eq!(FALLBACK.guess_range(12, 12), Some((5, 5)));
        assert_eq!(FALLBACK.guess_range(12, 14), Some((5, 7)));
    }

    #[test]
    fn rejects_positions_past_the_authored_eof() {
        // The #3299 shape: generated `__VizeThis` members push the virtual line
        // far beyond anything the authored file can render.
        assert_eq!(FALLBACK.guess_range(80, 80), None);
        // A start inside the document with an end past it is rejected too: an
        // editor would draw the squiggle to a line that does not exist.
        assert_eq!(FALLBACK.guess_range(12, 80), None);
    }

    #[test]
    fn keeps_the_last_authored_line_addressable() {
        let last_line = FALLBACK.authored_line_count - 1;
        let virtual_line = FALLBACK.user_code_start_line + last_line
            - (FALLBACK.sfc_script_start_line - 1)
            - FALLBACK.skipped_import_lines;
        assert_eq!(
            FALLBACK.guess_range(virtual_line, virtual_line),
            Some((last_line, last_line))
        );
        assert_eq!(
            FALLBACK.guess_range(virtual_line + 1, virtual_line + 1),
            None
        );
    }

    #[test]
    fn rejects_preamble_diagnostics_ahead_of_user_code() {
        assert_eq!(FALLBACK.guess_range(9, 9), None);
        assert_eq!(FALLBACK.guess_range(0, 0), None);
    }

    #[test]
    fn rejects_every_guess_in_an_empty_document() {
        let empty = ScriptFallback {
            authored_line_count: 0,
            ..FALLBACK
        };
        assert_eq!(empty.guess_range(12, 12), None);
    }
}
