//! Line grammar of the disegno ops section: one op (or structural) line
//! in, one parsed item out.
//!
//! Expression positions hold the payload tokens of
//! [`expr_token`](crate::dump::parse::expr_token) - `js(...)`, `opaque(...)`,
//! `foreign(...)`. Quoted strings escape `\\`, `\"`, `\n`, `\r`,
//! `\t`; values embedding other control characters, attribute names
//! containing `=`, ` ` or `"` are outside the contract (the
//! derived-page "documented edges" rule).

use vize_l0::dump::Error as DumpError;
use vize_l0::{Span, String, cstr};

use crate::dump::owned::{
    Attribute, Bind, Branch, Comment, Component, Element, For, ForBinding, If, Interpolation,
    Model, Name, On, Op, OriginalFor, Slot, SlotContent, Text, VueCloak, VueCssBind, VueDirective,
    VueHtml, VueMemo, VueOnce, VueShow, VueSlotScope, VueSync, VueText,
};
use crate::dump::parse::expr_token::take_expr;
use crate::op::Namespace;

/// One classified ops-section line.
pub(in crate::dump) enum Item {
    Attr(Attribute),
    Bind(Bind),
    On(On),
    Model(Model),
    SlotContent(SlotContent),
    Directive(VueDirective),
    CssBind(VueCssBind),
    Sync(VueSync),
    SlotScope(VueSlotScope),
    Once(VueOnce),
    Memo(VueMemo),
    Show(VueShow),
    Html(VueHtml),
    VueText(VueText),
    Cloak(VueCloak),
    Branch(Branch),
    Op(Op),
}

/// Build a [`DumpError`] attributed to `line_no`.
pub(in crate::dump) fn err(line_no: usize, message: String) -> DumpError {
    DumpError::new(line_no, message)
}

fn split_word(text: &str) -> (&str, &str) {
    text.split_once(' ').unwrap_or((text, ""))
}

/// Parse a quoted string starting at `rest[0]`; returns the content and
/// the remainder after the closing quote.
pub(super) fn take_quoted(rest: &str, line_no: usize) -> Result<(String, &str), DumpError> {
    let Some(body) = rest.strip_prefix('"') else {
        return Err(err(line_no, cstr!("expected quoted string")));
    };
    let mut content = String::default();
    let mut chars = body.char_indices();
    while let Some((idx, c)) = chars.next() {
        match c {
            '"' => return Ok((content, body.get(idx + 1..).unwrap_or_default())),
            '\\' => match chars.next() {
                Some((_, 'n')) => content.push('\n'),
                Some((_, 'r')) => content.push('\r'),
                Some((_, 't')) => content.push('\t'),
                Some((_, '"')) => content.push('"'),
                Some((_, '\\')) => content.push('\\'),
                Some((_, other)) => {
                    return Err(err(line_no, cstr!("invalid escape `\\{other}`")));
                }
                None => return Err(err(line_no, cstr!("unterminated quoted string"))),
            },
            other => content.push(other),
        }
    }
    Err(err(line_no, cstr!("unterminated quoted string")))
}

/// Parse `@start:end` making up the whole remainder.
pub(super) fn final_span(rest: &str, line_no: usize) -> Result<Span, DumpError> {
    if rest.is_empty() {
        return Err(err(line_no, cstr!("missing span")));
    }
    let parsed = rest.strip_prefix('@').and_then(|body| {
        let (start, end) = body.split_once(':')?;
        Some(Span::new(start.parse().ok()?, end.parse().ok()?))
    });
    parsed.ok_or_else(|| err(line_no, cstr!("invalid span `{rest}`")))
}

/// Parse the ` @start:end` tail after a completed component.
pub(super) fn tail_span(rest: &str, line_no: usize) -> Result<Span, DumpError> {
    match rest.strip_prefix(' ') {
        Some(tail) => final_span(tail, line_no),
        None if rest.is_empty() => Err(err(line_no, cstr!("missing span"))),
        None => Err(err(line_no, cstr!("trailing content `{rest}`"))),
    }
}

/// Classify and parse one non-blank, dedented ops-section line.
pub(in crate::dump) fn parse_item(content: &str, line_no: usize) -> Result<Item, DumpError> {
    let (keyword, rest) = split_word(content);
    match keyword {
        "attr" => attr(rest, line_no),
        "branch" => branch(rest, line_no),
        "ui.element" => element(rest, line_no),
        "ui.component" => component(rest, line_no),
        "ui.text" => text(rest, line_no),
        "ui.interpolation" => interpolation(rest, line_no),
        "ui.comment" => comment(rest, line_no),
        "ui.if" => Ok(Item::Op(Op::If(If {
            branches: alloc::vec::Vec::new(),
            span: final_span(rest, line_no)?,
        }))),
        "ui.for" => for_op(rest, line_no),
        "ui.slot" => slot(rest, line_no),
        "ui.bind" => crate::dump::parse::binding_line::bind(rest, line_no),
        "ui.on" => crate::dump::parse::binding_line::on(rest, line_no),
        "ui.slot-content" => crate::dump::parse::binding_line::slot_content(rest, line_no),
        "ui.model" => crate::dump::parse::binding_line::model(rest, line_no),
        "vue.directive" => crate::dump::parse::binding_line::directive(rest, line_no),
        "vue.css-bind" => crate::dump::parse::binding_line::css_bind(rest, line_no),
        "vue.sync" => crate::dump::parse::binding_line::sync(rest, line_no),
        "vue.slot-scope" => crate::dump::parse::binding_line::slot_scope(rest, line_no),
        "vue.once" => crate::dump::parse::binding_line::once(rest, line_no),
        "vue.memo" => crate::dump::parse::binding_line::memo(rest, line_no),
        "vue.show" => crate::dump::parse::binding_line::show(rest, line_no),
        "vue.html" => crate::dump::parse::binding_line::html(rest, line_no),
        "vue.text" => crate::dump::parse::binding_line::text(rest, line_no),
        "vue.cloak" => crate::dump::parse::binding_line::cloak(rest, line_no),
        other => Err(err(line_no, cstr!("unknown op `{other}`"))),
    }
}

fn attr(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    let name_end = rest.find(['=', ' ']).unwrap_or(rest.len());
    let (name, after) = rest.split_at(name_end);
    if name.is_empty() {
        return Err(err(line_no, cstr!("missing attribute name")));
    }
    let (value, span) = if let Some(quoted) = after.strip_prefix('=') {
        let (value, tail) = take_quoted(quoted, line_no)?;
        (Some(value), tail_span(tail, line_no)?)
    } else {
        (None, tail_span(after, line_no)?)
    };
    Ok(Item::Attr(Attribute {
        name: String::from(name),
        value,
        span,
    }))
}

fn branch(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    let (condition, span) = if rest.is_empty() || rest.starts_with('@') {
        (None, final_span(rest, line_no)?)
    } else {
        let (condition, tail) = take_expr(rest, line_no)?;
        (Some(condition), tail_span(tail, line_no)?)
    };
    Ok(Item::Branch(Branch {
        condition,
        ops: alloc::vec::Vec::new(),
        span,
    }))
}

fn element(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    let (tag, rest) = split_word(rest);
    if tag.is_empty() {
        return Err(err(line_no, cstr!("missing tag")));
    }
    let (namespace, rest) = if let Some(after) = rest.strip_prefix("ns=") {
        let (name, tail) = split_word(after);
        let namespace = match name {
            "svg" => Namespace::Svg,
            "mathml" => Namespace::MathMl,
            other => return Err(err(line_no, cstr!("invalid namespace `{other}`"))),
        };
        (namespace, tail)
    } else {
        (Namespace::Html, rest)
    };
    Ok(Item::Op(Op::Element(Element {
        tag: String::from(tag),
        namespace,
        attributes: alloc::vec::Vec::new(),
        bindings: alloc::vec::Vec::new(),
        children: alloc::vec::Vec::new(),
        span: final_span(rest, line_no)?,
    })))
}

fn component(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    let (name, rest) = split_word(rest);
    if name.is_empty() {
        return Err(err(line_no, cstr!("missing component name")));
    }
    Ok(Item::Op(Op::Component(Component {
        name: String::from(name),
        attributes: alloc::vec::Vec::new(),
        bindings: alloc::vec::Vec::new(),
        children: alloc::vec::Vec::new(),
        span: final_span(rest, line_no)?,
    })))
}

fn text(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    let (content, tail) = take_quoted(rest, line_no)?;
    Ok(Item::Op(Op::Text(Text {
        content,
        span: tail_span(tail, line_no)?,
    })))
}

fn interpolation(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    let (expression, tail) = take_expr(rest, line_no)?;
    Ok(Item::Op(Op::Interpolation(Interpolation {
        expression,
        span: tail_span(tail, line_no)?,
    })))
}

fn comment(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    let (content, tail) = take_quoted(rest, line_no)?;
    Ok(Item::Op(Op::Comment(Comment {
        content,
        span: tail_span(tail, line_no)?,
    })))
}

fn for_op(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    if let Some(rest) = rest.strip_prefix("original-ref=") {
        let (node, tail) = rest.split_at(rest.find(' ').unwrap_or(rest.len()));
        let node = node
            .parse::<u32>()
            .map_err(|_| err(line_no, cstr!("invalid original For reference `{node}`")))?;
        return Ok(Item::Op(Op::OriginalFor(OriginalFor {
            node,
            ops: alloc::vec::Vec::new(),
            span: tail_span(tail, line_no)?,
        })));
    }
    let Some(rest) = rest.strip_prefix("source=") else {
        return Err(err(line_no, cstr!("expected `source=`")));
    };
    let (source, rest) = take_expr(rest, line_no)?;
    let Some(rest) = rest.strip_prefix(" value=") else {
        return Err(err(line_no, cstr!("expected `value=`")));
    };
    let (value, mut rest) = take_expr(rest, line_no)?;
    let mut key = None;
    let mut index = None;
    if let Some(tail) = rest.strip_prefix(" key=") {
        let (expr, tail) = take_expr(tail, line_no)?;
        key = Some(expr);
        rest = tail;
    }
    if let Some(tail) = rest.strip_prefix(" index=") {
        let (expr, tail) = take_expr(tail, line_no)?;
        index = Some(expr);
        rest = tail;
    }
    Ok(Item::Op(Op::For(For {
        binding: ForBinding {
            source,
            value,
            key,
            index,
        },
        ops: alloc::vec::Vec::new(),
        span: tail_span(rest, line_no)?,
    })))
}

/// Parse a `name=` value: a quoted static name or an expression payload.
pub(super) fn name_value(rest: &str, line_no: usize) -> Result<(Name, &str), DumpError> {
    if rest.starts_with('"') {
        let (name, tail) = take_quoted(rest, line_no)?;
        return Ok((Name::Static(name), tail));
    }
    if ["js(", "opaque(", "foreign(", "vue.filter("]
        .iter()
        .any(|head| rest.starts_with(head))
    {
        let (expr, tail) = take_expr(rest, line_no)?;
        return Ok((Name::Dynamic(expr), tail));
    }
    Err(err(
        line_no,
        cstr!("expected quoted string or an expression payload"),
    ))
}

fn slot(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    let Some(rest) = rest.strip_prefix("name=") else {
        return Err(err(line_no, cstr!("expected `name=`")));
    };
    let (name, tail) = name_value(rest, line_no)?;
    Ok(Item::Op(Op::Slot(Slot {
        name,
        attributes: alloc::vec::Vec::new(),
        bindings: alloc::vec::Vec::new(),
        fallback: alloc::vec::Vec::new(),
        span: tail_span(tail, line_no)?,
    })))
}
