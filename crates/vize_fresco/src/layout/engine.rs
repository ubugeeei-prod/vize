//! Layout engine using taffy.

use rustc_hash::FxHashMap;
use taffy::prelude::{AvailableSpace, Dimension, NodeId, Size, TaffyTree};

use super::{flex::FlexStyle, rect::Rect};

/// Layout engine powered by taffy.
pub struct LayoutEngine {
    /// The taffy tree
    tree: TaffyTree<()>,
    /// Node ID mapping (our IDs to taffy NodeIds)
    node_map: FxHashMap<u64, NodeId>,
    /// Reverse mapping (taffy NodeIds to our IDs)
    reverse_map: FxHashMap<NodeId, u64>,
    /// Layout results cache
    layout_cache: FxHashMap<u64, Rect>,
    /// Next available node ID
    next_id: u64,
    /// Root node ID
    root: Option<u64>,
}

impl LayoutEngine {
    /// Create a new layout engine.
    pub fn new() -> Self {
        Self {
            tree: TaffyTree::new(),
            node_map: FxHashMap::default(),
            reverse_map: FxHashMap::default(),
            layout_cache: FxHashMap::default(),
            next_id: 0,
            root: None,
        }
    }

    /// Create a new node with the given style.
    pub fn new_node(&mut self, style: &FlexStyle) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let taffy_style = style.to_taffy();
        let node_id = self
            .tree
            .new_leaf(taffy_style)
            // Panic path by backend invariant: Fresco only constructs leaf nodes
            // with a locally generated style, so Taffy should reject this only if
            // its internal storage is inconsistent. Returning a synthetic id here
            // would corrupt `node_map` and make later layout reads unsound.
            .expect("Failed to create node");

        self.node_map.insert(id, node_id);
        self.reverse_map.insert(node_id, id);

        id
    }

    /// Create a new leaf node with measured size.
    pub fn new_leaf(&mut self, style: &FlexStyle, width: f32, height: f32) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let mut taffy_style = style.to_taffy();
        taffy_style.size = Size {
            width: Dimension::length(width),
            height: Dimension::length(height),
        };

        let node_id = self
            .tree
            .new_leaf(taffy_style)
            // Panic path by backend invariant: measured leaves are created from a
            // normalized `FlexStyle` plus finite dimensions supplied by Fresco's
            // renderer. Failing here means the layout tree is unusable.
            .expect("Failed to create leaf");

        self.node_map.insert(id, node_id);
        self.reverse_map.insert(node_id, id);

        id
    }

    /// Set the root node.
    pub fn set_root(&mut self, id: u64) {
        self.root = Some(id);
    }

    /// Get the root node ID.
    pub fn root(&self) -> Option<u64> {
        self.root
    }

    /// Add a child to a parent node.
    pub fn add_child(&mut self, parent: u64, child: u64) {
        if let (Some(&parent_id), Some(&child_id)) =
            (self.node_map.get(&parent), self.node_map.get(&child))
        {
            self.tree
                .add_child(parent_id, child_id)
                // Panic path by mapping invariant: both ids came from
                // `node_map`, so Taffy should know each node. If it rejects the
                // edge, the mirrored maps are already inconsistent and continuing
                // would cache wrong layouts.
                .expect("Failed to add child");
        }
    }

    /// Remove a child from a parent node.
    pub fn remove_child(&mut self, parent: u64, child: u64) {
        if let (Some(&parent_id), Some(&child_id)) =
            (self.node_map.get(&parent), self.node_map.get(&child))
        {
            self.tree
                .remove_child(parent_id, child_id)
                // Panic path by mapping invariant: callers can request unknown
                // ids, but those are filtered above. Once both ids are mapped,
                // failure means our mirrored Taffy tree has diverged.
                .expect("Failed to remove child");
        }
    }

    /// Update the style of a node.
    pub fn set_style(&mut self, id: u64, style: &FlexStyle) {
        if let Some(&node_id) = self.node_map.get(&id) {
            let taffy_style = style.to_taffy();
            self.tree
                .set_style(node_id, taffy_style)
                // Panic path by mapping invariant: `node_id` is only read from
                // `node_map`, so Taffy should still own it. A failure indicates
                // internal tree corruption rather than user input.
                .expect("Failed to set style");
        }
    }

    /// Remove a node from the tree.
    pub fn remove(&mut self, id: u64) {
        if let Some(node_id) = self.node_map.remove(&id) {
            self.reverse_map.remove(&node_id);
            self.layout_cache.remove(&id);
            // Panic path by mapping invariant: after a successful lookup in
            // `node_map`, Taffy must contain the node. Losing it would mean the
            // mirrored maps and layout tree diverged earlier.
            self.tree.remove(node_id).expect("Failed to remove node");
        }
    }

    /// Compute layout for the entire tree.
    pub fn compute(&mut self, available_width: f32, available_height: f32) {
        if let Some(root_id) = self.root.and_then(|id| self.node_map.get(&id).copied()) {
            let available = Size {
                width: AvailableSpace::Definite(available_width),
                height: AvailableSpace::Definite(available_height),
            };

            self.tree
                .compute_layout(root_id, available)
                // Panic path by mapping invariant: `root_id` is taken from
                // `node_map`, and all children are added through the same map.
                // If Taffy cannot compute this tree, Fresco should fail loudly
                // instead of rendering stale or partial geometry.
                .expect("Failed to compute layout");

            // Cache all layouts
            self.cache_layouts(root_id, 0.0, 0.0, true);
        }
    }

    /// Cache layout results recursively from Taffy's computed geometry.
    fn cache_layouts(&mut self, node_id: NodeId, parent_x: f32, parent_y: f32, is_root: bool) {
        // Panic path by compute invariant: `cache_layouts` is called only after
        // `compute_layout` succeeds for `root_id`, and recursive calls use
        // children returned by Taffy itself. Missing layout/style data would mean
        // Taffy accepted an internally inconsistent tree.
        let layout = self.tree.layout(node_id).expect("Failed to get layout");
        let (absolute_x, absolute_y) = if is_root {
            (parent_x, parent_y)
        } else {
            (parent_x + layout.location.x, parent_y + layout.location.y)
        };

        // Store this node's layout
        if let Some(&id) = self.reverse_map.get(&node_id) {
            self.layout_cache.insert(
                id,
                Rect::new(
                    absolute_x.round() as u16,
                    absolute_y.round() as u16,
                    layout.size.width.round() as u16,
                    layout.size.height.round() as u16,
                ),
            );
        }

        // Collect children to release the `&self` borrow before recursing.
        let children: Vec<_> = self.tree.children(node_id).unwrap_or_default();

        for child_id in children {
            self.cache_layouts(child_id, absolute_x, absolute_y, false);
        }
    }

    /// Get the computed layout for a node.
    pub fn layout(&self, id: u64) -> Option<Rect> {
        self.layout_cache.get(&id).copied()
    }

    /// Get all computed layouts.
    pub fn layouts(&self) -> &FxHashMap<u64, Rect> {
        &self.layout_cache
    }

    /// Clear all nodes.
    pub fn clear(&mut self) {
        self.tree = TaffyTree::new();
        self.node_map.clear();
        self.reverse_map.clear();
        self.layout_cache.clear();
        self.next_id = 0;
        self.root = None;
    }

    /// Get the number of nodes.
    pub fn node_count(&self) -> usize {
        self.node_map.len()
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{FlexStyle, LayoutEngine};

    #[test]
    fn test_engine_new() {
        let engine = LayoutEngine::new();
        assert_eq!(engine.node_count(), 0);
    }

    #[test]
    fn test_engine_create_node() {
        let mut engine = LayoutEngine::new();
        let style = FlexStyle::new();
        let id = engine.new_node(&style);
        assert_eq!(engine.node_count(), 1);
        assert_eq!(id, 0);
    }

    #[test]
    fn test_engine_add_child() {
        let mut engine = LayoutEngine::new();
        let style = FlexStyle::new();

        let parent = engine.new_node(&style);
        let child = engine.new_node(&style);

        engine.add_child(parent, child);
        assert_eq!(engine.node_count(), 2);
    }

    #[test]
    fn test_engine_compute_layout() {
        use super::super::flex::{Dimension, FlexDirection};

        let mut engine = LayoutEngine::new();

        // Create root with column direction
        let mut root_style = FlexStyle::new();
        root_style.flex_direction = FlexDirection::Column;
        root_style.width = Dimension::Points(100.0);
        root_style.height = Dimension::Points(100.0);
        let root = engine.new_node(&root_style);

        // Create child
        let mut child_style = FlexStyle::new();
        child_style.height = Dimension::Points(50.0);
        let child = engine.new_leaf(&child_style, 100.0, 50.0);

        engine.add_child(root, child);
        engine.set_root(root);
        engine.compute(100.0, 100.0);

        let root_layout = engine.layout(root).unwrap();
        assert_eq!(root_layout.width, 100);
        assert_eq!(root_layout.height, 100);

        let child_layout = engine.layout(child).unwrap();
        assert_eq!(child_layout.width, 100);
        assert_eq!(child_layout.height, 50);
    }

    #[test]
    fn uses_taffy_justify_and_align_positions() {
        use super::super::flex::{AlignItems, Dimension, JustifyContent};

        let mut engine = LayoutEngine::new();

        let mut root_style = FlexStyle::new();
        root_style.width = Dimension::Points(100.0);
        root_style.height = Dimension::Points(40.0);
        root_style.justify_content = JustifyContent::Center;
        root_style.align_items = AlignItems::Center;
        let root = engine.new_node(&root_style);

        let child_style = FlexStyle::new();
        let child = engine.new_leaf(&child_style, 20.0, 10.0);

        engine.add_child(root, child);
        engine.set_root(root);
        engine.compute(100.0, 40.0);

        let child_layout = engine.layout(child).unwrap();
        assert_eq!(child_layout.x, 40);
        assert_eq!(child_layout.y, 15);
        assert_eq!(child_layout.width, 20);
        assert_eq!(child_layout.height, 10);
    }

    #[test]
    fn uses_taffy_gap_positions() {
        use super::super::flex::{Dimension, Gap};

        let mut engine = LayoutEngine::new();

        let mut root_style = FlexStyle::new();
        root_style.width = Dimension::Points(100.0);
        root_style.height = Dimension::Points(20.0);
        root_style.gap = Gap::all(3.0);
        let root = engine.new_node(&root_style);

        let child_style = FlexStyle::new();
        let first = engine.new_leaf(&child_style, 10.0, 4.0);
        let second = engine.new_leaf(&child_style, 5.0, 4.0);

        engine.add_child(root, first);
        engine.add_child(root, second);
        engine.set_root(root);
        engine.compute(100.0, 20.0);

        let first_layout = engine.layout(first).unwrap();
        let second_layout = engine.layout(second).unwrap();
        assert_eq!(first_layout.x, 0);
        assert_eq!(second_layout.x, 13);
    }
}
