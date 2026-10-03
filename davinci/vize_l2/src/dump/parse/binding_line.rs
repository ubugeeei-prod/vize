//! Attached-binding grammar for UI bindings and Vue directives.
//! Split from [`line`](crate::dump::parse::line) along the op-family
//! boundary so region operations and bindings fit the source budget.

use alloc::vec::Vec;

use vize_l0::dump::Error as DumpError;
use vize_l0::{String, cstr};

use crate::dump::owned::{
    Bind, Contract, Expr, Model, Name, On, SlotContent, VueCloak, VueCssBind, VueDirective,
    VueHtml, VueMemo, VueOnce, VueShow, VueSlotScope, VueSync, VueText,
};
use crate::dump::parse::expr_token::take_expr;
use crate::dump::parse::line::{Item, err, final_span, name_value, tail_span, take_quoted};

/// Parse a legacy `"a,b"` or canonical `["a","b"]` modifier payload into owned names.
fn take_mods(rest: &str, line_no: usize) -> Result<(Vec<String>, &str), DumpError> {
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
    name: Option<Name>,
    modifiers: Vec<String>,
    expr: Option<Expr>,
    native_handler: Option<u32>,
    span: vize_l0::Span,
}

fn optional_fields(
    rest: &str,
    expr_key: &str,
    line_no: usize,
) -> Result<OptionalFields, DumpError> {
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
        let (value, tail) = name_value(rest.get(skip..).unwrap_or_default(), line_no)?;
        name = Some(value);
        rest = tail;
        any_field = true;
    }
    let mut modifiers = Vec::new();
    if let Some(skip) = field(rest, "mods=", any_field) {
        let (parsed, tail) = take_mods(rest.get(skip..).unwrap_or_default(), line_no)?;
        modifiers = parsed;
        rest = tail;
        any_field = true;
    }
    let mut expr = None;
    if let Some(skip) = field(rest, expr_key, any_field) {
        let (parsed, tail) = take_expr(rest.get(skip..).unwrap_or_default(), line_no)?;
        expr = Some(parsed);
        rest = tail;
        any_field = true;
    }
    let mut native_handler = None;
    if expr_key == "handler="
        && let Some(skip) = field(rest, "handler-ref=", any_field)
    {
        if expr.is_some() {
            return Err(err(
                line_no,
                cstr!("handler expression and body ref conflict"),
            ));
        }
        let payload = rest.get(skip..).unwrap_or_default();
        let end = payload
            .find(' ')
            .ok_or_else(|| err(line_no, cstr!("expected handler ref and span")))?;
        native_handler = Some(
            payload
                .get(..end)
                .ok_or_else(|| err(line_no, cstr!("invalid handler ref boundary")))?
                .parse()
                .map_err(|_| err(line_no, cstr!("invalid handler ref")))?,
        );
        rest = payload
            .get(end..)
            .ok_or_else(|| err(line_no, cstr!("invalid handler ref boundary")))?;
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
        native_handler,
        span,
    })
}

pub(super) fn slot_content(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    let fields = optional_fields(rest, "params=", line_no)?;
    Ok(Item::SlotContent(SlotContent {
        name: fields.name,
        modifiers: fields.modifiers,
        params: fields.expr,
        span: fields.span,
    }))
}

pub(super) fn bind(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    let fields = optional_fields(rest, "value=", line_no)?;
    Ok(Item::Bind(Bind {
        name: fields.name,
        modifiers: fields.modifiers,
        value: fields.expr,
        span: fields.span,
    }))
}

pub(super) fn on(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    let fields = optional_fields(rest, "handler=", line_no)?;
    Ok(Item::On(On {
        name: fields.name,
        modifiers: fields.modifiers,
        handler: fields.expr,
        native_handler: fields.native_handler,
        span: fields.span,
    }))
}

pub(super) fn model(rest: &str, line_no: usize) -> Result<Item, DumpError> {
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
    Ok(Item::Model(Model {
        contract: Contract { read, write },
        argument,
        attributes: Vec::new(),
        span: tail_span(tail, line_no)?,
    }))
}

pub(super) fn directive(rest: &str, line_no: usize) -> Result<Item, DumpError> {
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
    Ok(Item::Directive(VueDirective {
        name,
        argument,
        modifiers,
        value,
        span: tail_span(rest, line_no)?,
    }))
}

pub(super) fn css_bind(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    let Some(rest) = rest.strip_prefix("value=") else {
        return Err(err(line_no, cstr!("expected `value=`")));
    };
    let (value, rest) = take_expr(rest, line_no)?;
    Ok(Item::CssBind(VueCssBind {
        value,
        span: tail_span(rest, line_no)?,
    }))
}

pub(super) fn sync(rest: &str, line_no: usize) -> Result<Item, DumpError> {
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
    Ok(Item::Sync(VueSync {
        name,
        modifiers,
        value,
        span: tail_span(rest, line_no)?,
    }))
}

pub(super) fn slot_scope(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    let mut rest = rest;
    let mut any_field = false;
    let mut name = None;
    if let Some(value) = rest.strip_prefix("name=") {
        let (value, tail) = take_quoted(value, line_no)?;
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
    Ok(Item::SlotScope(VueSlotScope { name, params, span }))
}

pub(super) fn once(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    Ok(Item::Once(VueOnce {
        span: final_span(rest, line_no)?,
    }))
}

pub(super) fn memo(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    let Some(rest) = rest.strip_prefix("value=") else {
        return Err(err(line_no, cstr!("expected `value=`")));
    };
    let (value, rest) = take_expr(rest, line_no)?;
    Ok(Item::Memo(VueMemo {
        value,
        span: tail_span(rest, line_no)?,
    }))
}

pub(super) fn show(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    let Some(rest) = rest.strip_prefix("value=") else {
        return Err(err(line_no, cstr!("expected `value=`")));
    };
    let (value, rest) = take_expr(rest, line_no)?;
    Ok(Item::Show(VueShow {
        value,
        span: tail_span(rest, line_no)?,
    }))
}

pub(super) fn html(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    let Some(rest) = rest.strip_prefix("value=") else {
        return Ok(Item::Html(VueHtml {
            value: None,
            span: final_span(rest, line_no)?,
        }));
    };
    let (value, rest) = take_expr(rest, line_no)?;
    Ok(Item::Html(VueHtml {
        value: Some(value),
        span: tail_span(rest, line_no)?,
    }))
}

pub(super) fn text(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    let Some(rest) = rest.strip_prefix("value=") else {
        return Ok(Item::VueText(VueText {
            value: None,
            span: final_span(rest, line_no)?,
        }));
    };
    let (value, rest) = take_expr(rest, line_no)?;
    Ok(Item::VueText(VueText {
        value: Some(value),
        span: tail_span(rest, line_no)?,
    }))
}

pub(super) fn cloak(rest: &str, line_no: usize) -> Result<Item, DumpError> {
    Ok(Item::Cloak(VueCloak {
        span: final_span(rest, line_no)?,
    }))
}
