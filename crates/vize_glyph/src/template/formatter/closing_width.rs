//! A closing grammar break must make the emitted source-owned line fit.
use super::TemplateFormatter;
use unicode_width::UnicodeWidthStr;

impl TemplateFormatter<'_> {
    #[inline]
    pub(super) fn closing_bracket_overflows(&self, output: &[u8]) -> bool {
        let start = memchr::memrchr2(b'\r', b'\n', output).map_or(0, |pos| pos + 1);
        let line = output.get(start..).unwrap_or_default();
        let suffix = 1 + self.base_depth * self.options.tab_width as usize;
        // UTF-8 bytes bound display width without tabs. Fitting ordinary ASCII
        // chunks avoid validation and exact Unicode width scans.
        if line.len() + suffix <= self.options.print_width as usize
            && memchr::memchr(b'\t', line).is_none()
        {
            return false;
        }
        let line = core::str::from_utf8(line).unwrap_or_default();
        let width = line.split('\t').map(str::width).sum::<usize>()
            + line.bytes().filter(|byte| *byte == b'\t').count() * self.options.tab_width as usize;
        // Moving only `>` cannot repair an already-unbreakable wide prefix.
        // Preserve its historical layout; break only a genuine final-column
        // overflow, when this grammar boundary makes the whole line fit.
        width + suffix == self.options.print_width as usize + 1
    }
}
