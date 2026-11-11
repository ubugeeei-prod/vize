use vize_carton::String;

#[derive(Debug, PartialEq, Eq)]
pub(in crate::batch) struct ModuleSpecifierSpan {
    start: u32,
    end: u32,
    mode: crate::PackageResolutionMode,
    /// The parser-decoded spelling only when it differs from authored bytes.
    /// Common literals stay span-only and retain no duplicate path string.
    decoded: Option<String>,
}

impl ModuleSpecifierSpan {
    pub(super) fn new(
        source: &str,
        start: u32,
        end: u32,
        decoded: String,
        mode: crate::PackageResolutionMode,
    ) -> Self {
        let authored = source.get(start as usize..end as usize);
        let decoded = (authored != Some(decoded.as_str())).then_some(decoded);
        Self {
            start,
            end,
            mode,
            decoded,
        }
    }

    pub(in crate::batch) fn specifier<'a>(&'a self, source: &'a str) -> Option<&'a str> {
        self.decoded
            .as_deref()
            .or_else(|| source.get(self.start as usize..self.end as usize))
    }

    pub(in crate::batch) fn mode(&self) -> crate::PackageResolutionMode {
        self.mode
    }
}

pub(in crate::batch) type ModuleSpecifierSpans = Vec<ModuleSpecifierSpan>;
