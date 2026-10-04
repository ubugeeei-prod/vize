//! Semantic dependency demands, ordered by the original decision events.

/// Target-neutral roles mapped to the selected runtime vocabulary by L4.
///
/// These are not runtime spellings, helper table ids, or patch flag bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomDependency {
    DisplayValue,
    BlockBoundary,
    NativeElementBlock,
    NativeElementValue,
    TextValue,
    CommentValue,
    FragmentValue,
    ClassNormalization,
    StyleNormalization,
    ConditionalPlaceholder,
    CollectionIteration,
}
