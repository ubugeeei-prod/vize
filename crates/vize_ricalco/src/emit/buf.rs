//! Indenting JS buffer and the tiny helper set this installment uses.

use vize_carton::String;

/// Vue helpers this installment can mention, ranked the way
/// `vue_helper_import_rank` orders the shipped preamble (vnode creates
/// before `openBlock` before block creates).
#[derive(Clone, Copy)]
enum Helper {
    CreateElementVNode,
    OpenBlock,
    CreateElementBlock,
}

impl Helper {
    const ALL: [Self; 3] = [
        Self::CreateElementVNode,
        Self::OpenBlock,
        Self::CreateElementBlock,
    ];

    const fn bit(self) -> u8 {
        match self {
            Self::CreateElementVNode => 1,
            Self::OpenBlock => 2,
            Self::CreateElementBlock => 4,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::CreateElementVNode => "createElementVNode",
            Self::OpenBlock => "openBlock",
            Self::CreateElementBlock => "createElementBlock",
        }
    }

    const fn alias(self) -> &'static str {
        match self {
            Self::CreateElementVNode => "_createElementVNode",
            Self::OpenBlock => "_openBlock",
            Self::CreateElementBlock => "_createElementBlock",
        }
    }
}

/// Growing JS text plus the helpers the body mentioned.
pub(super) struct Buf {
    pub code: String,
    indent: u32,
    used: u8,
}

impl Buf {
    pub(super) fn new() -> Self {
        Self {
            code: String::default(),
            indent: 0,
            used: 0,
        }
    }

    pub(super) fn push(&mut self, text: &str) {
        self.code.push_str(text);
    }

    pub(super) fn newline(&mut self) {
        self.code.push('\n');
        for _ in 0..self.indent {
            self.code.push_str("  ");
        }
    }

    pub(super) fn indent(&mut self) {
        self.indent += 1;
    }

    pub(super) fn deindent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    pub(super) fn use_open_block(&mut self) {
        self.used |= Helper::OpenBlock.bit();
    }

    pub(super) fn use_create_element_block(&mut self) {
        self.used |= Helper::CreateElementBlock.bit();
    }

    pub(super) fn use_create_element_vnode(&mut self) {
        self.used |= Helper::CreateElementVNode.bit();
    }

    pub(super) fn open_block_alias() -> &'static str {
        Helper::OpenBlock.alias()
    }

    pub(super) fn create_element_block_alias() -> &'static str {
        Helper::CreateElementBlock.alias()
    }

    pub(super) fn create_element_vnode_alias() -> &'static str {
        Helper::CreateElementVNode.alias()
    }

    /// Function-mode preamble, helpers in import-rank order.
    pub(super) fn preamble(&self) -> String {
        let listed: [Helper; 3] = Helper::ALL;
        let mut n = 0;
        for helper in listed {
            if self.used & helper.bit() != 0 {
                n += 1;
            }
        }
        if n == 0 {
            return String::default();
        }
        let mut preamble = String::from("const { ");
        let mut first = true;
        for helper in listed {
            if self.used & helper.bit() == 0 {
                continue;
            }
            if !first {
                preamble.push_str(", ");
            }
            first = false;
            preamble.push_str(helper.name());
            preamble.push_str(": ");
            preamble.push_str(helper.alias());
        }
        preamble.push_str(" } = Vue\n");
        preamble
    }
}
