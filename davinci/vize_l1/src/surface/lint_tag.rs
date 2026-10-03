//! Private facts produced only by selected Vue 3 complete-header construction.

#[derive(Clone, Copy)]
#[repr(u8)]
pub(crate) enum LintTagFact {
    Element = 1,
    Component = 2,
    Slot = 3,
    Template = 4,
    AmbiguousVerbatim = 5,
}

impl LintTagFact {
    pub(crate) fn from_aux(aux: u8) -> Option<Self> {
        match aux >> 1 {
            1 => Some(Self::Element),
            2 => Some(Self::Component),
            3 => Some(Self::Slot),
            4 => Some(Self::Template),
            5 => Some(Self::AmbiguousVerbatim),
            _ => None,
        }
    }
}
