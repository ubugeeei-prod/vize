//! The byte markers one script block's prefilters have searched for.

use std::cell::{OnceCell, RefCell};

use memchr::memmem;
use vize_l0::FxHashMap;

use super::js_source::{imports_from_vue, source_may_define_component_options};

/// One script block and the answers its prefilters already found.
///
/// Every enabled built-in script rule gates on a few substring searches over
/// the same block, and many share their needles: `defineProps`, `computed`,
/// `props`, `export default`. Each search used to build a `memmem` searcher
/// and scan the whole block again, so a block paid one full scan per needle
/// per rule. A block now scans for a needle at most once.
pub(in crate::linter::script_rules) struct BlockMarkers<'s> {
    source: &'s str,
    found: RefCell<FxHashMap<&'static [u8], bool>>,
    component_options: OnceCell<bool>,
    vue_import: OnceCell<bool>,
}

impl<'s> BlockMarkers<'s> {
    pub(in crate::linter::script_rules) fn new(source: &'s str) -> Self {
        Self {
            source,
            found: RefCell::default(),
            component_options: OnceCell::new(),
            vue_import: OnceCell::new(),
        }
    }

    pub(super) fn source(&self) -> &'s str {
        self.source
    }

    /// Whether the block contains `needle`.
    pub(super) fn contains(&self, needle: &'static [u8]) -> bool {
        if let Some(found) = self.found.borrow().get(needle) {
            return *found;
        }
        let found = memmem::find(self.source.as_bytes(), needle).is_some();
        self.found.borrow_mut().insert(needle, found);
        found
    }

    /// [`source_may_define_component_options`] for this block.
    pub(super) fn may_define_component_options(&self) -> bool {
        *self
            .component_options
            .get_or_init(|| source_may_define_component_options(self.source.as_bytes()))
    }

    /// [`imports_from_vue`] for this block.
    pub(super) fn imports_from_vue(&self) -> bool {
        *self
            .vue_import
            .get_or_init(|| imports_from_vue(self.source.as_bytes()))
    }
}
