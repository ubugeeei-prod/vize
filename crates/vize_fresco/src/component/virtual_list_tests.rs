use super::{VirtualListNavigation as Navigation, VirtualListState};

fn keys(count: usize) -> Vec<u64> {
    (0..count as u64).collect()
}

#[test]
fn ten_thousand_items_materialize_only_the_bounded_window() {
    let keys = keys(10_000);
    let mut list = VirtualListState::with_overscan(20, 3);

    assert!(list.reconcile(&keys));
    assert!(list.select_index(&keys, 5_000));

    let window = list.window();
    assert_eq!(list.selected_key(), Some(&5_000));
    assert_eq!(window.visible_range(), 4_981..5_001);
    assert_eq!(window.materialized_range(), 4_978..5_004);
    assert_eq!(window.visible_len(), 20);
    assert_eq!(window.materialized_len(), 26);
    assert_eq!(window.leading_items(), 4_978);
    assert_eq!(window.trailing_items(), 4_996);
}

#[test]
fn stable_key_survives_filtering_and_reordering() {
    let initial = vec!["a", "b", "c", "d", "e"];
    let filtered_and_reordered = vec!["e", "c", "a"];
    let mut list = VirtualListState::new(2);

    assert!(list.reconcile(&initial));
    assert!(list.select_index(&initial, 2));
    assert!(!list.reconcile(&filtered_and_reordered));

    assert_eq!(list.selected_key(), Some(&"c"));
    assert_eq!(list.selected_index(), Some(1));
    assert_eq!(list.window().visible_range(), 1..3);
}

#[test]
fn removed_selection_falls_back_to_the_nearest_ordinal() {
    let initial = vec![10, 20, 30, 40, 50];
    let shortened = vec![10, 20, 40];
    let mut list = VirtualListState::new(2);

    assert!(list.reconcile(&initial));
    assert!(list.select_index(&initial, 4));
    assert!(list.reconcile(&shortened));

    assert_eq!(list.selected_key(), Some(&40));
    assert_eq!(list.selected_index(), Some(2));
    assert_eq!(list.scroll_offset(), 1);
}

#[test]
fn empty_filter_clears_selection_and_repopulation_selects_first() {
    let mut list = VirtualListState::new(10);

    assert!(list.reconcile(&[1, 2, 3]));
    assert!(list.reconcile(&[]));
    assert_eq!(list.selected_key(), None);
    assert_eq!(list.window().materialized_range(), 0..0);

    assert!(list.reconcile(&[7, 8]));
    assert_eq!(list.selected_key(), Some(&7));
    assert_eq!(list.selected_index(), Some(0));
}

#[test]
fn navigation_is_bounded_and_reveals_selection() {
    let keys = keys(10);
    let mut list = VirtualListState::with_overscan(3, 0);
    assert!(list.reconcile(&keys));

    assert!(!list.navigate(&keys, Navigation::Previous));
    assert!(list.navigate(&keys, Navigation::Next));
    assert!(list.navigate(&keys, Navigation::PageDown));
    assert_eq!(list.selected_index(), Some(3));
    assert_eq!(list.window().visible_range(), 1..4);

    assert!(list.navigate(&keys, Navigation::Last));
    assert_eq!(list.window().visible_range(), 7..10);
    assert!(!list.navigate(&keys, Navigation::Next));

    assert!(list.navigate(&keys, Navigation::PageUp));
    assert_eq!(list.selected_index(), Some(7));
    assert!(list.navigate(&keys, Navigation::First));
    assert_eq!(list.window().visible_range(), 0..3);
}

#[test]
fn navigation_reconciles_a_changed_sequence_before_moving() {
    let initial = vec!["a", "b", "c", "d"];
    let reordered = vec!["d", "a", "c", "b"];
    let mut list = VirtualListState::new(2);

    assert!(list.reconcile(&initial));
    assert!(list.select_index(&initial, 1));
    assert!(list.navigate(&reordered, Navigation::Previous));

    assert_eq!(list.selected_key(), Some(&"c"));
    assert_eq!(list.selected_index(), Some(2));
}

#[test]
fn resize_preserves_selection_and_normalizes_scroll() {
    let keys = keys(100);
    let mut list = VirtualListState::new(10);
    assert!(list.reconcile(&keys));
    assert!(list.select_index(&keys, 50));
    assert_eq!(list.scroll_offset(), 41);

    assert!(list.set_viewport_len(5));
    assert_eq!(list.selected_key(), Some(&50));
    assert_eq!(list.scroll_offset(), 46);
    assert_eq!(list.window().visible_range(), 46..51);

    assert!(list.set_viewport_len(80));
    assert_eq!(list.scroll_offset(), 20);
    assert!(list.window().is_visible(50));
    assert!(!list.set_viewport_len(80));
}

#[test]
fn independent_scrolling_clamps_without_moving_selection() {
    let keys = keys(100);
    let mut list = VirtualListState::new(10);
    assert!(list.reconcile(&keys));

    assert!(list.scroll_by(isize::MAX));
    assert_eq!(list.scroll_offset(), 90);
    assert_eq!(list.selected_key(), Some(&0));
    assert!(!list.scroll_by(1));

    assert!(list.scroll_by(isize::MIN));
    assert_eq!(list.scroll_offset(), 0);
    assert!(!list.scroll_by(-1));
}

#[test]
fn zero_height_viewport_is_empty_but_keeps_stable_selection() {
    let keys = keys(10);
    let mut list = VirtualListState::new(0);

    assert!(list.reconcile(&keys));
    assert!(list.navigate(&keys, Navigation::Last));
    assert_eq!(list.selected_key(), Some(&9));
    assert_eq!(list.scroll_offset(), 0);
    assert_eq!(list.window().visible_range(), 0..0);
    assert_eq!(list.window().materialized_range(), 0..0);
}

#[test]
fn overscan_is_clamped_at_both_edges() {
    let keys = keys(5);
    let mut list = VirtualListState::with_overscan(2, 10);
    assert!(list.reconcile(&keys));

    let first = list.window();
    assert_eq!(first.materialized_range(), 0..5);
    assert!(first.is_materialized(4));

    assert!(list.navigate(&keys, Navigation::Last));
    let last = list.window();
    assert_eq!(last.visible_range(), 3..5);
    assert_eq!(last.materialized_range(), 0..5);
    assert!(!last.is_materialized(5));
}

#[test]
fn out_of_range_selection_is_ignored_after_reconciliation() {
    let mut list = VirtualListState::new(3);

    assert!(list.select_index(&[10, 20], 99));
    assert_eq!(list.selected_key(), Some(&10));
    assert!(!list.select_index(&[10, 20], 99));
}
