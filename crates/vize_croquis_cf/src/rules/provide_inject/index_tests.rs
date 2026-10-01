use super::ProvideInjectIndex;
use crate::registry::FileId;
use std::cmp::Ordering;
use vize_carton::{FxHashMap, FxHashSet};

#[test]
fn shared_file_order_places_known_paths_before_missing_entries() {
    let first = FileId::new(7);
    let second = FileId::new(3);
    let missing_low = FileId::new(1);
    let missing_high = FileId::new(9);
    let index = ProvideInjectIndex {
        provides: FxHashMap::default(),
        injects: FxHashMap::default(),
        reactive_provides: FxHashSet::default(),
        component_parents: FxHashMap::default(),
        stable_file_order: FxHashMap::from_iter([(first, 0), (second, 1)]),
    };

    assert_eq!(index.compare_files(first, second), Ordering::Less);
    assert_eq!(index.compare_files(second, missing_low), Ordering::Less);
    assert_eq!(
        index.compare_files(missing_low, missing_high),
        Ordering::Less
    );
}
