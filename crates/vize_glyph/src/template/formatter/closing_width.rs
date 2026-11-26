//! A closing grammar break must make the emitted source-owned line fit.
use super::TemplateFormatter;
use unicode_width::UnicodeWidthStr;

impl TemplateFormatter<'_> {
    #[inline]
    pub(super) fn closing_bracket_overflows(&self, output: &str) -> bool {
        let line = super::current_line(output);
        let suffix = 1 + self.base_depth * self.options.tab_width as usize;
        // UTF-8 bytes bound display width without tabs. Fitting ordinary ASCII
        // chunks avoid validation and exact Unicode width scans.
        if line.len() + suffix <= self.options.print_width as usize
            && memchr::memchr(b'\t', line.as_bytes()).is_none()
        {
            return false;
        }
        let width = line.split('\t').map(str::width).sum::<usize>()
            + line.bytes().filter(|byte| *byte == b'\t').count() * self.options.tab_width as usize;
        // Moving only `>` cannot repair an already-unbreakable wide prefix.
        // Preserve its historical layout; break only a genuine final-column
        // overflow, when this grammar boundary makes the whole line fit.
        width + (suffix - 1) == self.options.print_width as usize
    }
}
