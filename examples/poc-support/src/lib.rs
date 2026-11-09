//! Shared plumbing for the level proof-of-concept examples under `examples/`.
//!
//! Every example runs one pipeline: its own dialect lowering builds a neutral
//! L2 region, [`lower`] runs the shared L2 to L3 lowering and the L3 verifier,
//! and the example's own L4 target walks the program through [`View`] and writes
//! text with the L4 writer. Nothing here names a framework or input language.

use std::fmt;
use std::process::ExitCode;

use oxc_span::SourceType;
use vize_l0::{Allocator, Span, String, cstr};
use vize_l2::op::Region;
use vize_l2_to_l3::Lowered;
use vize_l3::op::{Op, OpId, OpKind, Program, RegionId};
use vize_l3::operand::{Operand, OperandRole, ValueKind};

pub mod vue;

/// A located refusal. A proof of concept refuses what it does not cover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub message: String,
    pub span: Span,
}

impl Error {
    #[must_use]
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {}..{}",
            self.message, self.span.start, self.span.end
        )
    }
}

/// Lower one neutral L2 region to L3, refusing a program the L3 verifier rejects.
pub fn lower<'a>(allocator: &'a Allocator, root: &Region<'_>) -> Result<Lowered<'a>, Error> {
    let lowered = vize_l2_to_l3::lower(allocator, root);
    match vize_l3::verify::verify(&lowered.program).first() {
        Some(violation) => Err(Error::new(
            cstr!("L3 verifier: {}", violation.code.as_str()),
            violation.span,
        )),
        None => Ok(lowered),
    }
}

/// A tree view over one flat L3 program.
#[derive(Clone, Copy)]
pub struct View<'p, 'a> {
    program: &'p Program<'a>,
}

impl<'p, 'a> View<'p, 'a> {
    #[must_use]
    pub fn new(program: &'p Program<'a>) -> Self {
        Self { program }
    }

    /// Child-position ops of `region` in document order. Binding ops are
    /// reached through their target with [`Node::bindings`].
    pub fn children(self, region: RegionId) -> impl Iterator<Item = Node<'p, 'a>> {
        self.program
            .ops
            .iter()
            .filter(move |op| op.region == region)
            .map(move |op| Node { view: self, op })
            .filter(|node| node.target().is_none())
    }

    pub fn roots(self) -> impl Iterator<Item = Node<'p, 'a>> {
        self.children(RegionId::ROOT)
    }
}

/// One L3 op with its operands, bindings and owned regions.
#[derive(Clone, Copy)]
pub struct Node<'p, 'a> {
    view: View<'p, 'a>,
    op: &'p Op,
}

impl<'p, 'a> Node<'p, 'a> {
    #[must_use]
    pub fn id(self) -> OpId {
        self.op.id
    }

    #[must_use]
    pub fn kind(self) -> OpKind {
        self.op.kind
    }

    #[must_use]
    pub fn span(self) -> Span {
        self.op.span
    }

    /// Whether L3 gave the op an effect scope: its value can change after mount.
    #[must_use]
    pub fn is_dynamic(self) -> bool {
        self.op.effect.is_some()
    }

    pub fn operands(self, role: OperandRole) -> impl Iterator<Item = &'p Operand<'a>> {
        let id = self.op.id;
        self.view
            .program
            .operands
            .iter()
            .filter(move |operand| operand.op == id && operand.role == role)
    }

    #[must_use]
    pub fn operand(self, role: OperandRole) -> Option<&'p Operand<'a>> {
        self.operands(role).next()
    }

    /// The literal text of the first operand with `role`.
    #[must_use]
    pub fn literal(self, role: OperandRole) -> Option<&'a str> {
        self.operand(role)
            .filter(|operand| operand.value.kind == ValueKind::Literal)
            .map(|operand| operand.value.text)
    }

    /// The element or component a binding op is attached to.
    #[must_use]
    pub fn target(self) -> Option<OpId> {
        self.operand(OperandRole::BindingKind)?.target
    }

    /// `bind`, `on`, `model`, ... for a binding op.
    #[must_use]
    pub fn binding_kind(self) -> Option<&'a str> {
        self.literal(OperandRole::BindingKind)
    }

    pub fn bindings(self) -> impl Iterator<Item = Node<'p, 'a>> {
        let view = self.view;
        let id = self.op.id;
        view.program
            .ops
            .iter()
            .map(move |op| Node { view, op })
            .filter(move |node| node.target() == Some(id))
    }

    /// Regions owned by this op: element content, one per conditional branch,
    /// the repeated body of a loop.
    pub fn regions(self) -> impl Iterator<Item = RegionId> + 'p {
        let id = self.op.id;
        self.view
            .program
            .regions
            .iter()
            .filter(move |region| region.owner == Some(id))
            .map(|region| region.id)
    }

    /// The content of `region`.
    pub fn region(self, region: RegionId) -> impl Iterator<Item = Node<'p, 'a>> {
        self.view.children(region)
    }

    /// The content of this op's first owned region.
    pub fn children(self) -> impl Iterator<Item = Node<'p, 'a>> {
        let view = self.view;
        self.regions()
            .next()
            .into_iter()
            .flat_map(move |region| view.children(region))
    }
}

/// The source of a JavaScript value operand; other value kinds are refused.
pub fn js<'a>(operand: &Operand<'a>) -> Result<&'a str, Error> {
    match operand.value.kind {
        ValueKind::Js => Ok(operand.value.text),
        kind => Err(Error::new(
            cstr!("unsupported {} value", kind.as_str()),
            operand.value.span,
        )),
    }
}

/// A double-quoted JavaScript string literal.
#[must_use]
pub fn js_string(value: &str) -> String {
    let mut out = String::default();
    vize_l0::js_string::push_js_string_literal(&mut out, value);
    out
}

/// Text escaped for HTML character data and double-quoted attribute values.
#[must_use]
pub fn escape_html(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            ch => out.push(ch),
        }
    }
    out
}

/// `text` with `prefix` after every line break that starts a non-empty line.
#[must_use]
pub fn indent(text: &str, prefix: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for (index, line) in text.split('\n').enumerate() {
        if index > 0 {
            out.push('\n');
            if !line.is_empty() {
                out.push_str(prefix);
            }
        }
        out.push_str(line);
    }
    out
}

/// `text` without the indentation its non-blank lines share, and without
/// leading and trailing blank lines.
#[must_use]
pub fn dedent(text: &str) -> String {
    let lines: Vec<&str> = text.trim_matches('\n').lines().collect();
    let width = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);
    let mut out = String::with_capacity(text.len());
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(line.get(width..).unwrap_or_default());
    }
    out
}

/// The span of `slice` inside `source`; `slice` must borrow from `source`.
#[must_use]
pub fn span_in(source: &str, slice: &str) -> Span {
    let start = (slice.as_ptr() as usize).saturating_sub(source.as_ptr() as usize);
    let start = u32::try_from(start).unwrap_or(u32::MAX);
    let len = u32::try_from(slice.len()).unwrap_or(u32::MAX);
    Span::new(start, start.saturating_add(len))
}

/// The leading whitespace of the line that contains byte `offset`.
#[must_use]
pub fn line_indent(source: &str, offset: u32) -> &str {
    let before = source.get(..offset as usize).unwrap_or_default();
    let line = before.rsplit('\n').next().unwrap_or_default();
    let width = line.len() - line.trim_start().len();
    line.get(..width).unwrap_or_default()
}

/// Parse emitted JavaScript as an ES module and return its first syntax error.
pub fn check_module(source: &str) -> Result<(), String> {
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, SourceType::mjs()).parse();
    match parsed.diagnostics.iter().next() {
        Some(error) => Err(cstr!("{error}")),
        None => Ok(()),
    }
}

/// Compare `actual` with a checked-in snapshot. `VIZE_POC_UPDATE=1` rewrites it.
pub fn snapshot(path: &str, actual: &str) -> Result<(), String> {
    if std::env::var_os("VIZE_POC_UPDATE").is_some() {
        return std::fs::write(path, actual).map_err(|error| cstr!("{path}: {error}"));
    }
    let expected = std::fs::read(path).map_err(|error| cstr!("{path}: {error}"))?;
    if expected == actual.as_bytes() {
        Ok(())
    } else {
        Err(cstr!(
            "{path} is stale; rerun with VIZE_POC_UPDATE=1 to accept:\n{actual}"
        ))
    }
}

/// `<example> <input file>`: print the compiled module or the refusal.
pub fn run_cli(compile: impl Fn(&str) -> Result<String, Error>) -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: <example> <input file>");
        return ExitCode::FAILURE;
    };
    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("{path}: {error}");
            return ExitCode::FAILURE;
        }
    };
    match compile(&source) {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{path}: {error}");
            ExitCode::FAILURE
        }
    }
}
