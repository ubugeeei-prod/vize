//! Runtime helper registration and lookup.

use crate::RuntimeHelper;
use vize_carton::FxHashMap;

/// Runtime helper set for tracking used helpers
#[derive(Debug, Default)]
pub struct RuntimeHelpers {
    helpers: FxHashMap<RuntimeHelper, u32>,
    order: std::vec::Vec<RuntimeHelper>,
}

impl RuntimeHelpers {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a helper usage
    pub fn add(&mut self, helper: RuntimeHelper) {
        if !self.helpers.contains_key(&helper) {
            self.order.push(helper);
        }
        *self.helpers.entry(helper).or_insert(0) += 1;
    }

    /// Remove a helper usage
    pub fn remove(&mut self, helper: RuntimeHelper) {
        if let Some(count) = self.helpers.get_mut(&helper) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.helpers.remove(&helper);
                self.order.retain(|ordered| *ordered != helper);
            }
        }
    }

    /// Check if a helper is used
    pub fn contains(&self, helper: RuntimeHelper) -> bool {
        self.helpers.contains_key(&helper)
    }

    /// Get all used helpers
    pub fn iter(&self) -> impl Iterator<Item = RuntimeHelper> + '_ {
        self.order
            .iter()
            .copied()
            .filter(|helper| self.helpers.contains_key(helper))
    }

    /// Get the count of a helper usage
    pub fn count(&self, helper: RuntimeHelper) -> u32 {
        self.helpers.get(&helper).copied().unwrap_or(0)
    }

    /// Clear all helpers
    pub fn clear(&mut self) {
        self.helpers.clear();
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
