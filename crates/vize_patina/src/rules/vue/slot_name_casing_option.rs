/// Casing accepted for a rendered slot outlet name.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SlotNameCasingStyle {
    /// Lower camel case, such as `rowActions`.
    #[default]
    CamelCase,
    /// Lowercase hyphenated words, such as `row-actions`.
    KebabCase,
    /// One lowercase ASCII word, such as `actions`.
    Singleword,
}

impl SlotNameCasingStyle {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::CamelCase => "camelCase",
            Self::KebabCase => "kebab-case",
            Self::Singleword => "singleword",
        }
    }
}
