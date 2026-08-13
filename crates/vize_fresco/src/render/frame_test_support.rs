//! Shared render-tree and failing-writer fixtures for frame tests.

use std::io::{self, Write};

use super::{NodeKind, RenderNode, RenderTree};
use crate::{layout::Dimension, terminal::Color};

pub(super) fn text_tree(value: &str) -> RenderTree {
    let mut tree = RenderTree::new();
    let id = tree.next_id();
    let mut node = RenderNode::text_node(id, value);
    node.style.width = Dimension::Points(4.0);
    node.style.height = Dimension::Points(1.0);
    tree.insert_root(node);
    tree
}

pub(super) fn styled_child_tree(text: &str) -> RenderTree {
    let mut tree = RenderTree::new();
    let root = tree.next_id();
    tree.insert_root(RenderNode::box_node(root));
    let child = tree.next_id();
    let mut node = RenderNode::text_node(child, text);
    node.style.width = Dimension::Points(text.chars().count() as f32 + 2.0);
    node.style.height = Dimension::Points(1.0);
    node.appearance.fg = Some(Color::Red);
    node.appearance.bg = Some(Color::Blue);
    node.appearance.inverse = true;
    node.appearance.underline = true;
    tree.insert(node);
    tree.add_child(root, child);
    tree
}

pub(super) fn set_root_text(tree: &mut RenderTree, text: &str) {
    let root = tree.root().unwrap();
    let node = tree.get_mut(root).unwrap();
    let NodeKind::Text(content) = &mut node.kind else {
        panic!("test tree root must be text");
    };
    content.text = text.into();
}

/// Writer that fails every presentation attempt.
#[derive(Debug)]
pub(super) struct AlwaysFailWriter;

impl Write for AlwaysFailWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("injected frame failure"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("injected frame failure"))
    }
}

/// Writer that fails the next presentation attempt while it is armed.
#[derive(Debug, Default)]
pub(super) struct FailOnceWriter {
    pub(super) armed: bool,
    pub(super) output: Vec<u8>,
}

impl FailOnceWriter {
    pub(super) fn armed() -> Self {
        Self {
            armed: true,
            output: Vec::new(),
        }
    }
}

impl Write for FailOnceWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if std::mem::take(&mut self.armed) {
            return Err(io::Error::other("injected first-frame failure"));
        }
        self.output.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
