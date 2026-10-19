//! Preserve CompactString's full inline capacity; share only long owned keys.

use std::{
    borrow::Borrow,
    hash::{Hash, Hasher},
    ops::Deref,
};

use ecow::EcoString;
use vize_carton::CompactString;

#[derive(Debug, Clone)]
pub(super) enum ModuleKey {
    Inline(CompactString),
    Shared(EcoString),
}

impl Default for ModuleKey {
    fn default() -> Self {
        Self::Inline(CompactString::default())
    }
}

impl ModuleKey {
    pub(super) fn as_str(&self) -> &str {
        match self {
            Self::Inline(text) => text.as_str(),
            Self::Shared(text) => text.as_str(),
        }
    }

    pub(super) fn push_str(&mut self, suffix: &str) {
        match self {
            Self::Inline(text) => {
                let capacity = text.len().saturating_add(suffix.len());
                if capacity <= text.capacity() {
                    text.push_str(suffix);
                } else {
                    let mut shared = EcoString::with_capacity(capacity);
                    shared.push_str(text);
                    shared.push_str(suffix);
                    *self = Self::Shared(shared);
                }
            }
            Self::Shared(text) => text.push_str(suffix),
        }
    }
}

/// The same lexical normalization builds either stored or temporary keys.
pub(super) trait KeyBuffer: Default {
    fn push_str(&mut self, suffix: &str);
}

impl KeyBuffer for ModuleKey {
    fn push_str(&mut self, suffix: &str) {
        Self::push_str(self, suffix);
    }
}

impl KeyBuffer for CompactString {
    fn push_str(&mut self, suffix: &str) {
        Self::push_str(self, suffix);
    }
}

impl Deref for ModuleKey {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Borrow<str> for ModuleKey {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Hash for ModuleKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl PartialEq for ModuleKey {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for ModuleKey {}

#[cfg(test)]
mod tests {
    use super::ModuleKey;
    use vize_carton::{CompactString, FxHashMap};

    #[test]
    fn compact_inline_capacity_is_preserved_before_sharing_long_keys() {
        let capacity = CompactString::default().capacity();
        for length in 0..=capacity {
            let mut key = ModuleKey::default();
            let text = "x".repeat(length);
            key.push_str(&text);
            assert_eq!(key.as_str(), text);
            let ModuleKey::Inline(inline) = &key else {
                panic!("a CompactString-inline key must stay inline");
            };
            assert!(!inline.is_heap_allocated());
            let cloned = key.clone();
            assert!(matches!(cloned, ModuleKey::Inline(_)));
        }
        let mut key = ModuleKey::default();
        key.push_str(&"x".repeat(capacity));
        key.push_str("🙂");
        assert!(matches!(key, ModuleKey::Shared(_)));
        let cloned = key.clone();
        assert_eq!(key.as_ptr(), cloned.as_ptr());
        assert_eq!(key.len(), capacity + '🙂'.len_utf8());
    }

    #[test]
    fn borrowed_hash_lookup_survives_transition_and_cow_mutation() {
        let mut key = ModuleKey::default();
        key.push_str("src/components/application/共有状態/Module");
        let original = key.clone();
        let mut map = FxHashMap::default();
        map.insert(key.clone(), 7);
        assert_eq!(map.get(key.as_str()), Some(&7));
        key.push_str(".tsx");
        assert_eq!(
            original.as_str(),
            "src/components/application/共有状態/Module"
        );
        assert!(!map.contains_key(key.as_str()));
        assert_eq!(map.get(original.as_str()), Some(&7));
        assert_ne!(key.as_ptr(), original.as_ptr());
    }
}
