use vize_carton::String as CompactString;

use crate::batch::import_rewriter::ModuleSpecifierSpan;

pub(super) enum DependencySpecifierIter<'a> {
    Parsed(std::slice::Iter<'a, (CompactString, crate::PackageResolutionMode)>),
    Reused {
        source: &'a str,
        spans: std::slice::Iter<'a, ModuleSpecifierSpan>,
    },
}

impl<'a> Iterator for DependencySpecifierIter<'a> {
    type Item = (&'a str, crate::PackageResolutionMode);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Parsed(occurrences) => occurrences
                .next()
                .map(|(specifier, mode)| (specifier.as_str(), *mode)),
            Self::Reused { source, spans } => loop {
                let occurrence = spans.next()?;
                if let Some(specifier) = occurrence.specifier(source) {
                    return Some((specifier, occurrence.mode()));
                }
            },
        }
    }
}
