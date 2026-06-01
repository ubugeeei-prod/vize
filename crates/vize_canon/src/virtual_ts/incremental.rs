//! Incremental virtual TypeScript regeneration tracker.
//!
//! Foundation for issue #698. Today every keystroke regenerates the whole
//! virtual TS document for the file. This module ships the block-level
//! signature used to detect "only template changed", "only script changed",
//! etc., so the regen path can reuse cached sections in a follow-up.
//!
//! The actual reuse logic depends on `generate_virtual_ts_with_offsets`
//! splitting its output by block, which is the next step. This commit lands
//! the change-detection model so both sides can be developed independently.

use vize_atelier_sfc::SfcDescriptor;
use vize_carton::hash::content_hash;

/// Block-level content signature for an SFC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualTsCacheKey {
    pub script_hash: Option<vize_carton::String>,
    pub script_setup_hash: Option<vize_carton::String>,
    pub template_hash: Option<vize_carton::String>,
}

impl VirtualTsCacheKey {
    /// Capture the per-block content hash for a parsed SFC descriptor.
    /// Two descriptors produce equal keys exactly when every block's content
    /// is byte-identical, which is the right granularity for the cache to
    /// reuse a generated virtual TS section.
    pub fn from_descriptor(descriptor: &SfcDescriptor<'_>) -> Self {
        Self {
            script_hash: descriptor
                .script
                .as_ref()
                .map(|block| content_hash(&block.content)),
            script_setup_hash: descriptor
                .script_setup
                .as_ref()
                .map(|block| content_hash(&block.content)),
            template_hash: descriptor.template_hash(),
        }
    }

    /// Returns true when only the template block differs between `self`
    /// and `previous`. The cache layer uses this to short-circuit script
    /// regeneration on template-only edits.
    pub fn only_template_changed(&self, previous: &Self) -> bool {
        self.script_hash == previous.script_hash
            && self.script_setup_hash == previous.script_setup_hash
            && self.template_hash != previous.template_hash
    }

    /// Returns true when only the script blocks differ.
    pub fn only_script_changed(&self, previous: &Self) -> bool {
        self.template_hash == previous.template_hash
            && (self.script_hash != previous.script_hash
                || self.script_setup_hash != previous.script_setup_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::VirtualTsCacheKey;
    use vize_atelier_sfc::{SfcParseOptions, parse_sfc};

    fn key_for(source: &str) -> VirtualTsCacheKey {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        VirtualTsCacheKey::from_descriptor(&descriptor)
    }

    #[test]
    fn detects_template_only_change() {
        let prev = key_for("<script setup>const x = 1</script><template><div>old</div></template>");
        let next = key_for("<script setup>const x = 1</script><template><div>new</div></template>");
        assert!(next.only_template_changed(&prev));
        assert!(!next.only_script_changed(&prev));
    }

    #[test]
    fn detects_script_only_change() {
        let prev =
            key_for("<script setup>const x = 1</script><template><div>same</div></template>");
        let next =
            key_for("<script setup>const x = 2</script><template><div>same</div></template>");
        assert!(!next.only_template_changed(&prev));
        assert!(next.only_script_changed(&prev));
    }

    #[test]
    fn identical_descriptors_yield_equal_keys() {
        let source = "<script setup>const x = 1</script><template><p /></template>";
        assert_eq!(key_for(source), key_for(source));
    }
}
