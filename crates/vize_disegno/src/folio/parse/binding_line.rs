//! Attached-binding line grammar: `ui.bind`, `ui.on`, `ui.model`,
//! `ui.slot-content`, `vue.directive`, `vue.css-bind` — split from
//! [`line`](super::line)
//! along the op-family boundary (region-op lines there, binding lines
//! here) so each file stays within the source budget.

use alloc::vec::Vec;

use vize_davinci::folio::FolioError;
use vize_s0::{String, cstr};

use super::super::owned::{
    FolioBind, FolioContract, FolioExpr, FolioModel, FolioName, FolioOn, FolioSlotContent,
    FolioVueCssBind, FolioVueDirective, FolioVueMemo, FolioVueOnce, FolioVueSlotScope,
    FolioVueSync,
};
use super::expr_token::take_expr;
use super::line::{Item, err, final_span, name_value, tail_span, take_quoted};

/// Parse a legacy `"a,b"` or canonical `["a","b"]` modifier payload into owned names.
fn take_mods(rest: &str, line_no: usize) -> Result<(Vec<String>, &str), FolioError> {
    if let Some(mut tail) = rest.strip_prefix('[') {
        let mut modifiers = Vec::new();
        loop {
            let (modifier, after) = take_quoted(tail, line_no)?;
            if modifier.is_empty() {
                return Err(err(line_no, cstr!("invalid modifier list")));
            }
            modifiers.push(modifier);
            if let Some(after) = after.strip_prefix(']') {
                return Ok((modifiers, after));
            }
            let Some(after) = after.strip_prefix(',') else {
                return Err(err(line_no, cstr!("invalid modifier list")));
            };
            tail = after;
        }
    }
    let (joined, tail) = take_quoted(rest, line_no)?;
    let mut modifiers = Vec::new();
    for part in joined.as_str().split(',') {
        if part.is_empty() {
            return Err(err(line_no, cstr!("invalid modifier list")));
        }
        modifiers.push(String::from(part));
    }
    Ok((modifiers, tail))
}

/// The optional-field walker every all-optional binding line shares:
/// the first present field follows the keyword's space (already consumed
/// by `split_word`), each later one carries its own leading space — the
/// same strictness as `vue.directive`'s tail. The walker parses
/// `name=` / `mods=` / one trailing expression field (`params=`,
/// `value=`, `handler=`), then the span.
struct OptionalFields {
    name: Option<FolioName>,
    modifiers: Vec<String>,
    expr: Option<FolioExpr>,
    span: vize_s0::Span,
}

fn optional_fields(
    rest: &str,
    expr_key: &str,
    line_no: usize,
) -> Result<OptionalFields, FolioError> {
    let mut rest = rest;
    let mut any_field = false;
    let field = |rest: &'_ str, key: &str, any_field: bool| -> Option<usize> {
        if any_field {
            rest.strip_prefix(' ')
                .is_some_and(|tail| tail.starts_with(key))
                .then_some(key.len() + 1)
        } else {
            rest.starts_with(key).then_some(key.len())
        }
    };
    let mut name = None;
    if let Some(skip) = field(rest, "name=", any_field) {
        let (value, tail) = name_value(&rest[skip..], line_no)?;
        name = Some(value);
        rest = tail;
        any_field = true;
    }
    let mut modifiers = Vec::new();
    if let Some(skip) = field(rest, "mods=", any_field) {
        let (parsed, tail) = take_mods(&rest[skip..], line_no)?;
        modifiers = parsed;
        rest = tail;
        any_field = true;
    }
    let mut expr = None;
    if let Some(skip) = field(rest, expr_key, any_field) {
        let (parsed, tail) = take_expr(&rest[skip..], line_no)?;
        expr = Some(parsed);
        rest = tail;
        any_field = true;
    }
    let span = if any_field {
        tail_span(rest, line_no)?
    } else {
        final_span(rest, line_no)?
    };
    Ok(OptionalFields {
        name,
        modifiers,
        expr,
        span,
    })
}

pub(super) fn slot_content(rest: &str, line_no: usize) -> Result<Item, FolioError> {
    let fields = optional_fields(rest, "params=", line_no)?;
    Ok(Item::SlotContent(FolioSlotContent {
        name: fields.name,
        modifiers: fields.modifiers,
        params: fields.expr,
        span: fields.span,
    }))
}

pub(super) fn bind(rest: &str, line_no: usize) -> Result<Item, FolioError> {
    let fields = optional_fields(rest, "value=", line_no)?;
    Ok(Item::Bind(FolioBind {
        name: fields.name,
        modifiers: fields.modifiers,
        value: fields.expr,
        span: fields.span,
    }))
}

pub(super) fn on(rest: &str, line_no: usize) -> Result<Item, FolioError> {
    let fields = optional_fields(rest, "handler=", line_no)?;
    Ok(Item::On(FolioOn {
        name: fields.name,
        modifiers: fields.modifiers,
        handler: fields.expr,
        span: fields.span,
    }))
}

pub(super) fn model(rest: &str, line_no: usize) -> Result<Item, FolioError> {
    let mut rest = rest;
    let mut argument = None;
    if let Some(after) = rest.strip_prefix("name=") {
        let (value, tail) = name_value(after, line_no)?;
        argument = Some(value);
        let Some(tail) = tail.strip_prefix(' ') else {
            return Err(err(line_no, cstr!("expected `read=`")));
        };
        rest = tail;
    }
    let Some(rest) = rest.strip_prefix("read=") else {
        return Err(err(line_no, cstr!("expected `read=`")));
    };
    let (read, rest) = take_expr(rest, line_no)?;
    let Some(rest) = rest.strip_prefix(" write=") else {
        return Err(err(line_no, cstr!("expected `write=`")));
    };
    let (write, tail) = take_expr(rest, line_no)?;
    Ok(Item::Model(FolioModel {
        contract: FolioContract { read, write },
        argument,
        attributes: Vec::new(),
        span: tail_span(tail, line_no)?,
    }))
}

pub(super) fn directive(rest: &str, line_no: usize) -> Result<Item, FolioError> {
    let (name, mut rest) = take_quoted(rest, line_no)?;
    let mut argument = None;
    if let Some(after) = rest.strip_prefix(" arg=") {
        let (value, tail) = name_value(after, line_no)?;
        argument = Some(value);
        rest = tail;
    }
    let mut modifiers = Vec::new();
    if let Some(after) = rest.strip_prefix(" mods=") {
        let (parsed, tail) = take_mods(after, line_no)?;
        modifiers = parsed;
        rest = tail;
    }
    let mut value = None;
    if let Some(tail) = rest.strip_prefix(" value=") {
        let (expr, tail) = take_expr(tail, line_no)?;
        value = Some(expr);
        rest = tail;
    }
    Ok(Item::Directive(FolioVueDirective {
        name,
        argument,
        modifiers,
        value,
        span: tail_span(rest, line_no)?,
    }))
}

pub(super) fn css_bind(rest: &str, line_no: usize) -> Result<Item, FolioError> {
    let Some(rest) = rest.strip_prefix("value=") else {
        return Err(err(line_no, cstr!("expected `value=`")));
    };
    let (value, rest) = take_expr(rest, line_no)?;
    Ok(Item::CssBind(FolioVueCssBind {
        value,
        span: tail_span(rest, line_no)?,
    }))
}

pub(super) fn sync(rest: &str, line_no: usize) -> Result<Item, FolioError> {
    let Some(rest) = rest.strip_prefix("name=") else {
        return Err(err(line_no, cstr!("expected `name=`")));
    };
    let (name, rest) = take_quoted(rest, line_no)?;
    let mut rest = rest;
    let mut modifiers = Vec::new();
    if let Some(after) = rest.strip_prefix(" mods=") {
        let (parsed, tail) = take_mods(after, line_no)?;
        modifiers = parsed;
        rest = tail;
    }
    let Some(rest) = rest.strip_prefix(" value=") else {
        return Err(err(line_no, cstr!("expected `value=`")));
    };
    let (value, rest) = take_expr(rest, line_no)?;
    Ok(Item::Sync(FolioVueSync {
        name,
        modifiers,
        value,
        span: tail_span(rest, line_no)?,
    }))
}

pub(super) fn slot_scope(rest: &str, line_no: usize) -> Result<Item, FolioError> {
    let mut rest = rest;
    let mut any_field = false;
    let mut name = None;
    if rest.starts_with("name=") {
        let (value, tail) = take_quoted(&rest["name=".len()..], line_no)?;
        name = Some(value);
        rest = tail;
        any_field = true;
    }
    let mut params = None;
    let params_at = if any_field {
        rest.strip_prefix(" params=")
    } else {
        rest.strip_prefix("params=")
    };
    if let Some(after) = params_at {
        let (expr, tail) = take_expr(after, line_no)?;
        params = Some(expr);
        rest = tail;
        any_field = true;
    }
    let span = if any_field {
        tail_span(rest, line_no)?
    } else {
        final_span(rest, line_no)?
    };
    Ok(Item::SlotScope(FolioVueSlotScope { name, params, span }))
}

pub(super) fn once(rest: &str, line_no: usize) -> Result<Item, FolioError> {
    Ok(Item::Once(FolioVueOnce {
        span: final_span(rest, line_no)?,
    }))
}

pub(super) fn memo(rest: &str, line_no: usize) -> Result<Item, FolioError> {
    let Some(rest) = rest.strip_prefix("value=") else {
        return Err(err(line_no, cstr!("expected `value=`")));
    };
    let (value, rest) = take_expr(rest, line_no)?;
    Ok(Item::Memo(FolioVueMemo {
        value,
        span: tail_span(rest, line_no)?,
    }))
}
