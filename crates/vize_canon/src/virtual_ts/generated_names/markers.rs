//! Line prefixes that mark generated template-use rows.

/// The line prefix of a template use projected onto its setup binding.
pub(crate) const TEMPLATE_USE_PREFIX: &str = concat!(
    "/* ",
    vize_l0::generated_name!(TEMPLATE_USE_MARKER),
    " */ typeof "
);

/// The line prefix of a template use's unwrapped, const-preserving hover proxy.
pub(crate) const TEMPLATE_HOVER_PREFIX: &str = concat!(
    "/* ",
    vize_l0::generated_name!(TEMPLATE_HOVER_MARKER),
    " */ const "
);
