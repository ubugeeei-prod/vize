//! Structural recovery shared by node construction and live scope consumers.
//!
//! Start tags close open elements the way the shipped parser's "in body" and
//! "in table" rules do (`vize_armature/src/parser/element/{scope,table}.rs`).
//! L1 keeps authored order, so the rules that insert or move nodes (implicit
//! `tbody`/`tr`, foster parenting) stay in L1->L2 lowering.

use vize_l0::{Allocator, Namespace, Vec, is_html_tag, is_math_ml_tag, is_svg_tag};

/// A caller's open-element frame; recovery never owns or copies its payload.
pub(crate) trait Frame<'a> {
    fn tag(&self) -> &'a str;
    fn namespace(&self) -> Namespace;
}

struct Closed<'a> {
    tag: &'a str,
    depth: usize,
}

/// Open frames the start-tag rules search for, by depth. Searches start at
/// the innermost mark, so a template without these tags never scans its stack.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    P,
    Li,
    DtDd,
    Option,
    /// A component spelled `Option`: the shipped parser closes it by name.
    OptionName,
    Optgroup,
    Table,
    Section,
    Row,
}

/// Holds only the redundant-end-tag evidence already needed by the builder.
pub(crate) struct Recovery<'a> {
    closed: Vec<'a, Closed<'a>>,
    marks: Vec<'a, (usize, Mark)>,
    repair_interactive: bool,
}

impl<'a> Recovery<'a> {
    pub(crate) fn new(allocator: &'a Allocator, repair_interactive: bool) -> Self {
        Self {
            closed: Vec::new_in(&allocator),
            marks: Vec::new_in(&allocator),
            repair_interactive,
        }
    }

    /// Compute the namespace and first implicitly closed depth. The caller
    /// pops its own frames, preserving node payloads without a second stack.
    pub(crate) fn open<F: Frame<'a>>(
        &mut self,
        stack: &[F],
        tag: &str,
        self_closing: bool,
    ) -> (Namespace, Option<usize>) {
        let ns = namespace(stack, tag);
        if !self.repair_interactive || ns != Namespace::Html {
            return (ns, None);
        }
        if interactive(tag) {
            let depth = (!self_closing)
                .then(|| self.interactive_depth(stack, tag))
                .flatten();
            if let Some(depth) = depth {
                self.truncate(depth);
            }
            return (ns, depth);
        }
        // Marked names are 1, 2, 5, 6 or 8 bytes long.
        if self.marks.is_empty() && !matches!(tag.len(), 1 | 2 | 5 | 6 | 8) {
            return (ns, None);
        }
        (ns, self.implied(stack, tag, self_closing))
    }

    /// Optional end tags close silently: a later end tag of a closed
    /// descendant stays a stray end tag, as in the shipped parser.
    #[inline(never)]
    fn implied<F: Frame<'a>>(
        &mut self,
        stack: &[F],
        tag: &str,
        self_closing: bool,
    ) -> Option<usize> {
        let depth = self.implied_depth(stack, tag);
        let live = depth.unwrap_or(stack.len());
        self.truncate(live);
        if !self_closing && let Some(mark) = mark(tag) {
            self.marks.push((live, mark));
        }
        depth
    }

    /// Matching an ancestor also ends descendants; stray and redundant
    /// implicitly closed tags leave the caller's live scope intact.
    pub(crate) fn close<F: Frame<'a>>(&mut self, stack: &[F], tag: &str) -> Option<usize> {
        let matched = stack
            .iter()
            .rposition(|frame| frame.tag().eq_ignore_ascii_case(tag));
        if self.closed.last().is_some_and(|closed| {
            closed.tag.eq_ignore_ascii_case(tag) && matched.is_none_or(|depth| depth < closed.depth)
        }) {
            self.closed.pop();
            return None;
        }
        if let Some(depth) = matched {
            self.truncate(depth);
        }
        matched
    }

    fn truncate(&mut self, depth: usize) {
        while self.marks.last().is_some_and(|(at, _)| *at >= depth) {
            self.marks.pop();
        }
    }

    fn interactive_depth<F: Frame<'a>>(&mut self, stack: &[F], tag: &str) -> Option<usize> {
        let depth = stack.iter().rposition(|frame| {
            frame.namespace() == Namespace::Html
                && interactive(frame.tag())
                && frame.tag().eq_ignore_ascii_case(tag)
        })?;
        for (depth, frame) in stack.iter().enumerate().skip(depth) {
            self.closed.push(Closed {
                tag: frame.tag(),
                depth,
            });
        }
        Some(depth)
    }

    /// The innermost mark of `kinds`, its depth.
    fn last(&self, kinds: &[Mark]) -> Option<usize> {
        self.marks
            .iter()
            .rev()
            .find(|(_, mark)| kinds.contains(mark))
            .map(|(depth, _)| *depth)
    }

    /// The shipped parser's start-tag closes. Only lowercase HTML names are
    /// tree elements there; any other spelling is a component.
    fn implied_depth<F: Frame<'a>>(&self, stack: &[F], tag: &str) -> Option<usize> {
        // Every rule closes a marked frame; most templates have none open.
        if self.marks.is_empty() {
            return None;
        }
        match tag {
            "li" => scoped(stack, self.last(&[Mark::Li])?, LIST_ITEM_SCOPE),
            "dt" | "dd" => self.last(&[Mark::DtDd]),
            "option" => self.last(&[Mark::Option, Mark::OptionName]),
            "optgroup" => self.last(&[Mark::Option, Mark::Optgroup]),
            "tbody" | "thead" | "tfoot" => self.table_sibling(stack, Mark::Section),
            "tr" => self.table_sibling(stack, Mark::Row),
            _ if CLOSES_P.contains(&tag) => scoped(stack, self.last(&[Mark::P])?, BUTTON_SCOPE),
            _ => None,
        }
    }

    /// A row group closes the open row group, and a row the open row, when
    /// it is the current node of the table insertion context.
    fn table_sibling<F: Frame<'a>>(&self, stack: &[F], kind: Mark) -> Option<usize> {
        let top = stack.len().checked_sub(1)?;
        let table = self.last(&[Mark::Table])?;
        let current = self.marks.last().filter(|(depth, _)| *depth == top)?.1;
        (current == kind && in_table_context(stack, table)).then_some(top)
    }
}

/// `target` is reachable from the insertion point unless a scope boundary
/// or a non-element (component, slot, structural template) is open above it.
fn scoped<'a, F: Frame<'a>>(stack: &[F], target: usize, boundary: &[&str]) -> Option<usize> {
    let above = stack.get(target + 1..)?;
    above
        .iter()
        .all(|frame| tree_element(frame) && !boundary.contains(&frame.tag()))
        .then_some(target)
}

/// Every frame above the table is a table part the shipped parser would not
/// foster, and none is a cell, caption or template. Its implicit `tbody` and
/// `tr` owners are absent here, so a row fits under any table part.
fn in_table_context<'a, F: Frame<'a>>(stack: &[F], table: usize) -> bool {
    let mut parent = "table";
    for frame in stack.get(table + 1..).unwrap_or_default() {
        let tag = frame.tag();
        let fits = match tag {
            "tbody" | "thead" | "tfoot" | "colgroup" => matches!(parent, "table" | "colgroup"),
            "tr" => true,
            _ => false,
        };
        if !fits || frame.namespace() != Namespace::Html {
            return false;
        }
        parent = tag;
    }
    true
}

fn tree_element<'a, F: Frame<'a>>(frame: &F) -> bool {
    let tag = frame.tag();
    frame.namespace() == Namespace::Html && is_html_tag(tag) && !matches!(tag, "slot" | "component")
}

/// The mark of an HTML-namespace frame.
fn mark(tag: &str) -> Option<Mark> {
    Some(match tag {
        "p" => Mark::P,
        "li" => Mark::Li,
        "dt" | "dd" => Mark::DtDd,
        "option" => Mark::Option,
        "optgroup" => Mark::Optgroup,
        "table" => Mark::Table,
        "tbody" | "thead" | "tfoot" => Mark::Section,
        "tr" => Mark::Row,
        _ if tag.eq_ignore_ascii_case("option") => Mark::OptionName,
        _ => return None,
    })
}

fn namespace<'a, F: Frame<'a>>(stack: &[F], tag: &str) -> Namespace {
    if is_svg_tag(tag) {
        Namespace::Svg
    } else if is_math_ml_tag(tag) {
        Namespace::MathMl
    } else {
        stack
            .last()
            .map_or(Namespace::Html, |frame| match frame.namespace() {
                Namespace::Svg if matches!(frame.tag(), "foreignObject" | "desc" | "title") => {
                    Namespace::Html
                }
                Namespace::MathMl if matches!(frame.tag(), "mi" | "mo" | "mn" | "ms" | "mtext") => {
                    Namespace::Html
                }
                other => other,
            })
    }
}

fn interactive(tag: &str) -> bool {
    (tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("button")) && is_html_tag(tag)
}

const BUTTON_SCOPE: &[&str] = &[
    "applet", "button", "caption", "html", "table", "td", "th", "marquee", "object", "template",
];

const LIST_ITEM_SCOPE: &[&str] = &[
    "applet", "caption", "html", "table", "td", "th", "marquee", "object", "template", "ol", "ul",
];

const CLOSES_P: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "div",
    "dl",
    "fieldset",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "main",
    "nav",
    "ol",
    "p",
    "pre",
    "section",
    "table",
    "ul",
];
