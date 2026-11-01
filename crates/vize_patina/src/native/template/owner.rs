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
    /// The same closed select grammar plus explicitly closed textarea owners
    /// whose body is opaque plain literal text (no markup/entities/markers).
    ClosedSelectAndTextarea,
}

impl NativeTemplateOwnerProfile {
    pub(in crate::native) fn intersect(self, other: Self) -> Self {
        match (self, other) {
            (Self::ClosedSelectAndTextarea, profile) | (profile, Self::ClosedSelectAndTextarea) => {
                profile
            }
            (Self::ClosedSelect, Self::ClosedSelect) => Self::ClosedSelect,
            _ => Self::Strict,
        }
    }

    pub(in crate::native) fn allows_select(self) -> bool {
        matches!(self, Self::ClosedSelect | Self::ClosedSelectAndTextarea)
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

/// Sealed original closed textarea with a completely checked plain-text body.
/// Its diagnostic authority remains only the original opening range.
///
/// ```compile_fail
/// use vize_patina::native::template::NativeTemplateClosedTextarea;
/// let proof = NativeTemplateClosedTextarea {};
/// ```
pub struct NativeTemplateClosedTextarea<'s, 'o, 'a> {
    pub(super) element: &'s NativeTemplateElement<'o, 'a>,
}
