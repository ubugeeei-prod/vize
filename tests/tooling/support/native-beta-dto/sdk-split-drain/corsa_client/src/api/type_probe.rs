use corsa_core::fast::CompactString;

/// Options that control how much checker detail is loaded for a type probe.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TypeProbeOptions {
    /// When true, resolve the types of each property symbol in addition to the
    /// property names themselves.
    pub load_property_types: bool,
    /// When true, resolve call signatures and their return types.
    pub load_signatures: bool,
}

/// Aggregated checker view of a type at a single query site.
///
/// This is intentionally higher level than the raw Corsa endpoints. It keeps
/// the hot path for linters and editor integrations small by bundling the
/// repeated symbol, type, property, and signature lookups that usually travel
/// together.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TypeProbe {
    /// Rendered type texts for the primary type at the query site.
    pub type_texts: Vec<CompactString>,
    /// Properties exposed by the probed type.
    pub property_names: Vec<CompactString>,
    /// Rendered types for each property, aligned with [`Self::property_names`].
    pub property_types: Vec<Vec<CompactString>>,
    /// Rendered parameter types for each call signature.
    pub call_signatures: Vec<Vec<Vec<CompactString>>>,
    /// Rendered return types for each call signature.
    pub return_types: Vec<Vec<CompactString>>,
}
