//! Hello, dialect: the smallest input dialect built on `vize_extension_sdk`.
//!
//! A `lang="hello"` block lists one name per line; the dialect greets each.
//! S1 records every line as one text token (tiling the block), S2 lowers the
//! block to a list:
//!
//! ```text
//! Ada          ->  ui.element ul
//! Grace              ui.element li   ui.text "Hello, Ada!"
//!                    ui.element li   ui.text "Hello, Grace!"
//! ```
//!
//! CI builds it for `wasm32-wasip2` against the packed SDK tarball alone
//! (`crates/vize_extension_host/tests/sdk_hello.rs`) and runs it through the
//! host in both hosting modes.

#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use vize_extension_sdk::handshake::{Capability, Guest as Handshake};
use vize_extension_sdk::input_lowering::{Guest as Lowering, LoweredBlock, SourceBlock};
use vize_extension_sdk::pages::s1::{Node, SurfacePage, Token};
use vize_extension_sdk::pages::s2::{Op, SemanticPage};
use vize_extension_sdk::pages::{s1_page, s2_page};
use vize_extension_sdk::types::Span;

struct Hello;

impl Handshake for Hello {
    fn get_capability() -> Capability {
        vize_extension_sdk::capability(&["hello"])
    }
}

impl Lowering for Hello {
    fn lower_block(block: SourceBlock) -> LoweredBlock {
        let mut tokens = Vec::new();
        let mut items = Vec::new();
        let mut start = 0u32;
        for line in block.source.split_inclusive('\n') {
            let end = start + line.len() as u32;
            tokens.push(Node::Text(Token::new(start, start, end)));
            let name = line.trim_end_matches(['\n', '\r']);
            if !name.is_empty() {
                let at = Span {
                    start: block.base + start,
                    end: block.base + start + name.len() as u32,
                };
                let mut greeting = String::from("Hello, ");
                greeting.push_str(name);
                greeting.push('!');
                items.push(Op::Element {
                    tag: String::from("li"),
                    attrs: Vec::new(),
                    children: Vec::from([Op::Text {
                        value: greeting,
                        span: at,
                    }]),
                    span: at,
                });
            }
            start = end;
        }
        let list = Op::Element {
            tag: String::from("ul"),
            attrs: Vec::new(),
            children: items,
            span: Span {
                start: block.base,
                end: block.base + start,
            },
        };
        LoweredBlock {
            surface: s1_page(SurfacePage { children: tokens }.text()),
            semantic: s2_page(
                SemanticPage {
                    ops: Vec::from([list]),
                }
                .text(),
            ),
            diagnostics: Vec::new(),
        }
    }
}

vize_extension_sdk::export_input_dialect!(Hello);
