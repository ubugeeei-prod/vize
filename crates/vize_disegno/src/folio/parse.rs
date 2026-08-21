//! Parser for the disegno folio text format.
//!
//! Accepts canonical output byte-for-byte and is lenient exactly where the
//! printer normalizes: blank-line placement (separators vanish), the
//! `ops=` count (validated as an integer, recomputed by print), and
//! non-canonical integer spellings inside spans. Everything else is
//! strict: the section order is fixed because there are exactly two
//! sections, the grouping under an element (attributes, bindings,
//! children) is part of the grammar, and a line that does not match is a
//! [`FolioError`] with a 1-based line number.
//!
//! Tree structure is driven by indentation (two spaces per level) over a
//! frame stack: container ops open a frame, a shallower line closes every
//! deeper frame back into its owner. Only tree *shape* is validated here;
//! semantic invariants belong to the S2 verifier (P2-6).

use alloc::vec::Vec;

use vize_carton::cstr;
use vize_davinci::folio::FolioError;

use super::DisegnoFolio;
use super::owned::{FolioBinding, FolioOp};

mod binding_line;
mod expr_token;
mod frame;
mod line;

use frame::{Frame, Phase};
use line::{Item, err};

/// Where in the page the scan currently is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    BeforeHeader,
    Header,
    Ops,
}

struct Parser {
    root: Vec<FolioOp>,
    stack: Vec<Frame>,
    section: Section,
    seen_ops_section: bool,
    seen_ops_field: bool,
}

pub(super) fn parse(input: &str) -> Result<DisegnoFolio, FolioError> {
    let mut parser = Parser {
        root: Vec::new(),
        stack: Vec::new(),
        section: Section::BeforeHeader,
        seen_ops_section: false,
        seen_ops_field: false,
    };
    for (idx, line) in input.split('\n').enumerate() {
        parser.line(line, idx + 1)?;
    }
    parser.finish()
}

impl Parser {
    fn line(&mut self, line: &str, line_no: usize) -> Result<(), FolioError> {
        if let Some(name) = line.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
            return self.enter(name, line_no);
        }
        if line.is_empty() {
            return Ok(());
        }
        match self.section {
            Section::BeforeHeader => {
                Err(err(line_no, cstr!("content before the [disegno] header")))
            }
            Section::Header => self.field_line(line, line_no),
            Section::Ops => self.ops_line(line, line_no),
        }
    }

    fn enter(&mut self, name: &str, line_no: usize) -> Result<(), FolioError> {
        if self.section == Section::BeforeHeader {
            if name == "disegno" {
                self.section = Section::Header;
                return Ok(());
            }
            return Err(err(line_no, cstr!("first section must be [disegno]")));
        }
        match name {
            "disegno" => Err(err(line_no, cstr!("duplicate section [disegno]"))),
            "disegno.ops" => {
                if self.seen_ops_section {
                    return Err(err(line_no, cstr!("duplicate section [disegno.ops]")));
                }
                self.seen_ops_section = true;
                self.section = Section::Ops;
                Ok(())
            }
            other => Err(err(line_no, cstr!("unknown section [{other}]"))),
        }
    }

    fn field_line(&mut self, line: &str, line_no: usize) -> Result<(), FolioError> {
        let Some((name, value)) = line.split_once('=') else {
            return Err(err(line_no, cstr!("field line is missing `=`")));
        };
        if name != "ops" {
            return Err(err(line_no, cstr!("unknown field `{name}`")));
        }
        if self.seen_ops_field {
            return Err(err(line_no, cstr!("duplicate field `ops`")));
        }
        if value.parse::<u64>().is_err() {
            return Err(err(line_no, cstr!("invalid integer `{value}`")));
        }
        // The count is the printer's statement about the tree; parse
        // validates the syntax and discards the value.
        self.seen_ops_field = true;
        Ok(())
    }

    fn ops_line(&mut self, line: &str, line_no: usize) -> Result<(), FolioError> {
        let content = line.trim_start_matches(' ');
        let spaces = line.len() - content.len();
        if !spaces.is_multiple_of(2) {
            return Err(err(line_no, cstr!("odd indentation")));
        }
        let depth = spaces / 2;
        if depth > self.stack.len() {
            return Err(err(line_no, cstr!("over-indented line")));
        }
        while self.stack.len() > depth {
            self.close_top();
        }
        match line::parse_item(content, line_no)? {
            Item::Attr(attribute) => self.push_attr(attribute, line_no),
            Item::Model(model) => {
                self.guard_binding(line_no)?;
                self.stack.push(Frame::Model(model));
                Ok(())
            }
            Item::Bind(bind) => {
                self.guard_binding(line_no)?;
                self.push_leaf_binding(FolioBinding::Bind(bind));
                Ok(())
            }
            Item::On(on) => {
                self.guard_binding(line_no)?;
                self.push_leaf_binding(FolioBinding::On(on));
                Ok(())
            }
            Item::SlotContent(content) => {
                self.guard_binding(line_no)?;
                self.push_leaf_binding(FolioBinding::SlotContent(content));
                Ok(())
            }
            Item::Directive(directive) => {
                self.guard_binding(line_no)?;
                self.push_leaf_binding(FolioBinding::VueDirective(directive));
                Ok(())
            }
            Item::Branch(branch) => match self.stack.last() {
                Some(Frame::If(_)) => {
                    self.stack.push(Frame::Branch(branch));
                    Ok(())
                }
                None
                | Some(
                    Frame::Element(..)
                    | Frame::Component(..)
                    | Frame::Model(_)
                    | Frame::Branch(_)
                    | Frame::For(_)
                    | Frame::Slot(..),
                ) => Err(err(line_no, cstr!("`branch` outside `ui.if`"))),
            },
            Item::Op(op) => {
                self.guard_child(line_no)?;
                match op {
                    FolioOp::Element(element) => {
                        self.stack.push(Frame::Element(element, Phase::Attrs));
                    }
                    FolioOp::Component(component) => {
                        self.stack.push(Frame::Component(component, Phase::Attrs));
                    }
                    FolioOp::If(if_op) => self.stack.push(Frame::If(if_op)),
                    FolioOp::For(for_op) => self.stack.push(Frame::For(for_op)),
                    FolioOp::Slot(slot) => self.stack.push(Frame::Slot(slot, Phase::Attrs)),
                    FolioOp::Text(_) | FolioOp::Interpolation(_) => self.attach_op(op),
                    #[cfg(feature = "_legacy")]
                    FolioOp::VueFilter(_) => self.attach_op(op),
                }
                Ok(())
            }
        }
    }

    fn finish(mut self) -> Result<DisegnoFolio, FolioError> {
        if self.section == Section::BeforeHeader {
            return Err(err(0, cstr!("missing [disegno] header")));
        }
        if !self.seen_ops_field {
            return Err(err(0, cstr!("missing field `ops`")));
        }
        while !self.stack.is_empty() {
            self.close_top();
        }
        Ok(DisegnoFolio { ops: self.root })
    }
}
