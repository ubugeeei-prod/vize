//! Line grammar of the disegno ops section: one op (or structural) line
//! in, one parsed item out.
//!
//! Expression positions hold the payload tokens of
//! [`expr_token`](super::expr_token) - `js(...)`, `opaque(...)`,
//! `foreign(...)`. Quoted strings escape `\\`, `\"`, `\n`, `\r`,
//! `\t`; values embedding other control characters, attribute names
//! containing `=`, ` ` or `"` are outside the contract (the
//! derived-page "documented edges" rule).

use vize_davinci::folio::FolioError;
use vize_s0::{Span, String, cstr};

use super::super::owned::{
    FolioAttribute, FolioBind, FolioBranch, FolioComponent, FolioElement, FolioFor,
    FolioForBinding, FolioIf, FolioInterpolation, FolioModel, FolioName, FolioOn, FolioOp,
    FolioSlot, FolioSlotContent, FolioText, FolioVueCssBind, FolioVueDirective, FolioVueHtml,
    FolioVueMemo, FolioVueOnce, FolioVueShow, FolioVueSlotScope, FolioVueSync,
};
use super::expr_token::take_expr;
use crate::op::Namespace;

/// One classified ops-section line.
pub(in super::super) enum Item {
    Attr(FolioAttribute),
    Bind(FolioBind),
    On(FolioOn),
    Model(FolioModel),
    SlotContent(FolioSlotContent),
    Directive(FolioVueDirective),
    CssBind(FolioVueCssBind),
    Sync(FolioVueSync),
    SlotScope(FolioVueSlotScope),
    Once(FolioVueOnce),
    Memo(FolioVueMemo),
    Show(FolioVueShow),
    Html(FolioVueHtml),
    Branch(FolioBranch),
    Op(FolioOp),
}

/// Build a [`FolioError`] attributed to `line_no`.
pub(in super::super) fn err(line_no: usize, message: String) -> FolioError {
    FolioError::new(line_no, message)
}

fn split_word(text: &str) -> (&str, &str) {
    text.split_once(' ').unwrap_or((text, ""))
}

/// Parse a quoted string starting at `rest[0]`; returns the content and
/// the remainder after the closing quote.
pub(super) fn take_quoted(rest: &str, line_no: usize) -> Result<(String, &str), FolioError> {
    let Some(body) = rest.strip_prefix('"') else {
        return Err(err(line_no, cstr!("expected quoted string")));
    };
    let mut content = String::default();
    let mut chars = body.char_indices();
    while let Some((idx, c)) = chars.next() {
        match c {
            '"' => return Ok((content, &body[idx + 1..])),
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
pub(super) fn final_span(rest: &str, line_no: usize) -> Result<Span, FolioError> {
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
pub(super) fn tail_span(rest: &str, line_no: usize) -> Result<Span, FolioError> {
    match rest.strip_prefix(' ') {
        Some(tail) => final_span(tail, line_no),
        None if rest.is_empty() => Err(err(line_no, cstr!("missing span"))),
        None => Err(err(line_no, cstr!("trailing content `{rest}`"))),
    }
}

/// Classify and parse one non-blank, dedented ops-section line.
pub(in super::super) fn parse_item(content: &str, line_no: usize) -> Result<Item, FolioError> {
    let (keyword, rest) = split_word(content);
    match keyword {
        "attr" => attr(rest, line_no),
        "branch" => branch(rest, line_no),
        "ui.element" => element(rest, line_no),
        "ui.component" => component(rest, line_no),
        "ui.text" => text(rest, line_no),
        "ui.interpolation" => interpolation(rest, line_no),
        "ui.if" => Ok(Item::Op(FolioOp::If(FolioIf {
            branches: alloc::vec::Vec::new(),
            span: final_span(rest, line_no)?,
        }))),
        "ui.for" => for_op(rest, line_no),
        "ui.slot" => slot(rest, line_no),
        "ui.bind" => super::binding_line::bind(rest, line_no),
        "ui.on" => super::binding_line::on(rest, line_no),
        "ui.slot-content" => super::binding_line::slot_content(rest, line_no),
        "ui.model" => super::binding_line::model(rest, line_no),
        "vue.directive" => super::binding_line::directive(rest, line_no),
        "vue.css-bind" => super::binding_line::css_bind(rest, line_no),
        "vue.sync" => super::binding_line::sync(rest, line_no),
        "vue.slot-scope" => super::binding_line::slot_scope(rest, line_no),
        "vue.once" => super::binding_line::once(rest, line_no),
        "vue.memo" => super::binding_line::memo(rest, line_no),
        "vue.show" => super::binding_line::show(rest, line_no),
        "vue.html" => super::binding_line::html(rest, line_no),
        other => Err(err(line_no, cstr!("unknown op `{other}`"))),
    }
}

fn attr(rest: &str, line_no: usize) -> Result<Item, FolioError> {
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
    Ok(Item::Attr(FolioAttribute {
        name: String::from(name),
        value,
        span,
    }))
}

fn branch(rest: &str, line_no: usize) -> Result<Item, FolioError> {
    let (condition, span) = if rest.is_empty() || rest.starts_with('@') {
        (None, final_span(rest, line_no)?)
    } else {
        let (condition, tail) = take_expr(rest, line_no)?;
        (Some(condition), tail_span(tail, line_no)?)
    };
    Ok(Item::Branch(FolioBranch {
        condition,
        ops: alloc::vec::Vec::new(),
        span,
    }))
}

fn element(rest: &str, line_no: usize) -> Result<Item, FolioError> {
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
    Ok(Item::Op(FolioOp::Element(FolioElement {
        tag: String::from(tag),
        namespace,
        attributes: alloc::vec::Vec::new(),
        bindings: alloc::vec::Vec::new(),
        children: alloc::vec::Vec::new(),
        span: final_span(rest, line_no)?,
    })))
}

fn component(rest: &str, line_no: usize) -> Result<Item, FolioError> {
    let (name, rest) = split_word(rest);
    if name.is_empty() {
        return Err(err(line_no, cstr!("missing component name")));
    }
    Ok(Item::Op(FolioOp::Component(FolioComponent {
        name: String::from(name),
        attributes: alloc::vec::Vec::new(),
        bindings: alloc::vec::Vec::new(),
        children: alloc::vec::Vec::new(),
        span: final_span(rest, line_no)?,
    })))
}

fn text(rest: &str, line_no: usize) -> Result<Item, FolioError> {
    let (content, tail) = take_quoted(rest, line_no)?;
    Ok(Item::Op(FolioOp::Text(FolioText {
        content,
        span: tail_span(tail, line_no)?,
    })))
}

fn interpolation(rest: &str, line_no: usize) -> Result<Item, FolioError> {
    let (expression, tail) = take_expr(rest, line_no)?;
    Ok(Item::Op(FolioOp::Interpolation(FolioInterpolation {
        expression,
        span: tail_span(tail, line_no)?,
    })))
}

fn for_op(rest: &str, line_no: usize) -> Result<Item, FolioError> {
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
    Ok(Item::Op(FolioOp::For(FolioFor {
        binding: FolioForBinding {
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
pub(super) fn name_value(rest: &str, line_no: usize) -> Result<(FolioName, &str), FolioError> {
    if rest.starts_with('"') {
        let (name, tail) = take_quoted(rest, line_no)?;
        return Ok((FolioName::Static(name), tail));
    }
    if ["js(", "opaque(", "foreign(", "vue.filter("]
        .iter()
        .any(|head| rest.starts_with(head))
    {
        let (expr, tail) = take_expr(rest, line_no)?;
        return Ok((FolioName::Dynamic(expr), tail));
    }
    Err(err(
        line_no,
        cstr!("expected quoted string or an expression payload"),
    ))
}

fn slot(rest: &str, line_no: usize) -> Result<Item, FolioError> {
    let Some(rest) = rest.strip_prefix("name=") else {
        return Err(err(line_no, cstr!("expected `name=`")));
    };
    let (name, tail) = name_value(rest, line_no)?;
    Ok(Item::Op(FolioOp::Slot(FolioSlot {
        name,
        attributes: alloc::vec::Vec::new(),
        bindings: alloc::vec::Vec::new(),
        fallback: alloc::vec::Vec::new(),
        span: tail_span(tail, line_no)?,
    })))
}
