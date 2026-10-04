use super::{EndOfLine, FormatOptions};

impl FormatOptions {
    /// Only the Auto path calls this; explicit/default options never scan or clone.
    #[cold]
    pub(crate) fn with_resolved_line_ending(&self, source: &str) -> Self {
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
        options
    }
}
