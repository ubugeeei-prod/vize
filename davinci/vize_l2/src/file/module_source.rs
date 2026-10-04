//! Inline original source sites; no detached literal or additional table.

use core::num::NonZeroU32;

pub(crate) const DYNAMIC_IMPORT: u8 = 1;
pub(crate) const EMPTY_SOURCE_EXPORT: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub(crate) struct SourceSite {
    pub(crate) statement: NonZeroU32,
}

#[cfg(test)]
mod tests {
    use super::SourceSite;
    use crate::file::{Export, Import, Namespace, ScriptUnit, ScriptUnitId};
    use crate::resolution::BindingId;
    use vize_l0::{Span, String};

    #[expect(
        dead_code,
        reason = "old production field layout is an independent size control"
    )]
    struct OldImport {
        unit: ScriptUnitId,
        source: String,
        namespace: Namespace,
        span: Span,
    }
    #[expect(
        dead_code,
        reason = "old production field layout is an independent size control"
    )]
    struct OldExport {
        unit: ScriptUnitId,
        name: String,
        local: Option<BindingId>,
        source: Option<String>,
        namespace: Namespace,
        span: Span,
        local_reference: Option<usize>,
    }

    #[test]
    fn original_source_sites_have_explicit_inline_growth_and_no_unit_growth() {
        if cfg!(target_pointer_width = "64") {
            assert_eq!(core::mem::size_of::<OldImport>(), 40);
            assert_eq!(core::mem::size_of::<OldExport>(), 88);
            assert_eq!(core::mem::size_of::<SourceSite>(), 4);
            assert_eq!(core::mem::size_of::<Option<SourceSite>>(), 4);
            assert_eq!(core::mem::size_of::<Import>(), 48);
            assert_eq!(core::mem::size_of::<Export>(), 96);
            assert_eq!(core::mem::size_of::<ScriptUnit>(), 48);
            assert_eq!(core::mem::align_of::<ScriptUnit>(), 8);
        }
    }
}
