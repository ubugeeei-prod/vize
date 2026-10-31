//! Non-block Vue creation for producer-owned opaque child values.
use super::{EmitCx, buf::Buf};

impl EmitCx<'_> {
    pub(super) fn use_open_block(&mut self) {
        if !self.untracked_children {
            self.buf.use_open_block();
        }
    }

    pub(super) fn use_element_block(&mut self) {
        if self.untracked_children {
            self.buf.use_create_element_vnode();
        } else {
            self.buf.use_create_element_block();
        }
    }

    pub(super) fn element_block_alias(&self) -> &'static str {
        if self.untracked_children {
            Buf::create_element_vnode_alias()
        } else {
            Buf::create_element_block_alias()
        }
    }

    pub(super) fn use_component_block(&mut self) {
        if self.untracked_children {
            self.buf.use_create_vnode();
        } else {
            self.buf.use_create_block();
        }
    }

    pub(super) fn component_block_alias(&self) -> &'static str {
        if self.untracked_children {
            Buf::create_vnode_alias()
        } else {
            Buf::create_block_alias()
        }
    }
}
