use super::{EndOfLine, FormatOptions};

impl FormatOptions {
    /// Keep source scanning and owned options outside every explicit/default frame.
    #[cold]
    #[inline(never)]
    pub(crate) fn format_with_source_line_ending<T>(
        &self,
        source: &str,
        format: impl FnOnce(&Self) -> T,
    ) -> T {
        let bytes = source.as_bytes();
        let mut options = self.clone();
        options.end_of_line = match memchr::memchr2(b'\r', b'\n', bytes) {
            Some(index) if bytes.get(index) == Some(&b'\r') => {
                if bytes.get(index + 1) == Some(&b'\n') {
                    EndOfLine::Crlf
                } else {
                    EndOfLine::Cr
                }
            }
            _ => EndOfLine::Lf,
        };
        format(&options)
    }
}
