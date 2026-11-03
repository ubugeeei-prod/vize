//! The runtime helper vocabulary.
//!
//! A [`Helper`] is an index into one runtime's [`Vocabulary`], so the
//! used-helper set is a single 128-bit mask plus the first-use order, and a
//! target never compares helper names as strings. Each runtime (and runtime
//! version, when the helper set changes) has its own vocabulary table.
//!
//! Built-in tables name actual exports of pinned Vue releases, including
//! the separate server-renderer import. Helper indices are runtime-local,
//! not interchangeable across vocabularies. Target admission stays separate.

use alloc::vec::Vec;

/// A runtime whose helpers emitted code imports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Runtime {
    /// `vue` — the virtual-DOM runtime.
    VueDom,
    /// `@vue/server-renderer` and `vue` — the server rendering runtime.
    VueServerRenderer,
    /// `vue` — the Vapor runtime.
    VueVapor,
}

/// One helper of a runtime vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Helper(u8);

impl Helper {
    /// The largest vocabulary a [`HelperSet`] can track.
    pub const LIMIT: usize = 128;

    /// The helper at `index` of its vocabulary, if below [`Self::LIMIT`].
    #[must_use]
    pub const fn from_index(index: u8) -> Option<Self> {
        if (index as usize) < Self::LIMIT {
            Some(Self(index))
        } else {
            None
        }
    }

    #[must_use]
    pub const fn index(self) -> u8 {
        self.0
    }

    const fn bit(self) -> u128 {
        1u128 << self.0
    }
}

/// A module's exports in one runtime vocabulary.
#[derive(Debug)]
pub struct HelperModule {
    /// Import specifier, e.g. `"vue"`.
    pub module: &'static str,
    /// Exported names in stable index order.
    pub names: &'static [&'static str],
}

/// Modules and helper names for one target runtime and version.
///
/// Helpers index the modules' concatenated names. At most [`Helper::LIMIT`]
/// names are permitted; they must be distinct JavaScript identifiers. Module
/// order determines import-group order; each group retains body first-use order.
#[derive(Debug)]
pub struct Vocabulary {
    pub modules: &'static [HelperModule],
}

impl Vocabulary {
    /// The exported name of `helper`, if it belongs to this vocabulary.
    #[must_use]
    pub fn name(&self, helper: Helper) -> Option<&'static str> {
        self.export(helper).map(|(_, name)| name)
    }

    /// The import specifier and exported name of `helper`.
    #[must_use]
    pub fn export(&self, helper: Helper) -> Option<(&'static str, &'static str)> {
        let mut index = usize::from(helper.index());
        for module in self.modules {
            if let Some(&name) = module.names.get(index) {
                return Some((module.module, name));
            }
            index = index.checked_sub(module.names.len())?;
        }
        None
    }

    /// The helper exported as `name`. Targets may cache the checked index.
    #[must_use]
    pub fn helper(&self, name: &str) -> Option<Helper> {
        let mut base = 0;
        for module in self.modules {
            if let Some(index) = module.names.iter().position(|candidate| *candidate == name) {
                return u8::try_from(base + index).ok().and_then(Helper::from_index);
            }
            base += module.names.len();
        }
        None
    }
}

mod vue;
pub use vue::{VUE_DOM, VUE_SERVER_RENDERER, VUE_VAPOR};

/// Default helper vocabulary for the pinned target release.
///
/// DOM and SSR target Vue 3.5.35. Vapor targets Vue 3.6.0-rc.9, as pinned by
/// the existing runtime-conformance lane. This is a vocabulary provider, not
/// admission of every grammar/dialect or a native product route.
#[must_use]
pub fn vocabulary(runtime: Runtime) -> &'static Vocabulary {
    match runtime {
        Runtime::VueDom => &VUE_DOM,
        Runtime::VueServerRenderer => &VUE_SERVER_RENDERER,
        Runtime::VueVapor => &VUE_VAPOR,
    }
}

/// Select an audited exact release. Unsupported pairs have no fallback.
#[must_use]
pub fn vocabulary_for(runtime: Runtime, version: &str) -> Option<&'static Vocabulary> {
    match (runtime, version) {
        (Runtime::VueDom | Runtime::VueServerRenderer, "3.5.35")
        | (Runtime::VueVapor, "3.6.0-rc.9") => Some(vocabulary(runtime)),
        _ => None,
    }
}

/// The helpers a body used, in first-use order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HelperSet {
    mask: u128,
    order: Vec<Helper>,
}

impl HelperSet {
    /// Record a use of `helper`; returns whether it was new.
    pub fn insert(&mut self, helper: Helper) -> bool {
        if self.contains(helper) {
            return false;
        }
        self.mask |= helper.bit();
        self.order.push(helper);
        true
    }

    #[must_use]
    pub fn contains(&self, helper: Helper) -> bool {
        self.mask & helper.bit() != 0
    }

    /// Used helpers, in first-use order.
    #[must_use]
    pub fn in_use_order(&self) -> &[Helper] {
        &self.order
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// Add every helper of `other` not yet used, keeping first-use order.
    pub fn extend(&mut self, other: &Self) {
        for &helper in &other.order {
            self.insert(helper);
        }
    }
}

#[cfg(test)]
mod tests;
