//! The compiler's CSS call grammar in the owning Vue file's byte coordinates.
use crate::container::vue::StyleView;
use vize_l0::{SourceBlock, Span};

/// A completed call in an original style. Its argument is a slice of the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyleBinding<'a> {
    pub call: Span,
    pub argument: Span,
    pub expression: &'a str,
}

impl<'a> StyleView<'_, 'a> {
    /// Stream in original order, with the compiler's partial-prefix recovery.
    pub fn binding_calls(self) -> impl Iterator<Item = StyleBinding<'a>> {
        binding_calls_in(self.block())
    }
}

/// Stream the compiler's CSS grammar inside an authenticated original file block.
pub fn binding_calls_in(block: SourceBlock<'_>) -> impl Iterator<Item = StyleBinding<'_>> {
    let start = block.start();
    let source = block.source();
    // CSS and every supported preprocessor spell the compiler marker verbatim.
    // Most original styles contain no calls, so skip their lexical walk.
    let css = if memchr::memchr(b'v', source.as_bytes()).is_some() {
        source
    } else {
        ""
    };
    super::v_bind_calls(css).map(move |hit| {
        let argument = hit.expression_range();
        StyleBinding {
            call: Span::new(start + hit.call_start as u32, start + hit.call_end as u32),
            argument: Span::new(start + argument.start as u32, start + argument.end as u32),
            expression: hit.expression,
        }
    })
}
