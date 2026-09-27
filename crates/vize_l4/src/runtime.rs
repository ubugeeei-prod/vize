//! The runtime helper vocabulary.
//!
//! A [`Helper`] is an index into one runtime's [`Vocabulary`], so the
//! used-helper set is a single 128-bit mask plus the first-use order, and a
//! target never compares helper names as strings. Each runtime (and runtime
//! version, when the helper set changes) has its own vocabulary table.
//!
//! The tables port from `vize_l1_to_l2::emit::helper` and the legacy
//! `RuntimeHelper` names (#6840); until then [`vocabulary`] is unfinished.

#![expect(clippy::todo, reason = "skeleton: #6840")]

use alloc::vec::Vec;

/// A runtime whose helpers emitted code imports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Runtime {
    /// `vue` — the virtual-DOM runtime.
    VueDom,
    /// `vue/server-renderer` — the server rendering runtime.
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

/// The helper names a runtime exports, and the module they come from.
#[derive(Debug)]
pub struct Vocabulary {
    /// Import specifier, e.g. `"vue"`.
    pub module: &'static str,
    /// Exported names, indexed by [`Helper::index`]. At most
    /// [`Helper::LIMIT`] entries.
    pub names: &'static [&'static str],
}

impl Vocabulary {
    /// The exported name of `helper`, if it belongs to this vocabulary.
    #[must_use]
    pub fn name(&self, helper: Helper) -> Option<&'static str> {
        self.names.get(usize::from(helper.index())).copied()
    }

    /// The helper exported as `name`.
    #[must_use]
    pub fn helper(&self, name: &str) -> Option<Helper> {
        let index = self.names.iter().position(|candidate| *candidate == name)?;
        u8::try_from(index).ok().and_then(Helper::from_index)
    }
}

/// The helper vocabulary of `runtime`.
#[must_use]
pub fn vocabulary(_runtime: Runtime) -> &'static Vocabulary {
    todo!("#6840: port the DOM, server-renderer and Vapor helper tables")
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
