//! Runtime helper registration and lookup.

use crate::RuntimeHelper;

/// One use count per `u8` discriminant, so every helper indexes in bounds.
const SLOTS: usize = 1 << u8::BITS;

/// Runtime helper set for tracking used helpers
#[derive(Debug)]
pub struct RuntimeHelpers {
    /// Use counts indexed by the helper's discriminant: a registration is a
    /// load and a store, with no hashing and no out-of-line map probe.
    counts: [u32; SLOTS],
    order: std::vec::Vec<RuntimeHelper>,
}

impl Default for RuntimeHelpers {
    fn default() -> Self {
        Self {
            counts: [0; SLOTS],
            order: std::vec::Vec::new(),
        }
    }
}

#[inline(always)]
fn slot(helper: RuntimeHelper) -> usize {
    usize::from(helper as u8)
}

impl RuntimeHelpers {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a helper usage
    #[inline]
    pub fn add(&mut self, helper: RuntimeHelper) {
        if let Some(count) = self.counts.get_mut(slot(helper)) {
            if *count == 0 {
                self.order.push(helper);
            }
            *count += 1;
        }
    }

    /// Remove a helper usage
    pub fn remove(&mut self, helper: RuntimeHelper) {
        if let Some(count) = self.counts.get_mut(slot(helper))
            && *count != 0
        {
            *count -= 1;
            if *count == 0 {
                self.order.retain(|ordered| *ordered != helper);
            }
        }
    }

    /// Check if a helper is used
    #[inline]
    pub fn contains(&self, helper: RuntimeHelper) -> bool {
        self.count(helper) != 0
    }

    /// Get all used helpers
    pub fn iter(&self) -> impl Iterator<Item = RuntimeHelper> + '_ {
        // `remove` evicts the last use from both collections, so `order`
        // already contains exactly the active helpers.
        self.order.iter().copied()
    }

    /// Get the count of a helper usage
    #[inline]
    pub fn count(&self, helper: RuntimeHelper) -> u32 {
        self.counts.get(slot(helper)).copied().unwrap_or(0)
    }

    /// Clear all helpers
    pub fn clear(&mut self) {
        self.counts = [0; SLOTS];
        self.order.clear();
    }
}

/// Get the helper for creating VNodes
pub fn get_vnode_helper(ssr: bool, is_component: bool) -> RuntimeHelper {
    if ssr || is_component {
        RuntimeHelper::CreateVNode
    } else {
        RuntimeHelper::CreateElementVNode
    }
}

/// Get the helper for creating block VNodes
pub fn get_vnode_block_helper(ssr: bool, is_component: bool) -> RuntimeHelper {
    if ssr || is_component {
        RuntimeHelper::CreateBlock
    } else {
        RuntimeHelper::CreateElementBlock
    }
}

#[cfg(test)]
mod tests {
    use super::{RuntimeHelper, RuntimeHelpers};

    #[test]
    fn test_helpers() {
        let mut helpers = RuntimeHelpers::new();

        helpers.add(RuntimeHelper::CreateVNode);
        assert!(helpers.contains(RuntimeHelper::CreateVNode));
        assert_eq!(helpers.count(RuntimeHelper::CreateVNode), 1);

        helpers.add(RuntimeHelper::CreateVNode);
        assert_eq!(helpers.count(RuntimeHelper::CreateVNode), 2);

        helpers.remove(RuntimeHelper::CreateVNode);
        assert_eq!(helpers.count(RuntimeHelper::CreateVNode), 1);

        helpers.remove(RuntimeHelper::CreateVNode);
        assert!(!helpers.contains(RuntimeHelper::CreateVNode));
    }

    #[test]
    fn test_helpers_preserve_insertion_order() {
        let mut helpers = RuntimeHelpers::new();

        helpers.add(RuntimeHelper::ToDisplayString);
        helpers.add(RuntimeHelper::CreateElementVNode);
        helpers.add(RuntimeHelper::OpenBlock);
        helpers.add(RuntimeHelper::CreateElementVNode);

        assert_eq!(
            helpers.iter().collect::<Vec<_>>(),
            [
                RuntimeHelper::ToDisplayString,
                RuntimeHelper::CreateElementVNode,
                RuntimeHelper::OpenBlock,
            ]
        );

        helpers.remove(RuntimeHelper::CreateElementVNode);
        assert_eq!(
            helpers.iter().collect::<Vec<_>>(),
            [
                RuntimeHelper::ToDisplayString,
                RuntimeHelper::CreateElementVNode,
                RuntimeHelper::OpenBlock,
            ]
        );

        helpers.remove(RuntimeHelper::CreateElementVNode);
        assert_eq!(
            helpers.iter().collect::<Vec<_>>(),
            [RuntimeHelper::ToDisplayString, RuntimeHelper::OpenBlock]
        );

        helpers.add(RuntimeHelper::CreateElementVNode);
        assert_eq!(
            helpers.iter().collect::<Vec<_>>(),
            [
                RuntimeHelper::ToDisplayString,
                RuntimeHelper::OpenBlock,
                RuntimeHelper::CreateElementVNode,
            ]
        );
    }
}
