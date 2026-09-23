//! Disjoint-set forest grouping files that share imports into one shard.

pub(super) struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    pub(super) fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
        }
    }

    pub(super) fn find(&mut self, node: usize) -> usize {
        let mut root = node;
        while let Some(&parent) = self.parent.get(root)
            && parent != root
        {
            root = parent;
        }
        let mut current = node;
        while let Some(parent) = self.parent.get_mut(current)
            && *parent != root
        {
            current = std::mem::replace(parent, root);
        }
        root
    }

    pub(super) fn union(&mut self, left: usize, right: usize) {
        let left_root = self.find(left);
        let right_root = self.find(right);
        if left_root != right_root
            && let Some(parent) = self.parent.get_mut(right_root)
        {
            *parent = left_root;
        }
    }
}
