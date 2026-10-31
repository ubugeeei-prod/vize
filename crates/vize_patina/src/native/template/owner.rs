//! Rule-bound ownership is separate from source/header observations.

use super::NativeTemplateElement;

/// Every actual enabled bare-template callback must offer the wider grammar.
/// Empty sets, existing callbacks and all selected SFC hosts remain strict.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NativeTemplateOwnerProfile {
    #[default]
    Strict,
    /// Explicitly closed select with direct, explicitly closed option children.
    /// No optgroup, label ancestry, dynamic owner or decoded value authority.
    ClosedSelect,
}

impl NativeTemplateOwnerProfile {
    pub(in crate::native) fn intersect(self, other: Self) -> Self {
        if self == Self::ClosedSelect && other == Self::ClosedSelect {
            Self::ClosedSelect
        } else {
            Self::Strict
        }
    }
}

/// Sealed proof borrowed only after the original select subtree was admitted.
/// It grants its checked opening range, never a whole-body diagnostic range.
///
/// ```compile_fail
/// use vize_patina::native::template::NativeTemplateClosedSelect;
/// let proof = NativeTemplateClosedSelect {};
/// ```
pub struct NativeTemplateClosedSelect<'s, 'o, 'a> {
    pub(super) element: &'s NativeTemplateElement<'o, 'a>,
}
