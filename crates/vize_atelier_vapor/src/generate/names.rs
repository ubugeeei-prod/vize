//! Compact generated-name membership with a bounded hash fallback.

use vize_carton::{FxHashSet, SmallVec, String};

/// Most generated name scopes contain only a few names. Keep those inline;
/// promote once to a hash set so large templates retain expected O(1) lookup.
#[derive(Default)]
pub(super) struct NameSet {
    small: SmallVec<[String; 4]>,
    hashed: Option<FxHashSet<String>>,
}

impl NameSet {
    pub(super) fn insert(&mut self, name: String) -> bool {
        if let Some(hashed) = &mut self.hashed {
            return hashed.insert(name);
        }
        if self
            .small
            .iter()
            .any(|existing| existing.as_str() == name.as_str())
        {
            return false;
        }
        if self.small.len() < 4 {
            self.small.push(name);
            return true;
        }

        let mut hashed = FxHashSet::with_capacity_and_hasher(32, Default::default());
        hashed.extend(self.small.drain(..));
        let inserted = hashed.insert(name);
        self.hashed = Some(hashed);
        inserted
    }

    pub(super) fn contains(&self, name: &str) -> bool {
        match &self.hashed {
            Some(hashed) => hashed.contains(name),
            None => self.small.iter().any(|existing| existing.as_str() == name),
        }
    }

    pub(super) fn remove(&mut self, name: &str) -> bool {
        if let Some(hashed) = &mut self.hashed {
            return hashed.remove(name);
        }
        if let Some(index) = self
            .small
            .iter()
            .position(|existing| existing.as_str() == name)
        {
            self.small.remove(index);
            true
        } else {
            false
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.hashed
            .as_ref()
            .map_or_else(|| self.small.is_empty(), FxHashSet::is_empty)
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &String> {
        self.small
            .iter()
            .chain(self.hashed.iter().flat_map(|set| set.iter()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vize_carton::cstr;

    #[test]
    fn matches_hash_set_membership_across_promotion_and_removal() {
        let mut names = NameSet::default();
        let mut expected = FxHashSet::default();
        for index in 0..40 {
            let name = cstr!("event-{index}");
            assert_eq!(names.insert(name.clone()), expected.insert(name.clone()));
            let actual_member = names.contains(&name);
            let expected_member = expected.contains(name.as_str());
            assert_eq!(actual_member, expected_member);
        }
        for name in ["event-0", "event-4", "event-39", "event-4", "missing"] {
            assert_eq!(names.remove(name), expected.remove(name), "{name}");
        }
        let mut actual: Vec<_> = names.iter().map(String::as_str).collect();
        let mut reference: Vec<_> = expected.iter().map(String::as_str).collect();
        actual.sort_unstable();
        reference.sort_unstable();
        assert_eq!(actual, reference);
        assert_eq!(names.is_empty(), expected.is_empty());
    }

    #[test]
    fn preserves_small_scope_duplicates_and_case() {
        let mut names = NameSet::default();
        assert!(names.insert(String::from("$event")));
        assert!(!names.insert(String::from("$event")));
        assert!(names.insert(String::from("$Event")));
        let membership = ["$event", "$Event", "event"].map(|name| names.contains(name));
        assert_eq!(membership, [true, true, false]);
        assert!(names.remove("$event"));
        let removed_member = names.contains("$event");
        assert!(!removed_member);
    }
}
