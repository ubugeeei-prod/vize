//! Optimization tracking for Vue template compilation.
//!
//! Tracks:
//! - Event handler caching
//! - v-once caching
//! - v-memo caching
//! - Block tree structure
//! - Patch flags

use vize_carton::{CompactString, SmallVec};

use crate::hoist::PatchFlags;

/// Cache entry ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CacheId(u32);

impl CacheId {
    #[inline(always)]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// Kind of cache
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CacheKind {
    Event = 0,
    Once = 1,
    Memo = 2,
}

/// Cached event handler
#[derive(Debug, Clone)]
pub struct CachedEvent {
    pub cache_index: u32,
    pub event_name: CompactString,
    pub handler: CompactString,
    pub is_inline: bool,
    pub is_component_event: bool,
    pub start: u32,
    pub end: u32,
}

/// Cached v-once content
#[derive(Debug, Clone)]
pub struct CachedOnce {
    pub cache_index: u32,
    pub content: CompactString,
    pub start: u32,
    pub end: u32,
}

/// Cached v-memo content
#[derive(Debug, Clone)]
pub struct CachedMemo {
    pub cache_index: u32,
    pub deps: CompactString,
    pub content: CompactString,
    pub start: u32,
    pub end: u32,
}

/// Block type in the block tree
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockType {
    Root = 0,
    If = 1,
    ElseIf = 2,
    Else = 3,
    For = 4,
    Component = 5,
    Suspense = 6,
    KeepAlive = 7,
    Teleport = 8,
    Fragment = 9,
}

/// Block in the block tree
#[derive(Debug, Clone)]
pub struct Block {
    pub id: u32,
    pub block_type: BlockType,
    pub parent_id: Option<u32>,
    pub dynamic_children_count: u32,
    pub start: u32,
    pub end: u32,
}

/// Patch info for a node
#[derive(Debug, Clone)]
pub struct NodePatchInfo {
    pub node_start: u32,
    pub node_end: u32,
    pub patch_flags: PatchFlags,
    pub dynamic_props: SmallVec<[CompactString; 4]>,
}

/// Statistics about optimizations
#[derive(Debug, Clone, Default)]
pub struct OptimizationStats {
    pub event_cache_count: u32,
    pub once_cache_count: u32,
    pub memo_cache_count: u32,
    pub block_count: u32,
    pub static_hoists: u32,
    pub total_dynamic_children: u32,
}

/// Tracks optimizations during template compilation
#[derive(Debug, Default)]
pub struct OptimizationTracker {
    event_cache: Vec<CachedEvent>,
    once_cache: Vec<CachedOnce>,
    memo_cache: Vec<CachedMemo>,
    blocks: Vec<Block>,
    node_patches: Vec<NodePatchInfo>,
    cache_index: u32,
    block_id: u32,
    block_stack: SmallVec<[u32; 8]>,
    total_dynamic_children: usize,
}

impl OptimizationTracker {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate next cache index
    #[inline]
    fn next_cache_index(&mut self) -> u32 {
        let idx = self.cache_index;
        self.cache_index += 1;
        idx
    }

    /// Allocate next block ID
    #[inline]
    fn next_block_id(&mut self) -> u32 {
        let id = self.block_id;
        self.block_id += 1;
        id
    }

    /// Cache an event handler
    pub fn cache_event(
        &mut self,
        event_name: CompactString,
        handler: CompactString,
        is_inline: bool,
        is_component_event: bool,
        start: u32,
        end: u32,
    ) -> u32 {
        let cache_index = self.next_cache_index();
        self.event_cache.push(CachedEvent {
            cache_index,
            event_name,
            handler,
            is_inline,
            is_component_event,
            start,
            end,
        });
        cache_index
    }

    /// Cache v-once content
    pub fn cache_once(&mut self, content: CompactString, start: u32, end: u32) -> u32 {
        let cache_index = self.next_cache_index();
        self.once_cache.push(CachedOnce {
            cache_index,
            content,
            start,
            end,
        });
        cache_index
    }

    /// Cache v-memo content
    pub fn cache_memo(
        &mut self,
        deps: CompactString,
        content: CompactString,
        start: u32,
        end: u32,
    ) -> u32 {
        let cache_index = self.next_cache_index();
        self.memo_cache.push(CachedMemo {
            cache_index,
            deps,
            content,
            start,
            end,
        });
        cache_index
    }

    /// Enter a block
    pub fn enter_block(&mut self, block_type: BlockType, start: u32, end: u32) -> u32 {
        let id = self.next_block_id();
        let parent_id = self.block_stack.last().copied();

        self.blocks.push(Block {
            id,
            block_type,
            parent_id,
            dynamic_children_count: 0,
            start,
            end,
        });

        self.block_stack.push(id);
        id
    }

    /// Exit current block
    pub fn exit_block(&mut self) {
        self.block_stack.pop();
    }

    /// Add dynamic child to current block
    pub fn add_dynamic_child(&mut self) {
        if let Some(&block_id) = self.block_stack.last() {
            if let Some(block) = self.blocks.iter_mut().find(|b| b.id == block_id) {
                block.dynamic_children_count += 1;
            }
        }
        self.total_dynamic_children += 1;
    }

    /// Record patch info for a node
    pub fn record_patch_info(
        &mut self,
        node_start: u32,
        node_end: u32,
        patch_flags: PatchFlags,
        dynamic_props: SmallVec<[CompactString; 4]>,
    ) {
        self.node_patches.push(NodePatchInfo {
            node_start,
            node_end,
            patch_flags,
            dynamic_props,
        });
    }

    // Getters

    #[inline]
    pub fn event_cache(&self) -> &[CachedEvent] {
        &self.event_cache
    }

    #[inline]
    pub fn once_cache(&self) -> &[CachedOnce] {
        &self.once_cache
    }

    #[inline]
    pub fn memo_cache(&self) -> &[CachedMemo] {
        &self.memo_cache
    }

    #[inline]
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    #[inline]
    pub fn node_patches(&self) -> &[NodePatchInfo] {
        &self.node_patches
    }

    #[inline]
    pub fn current_cache_index(&self) -> u32 {
        self.cache_index
    }

    /// Get optimization statistics
    pub fn stats(&self) -> OptimizationStats {
        OptimizationStats {
            event_cache_count: self.event_cache.len() as u32,
            once_cache_count: self.once_cache.len() as u32,
            memo_cache_count: self.memo_cache.len() as u32,
            block_count: self.blocks.len() as u32,
            static_hoists: 0, // Filled by HoistTracker
            total_dynamic_children: self.total_dynamic_children as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockType, OptimizationTracker};
    use vize_carton::CompactString;

    #[test]
    fn test_event_caching() {
        let mut tracker = OptimizationTracker::new();

        let idx = tracker.cache_event(
            CompactString::new("click"),
            CompactString::new("handleClick"),
            false,
            false,
            0,
            20,
        );

        assert_eq!(idx, 0);
        assert_eq!(tracker.event_cache().len(), 1);
        assert_eq!(tracker.current_cache_index(), 1);
    }

    #[test]
    fn test_block_tree() {
        let mut tracker = OptimizationTracker::new();

        let root = tracker.enter_block(BlockType::Root, 0, 100);
        let _if_block = tracker.enter_block(BlockType::If, 10, 50);

        tracker.add_dynamic_child();
        tracker.exit_block();
        tracker.exit_block();

        assert_eq!(tracker.blocks().len(), 2);
        assert_eq!(tracker.blocks()[1].parent_id, Some(root));
    }
}
