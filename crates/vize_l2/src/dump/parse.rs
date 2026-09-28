//! Parser for the selected current or explicit historical L2 dump grammar.
//!
//! Accepts canonical output byte-for-byte and is lenient exactly where the
//! printer normalizes: blank-line placement (separators vanish), the
//! `ops=` count (validated as an integer, recomputed by print), and
//! non-canonical integer spellings inside spans. Everything else is
//! strict: the section order is fixed because there are exactly two
//! sections, the grouping under an element (attributes, bindings,
//! children) is part of the grammar, and a line that does not match is a
//! [`DumpError`] with a 1-based line number.
//!
//! Tree structure is driven by indentation (two spaces per level) over a
//! frame stack: container ops open a frame, a shallower line closes every
//! deeper frame back into its owner. Only tree *shape* is validated here;
//! semantic invariants belong to the L2 verifier (P2-6).

use alloc::vec::Vec;

use vize_davinci::dump::Error as DumpError;
use vize_l0::cstr;

use crate::dump::Page;
use crate::dump::codec::Grammar;
use crate::dump::owned::{Binding, Op};

mod binding_line;
mod expr_token;
mod frame;
mod line;

use frame::{Frame, Phase};
use line::{Item, err};

/// The page's quoted-string grammar, for sibling pages (the provenance
/// page) that share its escapes.
pub(super) fn take_quoted_value(
    rest: &str,
    line: usize,
) -> Result<(vize_l0::String, &str), DumpError> {
    line::take_quoted(rest, line)
}

/// The page's ` @start:end` line tail, for sibling pages.
pub(super) fn tail_span_value(rest: &str, line: usize) -> Result<vize_l0::Span, DumpError> {
    line::tail_span(rest, line)
}

/// Where in the page the scan currently is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    BeforeHeader,
    Header,
    Ops,
}

struct Parser {
    grammar: Grammar,
    root: Vec<Op>,
    stack: Vec<Frame>,
    section: Section,
    seen_ops_section: bool,
    seen_ops_field: bool,
}

pub(super) fn parse(input: &str) -> Result<Page, DumpError> {
    Parser::current_v2().read(input)
}

pub(super) fn parse_historical_v1(input: &str) -> Result<Page, DumpError> {
    Parser::historical_v1().read(input)
}

impl Parser {
    fn current_v2() -> Self {
        Self::new(Grammar::CurrentV2)
    }

    fn historical_v1() -> Self {
        Self::new(Grammar::HistoricalV1)
    }

    fn new(grammar: Grammar) -> Self {
        Self {
            grammar,
            root: Vec::new(),
            stack: Vec::new(),
            section: Section::BeforeHeader,
            seen_ops_section: false,
            seen_ops_field: false,
        }
    }

    fn read(mut self, input: &str) -> Result<Page, DumpError> {
        for (idx, line) in input.split('\n').enumerate() {
            self.line(line, idx + 1)?;
        }
        self.finish()
    }

    fn line(&mut self, line: &str, line_no: usize) -> Result<(), DumpError> {
        if let Some(name) = line.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
            return self.enter(name, line_no);
        }
        if line.is_empty() {
            return Ok(());
        }
        match self.section {
            Section::BeforeHeader => {
                let header = self.grammar.header();
                Err(err(line_no, cstr!("content before the [{header}] header")))
            }
            Section::Header => self.field_line(line, line_no),
            Section::Ops => self.ops_line(line, line_no),
        }
    }

    fn enter(&mut self, name: &str, line_no: usize) -> Result<(), DumpError> {
        let header = self.grammar.header();
        let ops_header = self.grammar.ops_header();
        if self.section == Section::BeforeHeader {
            if name == header {
                self.section = Section::Header;
                return Ok(());
            }
            return Err(err(line_no, cstr!("first section must be [{header}]")));
        }
        match name {
            name if name == header => Err(err(line_no, cstr!("duplicate section [{header}]"))),
            name if name == ops_header => {
                if self.seen_ops_section {
                    return Err(err(line_no, cstr!("duplicate section [{ops_header}]")));
                }
                self.seen_ops_section = true;
                self.section = Section::Ops;
                Ok(())
            }
            other => Err(err(line_no, cstr!("unknown section [{other}]"))),
        }
    }

    fn field_line(&mut self, line: &str, line_no: usize) -> Result<(), DumpError> {
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

    fn ops_line(&mut self, line: &str, line_no: usize) -> Result<(), DumpError> {
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
            self.close_top(line_no)?;
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
                self.push_leaf_binding(Binding::Bind(bind), line_no)
            }
            Item::On(on) => {
                self.guard_binding(line_no)?;
                self.push_leaf_binding(Binding::On(on), line_no)
            }
            Item::SlotContent(content) => {
                self.guard_binding(line_no)?;
                self.push_leaf_binding(Binding::SlotContent(content), line_no)
            }
            Item::Directive(directive) => {
                self.guard_binding(line_no)?;
                self.push_leaf_binding(Binding::VueDirective(directive), line_no)
            }
            Item::CssBind(bind) => {
                self.guard_binding(line_no)?;
                self.push_leaf_binding(Binding::VueCssBind(bind), line_no)
            }
            Item::Sync(sync) => {
                self.guard_binding(line_no)?;
                self.push_leaf_binding(Binding::VueSync(sync), line_no)
            }
            Item::SlotScope(scope) => {
                self.guard_binding(line_no)?;
                self.push_leaf_binding(Binding::VueSlotScope(scope), line_no)
            }
            Item::Once(once) => {
                self.guard_binding(line_no)?;
                self.push_leaf_binding(Binding::VueOnce(once), line_no)
            }
            Item::Memo(memo) => {
                self.guard_binding(line_no)?;
                self.push_leaf_binding(Binding::VueMemo(memo), line_no)
            }
            Item::Show(show) => {
                self.guard_binding(line_no)?;
                self.push_leaf_binding(Binding::VueShow(show), line_no)
            }
            Item::Html(html) => {
                self.guard_binding(line_no)?;
                self.push_leaf_binding(Binding::VueHtml(html), line_no)
            }
            Item::VueText(text) => {
                self.guard_binding(line_no)?;
                self.push_leaf_binding(Binding::VueText(text), line_no)
            }
            Item::Cloak(cloak) => {
                self.guard_binding(line_no)?;
                self.push_leaf_binding(Binding::VueCloak(cloak), line_no)
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
                    Op::Element(element) => {
                        self.stack.push(Frame::Element(element, Phase::Attrs));
                    }
                    Op::Component(component) => {
                        self.stack.push(Frame::Component(component, Phase::Attrs));
                    }
                    Op::If(if_op) => self.stack.push(Frame::If(if_op)),
                    Op::For(for_op) => self.stack.push(Frame::For(for_op)),
                    Op::Slot(slot) => self.stack.push(Frame::Slot(slot, Phase::Attrs)),
                    Op::Text(_) | Op::Interpolation(_) | Op::Comment(_) => {
                        return self.attach_op(op, line_no);
                    }
                }
                Ok(())
            }
        }
    }

    fn finish(mut self) -> Result<Page, DumpError> {
        if self.section == Section::BeforeHeader {
            let header = self.grammar.header();
            return Err(err(0, cstr!("missing [{header}] header")));
        }
        if !self.seen_ops_field {
            return Err(err(0, cstr!("missing field `ops`")));
        }
        while !self.stack.is_empty() {
            self.close_top(0)?;
        }
        Ok(Page { ops: self.root })
    }
}
