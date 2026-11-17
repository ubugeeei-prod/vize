//! Named child properties of checker syntax nodes. The encoder writes a
//! node's children right after it, and the low byte of its data marks which
//! of its child properties are present, in visitor order (typescript-go
//! `internal/api/encoder`). Declarations a symbol names in another file are
//! read here instead of parsing their text.
use super::{NODE, Tree, invalid, word};
use corsa::api::NodeHandle;
use vize_l0::cstr;

/// typescript-go 7.0 `ast.Kind` values. Every one sits below `SOURCE_FILE`,
/// so a runtime whose root is another kind numbers them differently and its
/// trees are refused.
pub(in crate::linter) mod kind {
    pub const TYPE_PARAMETER: u32 = 169;
    pub const PARAMETER: u32 = 170;
    pub const PROPERTY_SIGNATURE: u32 = 172;
    pub const PROPERTY_DECLARATION: u32 = 173;
    pub const GET_ACCESSOR: u32 = 178;
    pub const MAPPED_TYPE: u32 = 201;
    pub const VARIABLE_DECLARATION: u32 = 261;
    pub const SOURCE_FILE: u32 = 307;
    /// A node list: the parent of its elements.
    pub const LIST: u32 = u32::MAX;
}

/// Data whose top two bits are zero lists the node's present children.
const DATA_TYPE: u32 = 0xc000_0000;

pub(in crate::linter) struct Syntax<'a> {
    tree: Tree<'a>,
    count: usize,
    path: &'a str,
}

impl<'a> Syntax<'a> {
    pub(in crate::linter) fn read(data: &'a [u8]) -> corsa::Result<Self> {
        let tree = Tree::read(data)?;
        let count = (tree.node_end - tree.nodes) / NODE;
        if tree.field(1, 0)? != kind::SOURCE_FILE {
            return Err(invalid("unknown syntax kind numbering"));
        }
        let root_data = tree.field(1, 20)?;
        if root_data & DATA_TYPE != 0x8000_0000 {
            return Err(invalid("missing source-file metadata"));
        }
        let root = tree
            .extended
            .checked_add((root_data & 0x3fff_ffff) as usize)
            .ok_or_else(|| invalid("metadata overflow"))?;
        if root.checked_add(12).is_none_or(|end| end > tree.structured) {
            return Err(invalid("truncated source-file metadata"));
        }
        let path = std::str::from_utf8(tree.string(word(data, root + 8)?)?)
            .map_err(|_| invalid("invalid source path"))?;
        Ok(Self { tree, count, path })
    }

    pub(in crate::linter) fn kind(&self, index: usize) -> Option<u32> {
        (index > 0 && index < self.count)
            .then(|| self.tree.field(index, 0).ok())
            .flatten()
    }

    fn link(&self, index: usize, field: usize) -> Option<usize> {
        let linked = self.tree.field(index, field).ok()? as usize;
        (linked > 0 && linked < self.count).then_some(linked)
    }

    fn first_child(&self, index: usize) -> Option<usize> {
        let child = index.checked_add(1)?;
        (self.kind(child).is_some() && self.link(child, 16) == Some(index)).then_some(child)
    }

    /// The child at property `bit` of node `index`, in its kind's visitor
    /// order, when that property is present.
    pub(in crate::linter) fn property(&self, index: usize, bit: u32) -> Option<usize> {
        if self.kind(index)? == kind::LIST {
            return None;
        }
        let data = self.tree.field(index, 20).ok()?;
        let mask = data & 0xff;
        if data & DATA_TYPE != 0 || bit >= 8 || mask & (1 << bit) == 0 {
            return None;
        }
        let mut child = self.first_child(index)?;
        for _ in 0..(mask & ((1 << bit) - 1)).count_ones() {
            child = self.link(child, 12)?;
        }
        Some(child)
    }

    /// The node index a declaration handle names in this file: the compact
    /// `<index>.<kind>.<path>` form, or a full `<pos>.<end>.<kind>.<path>`.
    pub(in crate::linter) fn index_of(&self, handle: &NodeHandle) -> Option<usize> {
        if let Ok(parsed) = handle.parse() {
            if parsed.path.as_str() != self.path {
                return None;
            }
            return (1..self.count).find(|&index| {
                self.kind(index) == Some(u32::from(parsed.kind))
                    && self.tree.field(index, 4).ok() == Some(parsed.pos)
                    && self.tree.field(index, 8).ok() == Some(parsed.end)
            });
        }
        let mut parts = handle.as_str().splitn(3, '.');
        let index = parts.next()?.parse::<usize>().ok()?;
        let kind = parts.next()?.parse::<u32>().ok()?;
        (parts.next()? == self.path && self.kind(index) == Some(kind)).then_some(index)
    }

    pub(in crate::linter) fn handle(&self, index: usize) -> Option<NodeHandle> {
        let kind = self.kind(index)?;
        Some(NodeHandle::from(
            cstr!("{index}.{kind}.{}", self.path).as_str(),
        ))
    }
}
