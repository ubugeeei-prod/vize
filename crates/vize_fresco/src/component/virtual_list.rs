//! Bounded state for virtualized, stable-keyed lists.

use std::ops::Range;

/// The default number of off-screen items retained on each side of a viewport.
const DEFAULT_OVERSCAN: usize = 2;

/// A semantic selection command for a [`VirtualListState`].
///
/// Page commands overlap the old and new viewport by one item. That keeps
/// spatial context while guaranteeing progress, including in a one-row or
/// zero-row viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualListNavigation {
    /// Select the preceding item, stopping at the first item.
    Previous,
    /// Select the following item, stopping at the last item.
    Next,
    /// Move toward the start by one viewport page.
    PageUp,
    /// Move toward the end by one viewport page.
    PageDown,
    /// Select the first item.
    First,
    /// Select the last item.
    Last,
}

/// The bounded ranges needed to render one virtualized list frame.
///
/// `visible_range` contains only on-screen items. `materialized_range` adds
/// overscan before and after that range so a renderer can prepare a small
/// number of nearby rows without constructing the complete list. Both ranges
/// are half-open and always bounded by the list's item count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualWindow {
    visible_range: Range<usize>,
    materialized_range: Range<usize>,
    item_count: usize,
}

impl VirtualWindow {
    /// Return the half-open range of items occupying the viewport.
    pub fn visible_range(&self) -> Range<usize> {
        self.visible_range.clone()
    }

    /// Return the half-open range the renderer should materialize.
    pub fn materialized_range(&self) -> Range<usize> {
        self.materialized_range.clone()
    }

    /// Return the number of on-screen items.
    pub fn visible_len(&self) -> usize {
        self.visible_range.len()
    }

    /// Return the maximum number of rows the renderer needs to construct.
    pub fn materialized_len(&self) -> usize {
        self.materialized_range.len()
    }

    /// Return the number of non-materialized items before the window.
    ///
    /// Fixed-height renderers can multiply this by their row height to obtain
    /// the leading virtual spacer size.
    pub fn leading_items(&self) -> usize {
        self.materialized_range.start
    }

    /// Return the number of non-materialized items after the window.
    pub fn trailing_items(&self) -> usize {
        self.item_count.saturating_sub(self.materialized_range.end)
    }

    /// Return whether `index` is currently visible.
    pub fn is_visible(&self, index: usize) -> bool {
        self.visible_range.contains(&index)
    }

    /// Return whether `index` is inside the overscanned materialized window.
    pub fn is_materialized(&self, index: usize) -> bool {
        self.materialized_range.contains(&index)
    }
}

/// Selection and viewport state for a large, stable-keyed list.
///
/// The state retains one selected key and a constant amount of numeric state;
/// it never owns rows or copies the complete key sequence. Call [`reconcile`](Self::reconcile)
/// after filtering or reordering. Navigation also detects the common forms of
/// stale state and reconciles automatically. With an unchanged key sequence,
/// navigation is O(1); reconciliation is O(n) and allocation-free apart from
/// cloning a newly selected key.
///
/// Keys must be unique within the supplied slice. When a selected key is
/// removed, selection moves to the item at the same ordinal, or to the new last
/// item when the old ordinal is past the end. Empty lists have no selection.
#[derive(Debug, Clone)]
pub struct VirtualListState<K> {
    selected_key: Option<K>,
    selected_index: Option<usize>,
    item_count: usize,
    scroll_offset: usize,
    viewport_len: usize,
    overscan: usize,
}

impl<K: Clone + Eq> VirtualListState<K> {
    /// Create empty list state for a viewport containing `viewport_len` rows.
    ///
    /// Two off-screen rows are materialized on each side by default. Use
    /// [`with_overscan`](Self::with_overscan) when the renderer has a different
    /// latency or memory budget.
    pub fn new(viewport_len: usize) -> Self {
        Self::with_overscan(viewport_len, DEFAULT_OVERSCAN)
    }

    /// Create empty list state with an explicit per-side overscan count.
    pub fn with_overscan(viewport_len: usize, overscan: usize) -> Self {
        Self {
            selected_key: None,
            selected_index: None,
            item_count: 0,
            scroll_offset: 0,
            viewport_len,
            overscan,
        }
    }

    /// Return the selected stable key, if the current list is non-empty.
    pub fn selected_key(&self) -> Option<&K> {
        self.selected_key.as_ref()
    }

    /// Return the selected ordinal in the current key sequence.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Return the number of items observed by the latest reconciliation.
    pub fn item_count(&self) -> usize {
        self.item_count
    }

    /// Return the first visible item ordinal.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Return the number of rows available to visible items.
    pub fn viewport_len(&self) -> usize {
        self.viewport_len
    }

    /// Return the number of off-screen items materialized on each side.
    pub fn overscan(&self) -> usize {
        self.overscan
    }

    /// Reconcile selection against a filtered or reordered stable-key slice.
    ///
    /// Returns `true` when the selected key changes. Scroll position may still
    /// be normalized when this returns `false`.
    #[must_use]
    pub fn reconcile(&mut self, keys: &[K]) -> bool {
        let previous_index = self.selected_index.unwrap_or(0);
        self.item_count = keys.len();

        if keys.is_empty() {
            let changed = self.selected_key.take().is_some();
            self.selected_index = None;
            self.scroll_offset = 0;
            return changed;
        }

        let next_index = self
            .selected_key
            .as_ref()
            .and_then(|selected| keys.iter().position(|key| key == selected))
            .unwrap_or_else(|| previous_index.min(keys.len() - 1));
        let changed = self.selected_key.as_ref() != Some(&keys[next_index]);
        if changed {
            self.selected_key = Some(keys[next_index].clone());
        }
        self.selected_index = Some(next_index);
        self.reveal_selection();
        changed
    }

    /// Select `index`, returning whether selection or reconciliation changed.
    ///
    /// Out-of-range indexes are ignored after synchronizing the current keys.
    #[must_use]
    pub fn select_index(&mut self, keys: &[K], index: usize) -> bool {
        let reconciled = self.synchronize(keys);
        if index >= keys.len() {
            return reconciled;
        }
        let selected = self.set_selection(keys, index);
        reconciled || selected
    }

    /// Apply a semantic navigation command.
    ///
    /// Returns whether selection or reconciliation changed. Selection never
    /// wraps, and any newly selected item is scrolled into view.
    #[must_use]
    pub fn navigate(&mut self, keys: &[K], navigation: VirtualListNavigation) -> bool {
        let reconciled = self.synchronize(keys);
        let Some(current) = self.selected_index else {
            return reconciled;
        };
        let last = keys.len() - 1;
        let page = self.viewport_len.saturating_sub(1).max(1);
        let target = match navigation {
            VirtualListNavigation::Previous => current.saturating_sub(1),
            VirtualListNavigation::Next => current.saturating_add(1).min(last),
            VirtualListNavigation::PageUp => current.saturating_sub(page),
            VirtualListNavigation::PageDown => current.saturating_add(page).min(last),
            VirtualListNavigation::First => 0,
            VirtualListNavigation::Last => last,
        };
        let selected = self.set_selection(keys, target);
        reconciled || selected
    }

    /// Change viewport capacity while preserving the selected stable key.
    ///
    /// Shrinking the viewport adjusts the scroll offset just enough to keep the
    /// selection visible. A zero-row viewport renders no items and resets its
    /// offset without discarding selection.
    pub fn set_viewport_len(&mut self, viewport_len: usize) -> bool {
        if self.viewport_len == viewport_len {
            return false;
        }
        self.viewport_len = viewport_len;
        self.reveal_selection();
        true
    }

    /// Change per-side overscan without changing selection or scroll position.
    pub fn set_overscan(&mut self, overscan: usize) -> bool {
        if self.overscan == overscan {
            return false;
        }
        self.overscan = overscan;
        true
    }

    /// Scroll independently of selection by a signed number of rows.
    ///
    /// This is suitable for wheel or touchpad input. Keyboard selection
    /// navigation uses [`navigate`](Self::navigate), which reveals selection.
    pub fn scroll_by(&mut self, rows: isize) -> bool {
        let next = if rows.is_negative() {
            self.scroll_offset.saturating_sub(rows.unsigned_abs())
        } else {
            self.scroll_offset.saturating_add(rows as usize)
        }
        .min(self.max_scroll_offset());
        let changed = next != self.scroll_offset;
        self.scroll_offset = next;
        changed
    }

    /// Compute the visible and overscanned ranges for the current frame.
    ///
    /// This operation is O(1), regardless of the list's item count.
    pub fn window(&self) -> VirtualWindow {
        if self.item_count == 0 || self.viewport_len == 0 {
            return VirtualWindow {
                visible_range: 0..0,
                materialized_range: 0..0,
                item_count: self.item_count,
            };
        }

        let visible_start = self.scroll_offset.min(self.item_count);
        let visible_end = visible_start
            .saturating_add(self.viewport_len)
            .min(self.item_count);
        VirtualWindow {
            visible_range: visible_start..visible_end,
            materialized_range: visible_start.saturating_sub(self.overscan)
                ..visible_end
                    .saturating_add(self.overscan)
                    .min(self.item_count),
            item_count: self.item_count,
        }
    }

    fn synchronize(&mut self, keys: &[K]) -> bool {
        let synchronized = self.item_count == keys.len()
            && match (self.selected_index, self.selected_key.as_ref()) {
                (Some(index), Some(selected)) => keys.get(index) == Some(selected),
                (None, None) => keys.is_empty(),
                _ => false,
            };
        if synchronized {
            false
        } else {
            self.reconcile(keys)
        }
    }

    fn set_selection(&mut self, keys: &[K], index: usize) -> bool {
        let changed =
            self.selected_index != Some(index) || self.selected_key.as_ref() != keys.get(index);
        if changed {
            self.selected_key = Some(keys[index].clone());
            self.selected_index = Some(index);
        }
        self.reveal_selection();
        changed
    }

    fn reveal_selection(&mut self) {
        if self.viewport_len == 0 {
            self.scroll_offset = 0;
            return;
        }
        let Some(selected) = self.selected_index else {
            self.scroll_offset = 0;
            return;
        };
        if selected < self.scroll_offset {
            self.scroll_offset = selected;
        } else if selected >= self.scroll_offset.saturating_add(self.viewport_len) {
            self.scroll_offset = selected + 1 - self.viewport_len;
        }
        self.scroll_offset = self.scroll_offset.min(self.max_scroll_offset());
    }

    fn max_scroll_offset(&self) -> usize {
        if self.viewport_len == 0 {
            0
        } else {
            self.item_count.saturating_sub(self.viewport_len)
        }
    }
}
