//! Dynamic-name `ui.on` (`@[event]`) emission.

use alloc::vec::Vec as StdVec;

use oxc_ast::ast::{BindingIdentifier, Expression, IdentifierReference};
use oxc_ast_visit::Visit;
use vize_s2::expr::{ExprRef, JsExpr, OpaqueReason};
use vize_s2::op::{DynamicName, OnOp};

use super::buf::Buf;
use super::js::is_valid_js_identifier;
use super::{EmitCx, EmitError, UnsupportedReason as Reason};

pub(super) fn is_dynamic_on_name(on: &OnOp<'_>) -> bool {
    matches!(on.name, Some(DynamicName::Dynamic(_)))
}

pub(super) fn admit(on: &OnOp<'_>) -> Result<(), EmitError> {
    dynamic_name(on)?;
    match on.handler {
        None | Some(ExprRef::Js(_)) => Ok(()),
        Some(ExprRef::Opaque(opaque)) if opaque.reason == OpaqueReason::MultiStatement => Ok(()),
        Some(expr) => Err(EmitError::unsupported_at(
            Reason::OnHandlerNotJs,
            expr.span(),
        )),
    }
}

pub(super) fn emit_pair(cx: &mut EmitCx<'_>, on: &OnOp<'_>) -> Result<(), EmitError> {
    let js = dynamic_name(on)?;
    cx.buf.use_to_handler_key();
    cx.buf.push("[");
    cx.buf.push(Buf::to_handler_key_alias());
    cx.buf.push("(");
    emit_key_source(cx, js);
    cx.buf.push(")]: ");
    emit_value(cx, on)
}

pub(super) fn emit_value(cx: &mut EmitCx<'_>, on: &OnOp<'_>) -> Result<(), EmitError> {
    let classified = super::on::classify_dynamic_modifiers(on.modifiers.iter().copied());
    super::on::emit_wrapped_handler(cx, on, &classified)
}

pub(super) fn forces_inline(on: &OnOp<'_>) -> bool {
    on.modifiers
        .iter()
        .any(|modifier| !matches!(*modifier, "capture" | "once" | "passive"))
}

fn dynamic_name<'a>(on: &'a OnOp<'a>) -> Result<&'a JsExpr<'a>, EmitError> {
    match on.name {
        Some(DynamicName::Dynamic(ExprRef::Js(js))) => Ok(js),
        Some(DynamicName::Dynamic(expr)) => {
            Err(EmitError::unsupported_at(Reason::OnNameNotJs, expr.span()))
        }
        Some(DynamicName::Static(_)) | None => {
            Err(EmitError::unsupported_at(Reason::OnNameNotStatic, on.span))
        }
    }
}

pub(super) fn emit_key_source(cx: &mut EmitCx<'_>, js: &JsExpr<'_>) {
    if matches!(js.ast, Expression::TemplateLiteral(_)) {
        emit_template_literal_key_source(cx, js);
        return;
    }
    let source = js.source;
    if let Some(local) = source.strip_prefix("_ctx.")
        && cx.is_scope_name(local)
    {
        cx.buf.push(local);
        return;
    }
    if cx.is_scope_name(source)
        || source.contains('.')
        || source.starts_with('_')
        || source.starts_with('$')
    {
        cx.buf.push(source);
        return;
    }
    if is_valid_js_identifier(source) {
        cx.buf.push("_ctx.");
    }
    cx.buf.push(source);
}

fn emit_template_literal_key_source(cx: &mut EmitCx<'_>, js: &JsExpr<'_>) {
    let prefixes = {
        let mut visitor = TemplateLiteralKeyPrefixer {
            cx,
            prefixes: StdVec::new(),
            local_vars: StdVec::new(),
        };
        visitor.visit_expression(js.ast);
        visitor.prefixes
    };
    cx.buf.push("(");
    emit_template_literal_prefixes(cx, js.source, prefixes);
    cx.buf.push(")");
}

struct TemplateLiteralKeyPrefixer<'a, 'cx, 'facts> {
    cx: &'cx EmitCx<'facts>,
    prefixes: StdVec<(usize, usize)>,
    local_vars: StdVec<&'a str>,
}

impl<'a, 'cx, 'facts> Visit<'a> for TemplateLiteralKeyPrefixer<'a, 'cx, 'facts> {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        let name = ident.name.as_str();
        if self.cx.is_scope_name(name)
            || self.local_vars.contains(&name)
            || is_global_key_name(name)
        {
            return;
        }
        self.prefixes
            .push((ident.span.start as usize, ident.span.end as usize));
    }

    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        self.local_vars.push(ident.name.as_str());
    }
}

fn emit_template_literal_prefixes(
    cx: &mut EmitCx<'_>,
    source: &str,
    mut prefixes: StdVec<(usize, usize)>,
) {
    if prefixes.is_empty() {
        cx.buf.push(source);
        return;
    }
    prefixes.sort_unstable_by_key(|(start, _)| *start);
    let mut cursor = 0usize;
    for (start, end) in prefixes {
        if start < cursor || end > source.len() {
            cx.buf.push(source);
            return;
        }
        cx.buf.push(&source[cursor..start]);
        cx.buf.push("_ctx.");
        cx.buf.push(&source[start..end]);
        cursor = end;
    }
    cx.buf.push(&source[cursor..]);
}

fn is_global_key_name(name: &str) -> bool {
    matches!(
        name,
        "Infinity"
            | "undefined"
            | "NaN"
            | "Array"
            | "Boolean"
            | "Date"
            | "Error"
            | "Function"
            | "JSON"
            | "Math"
            | "Number"
            | "Object"
            | "Promise"
            | "Proxy"
            | "Reflect"
            | "RegExp"
            | "Set"
            | "String"
            | "Symbol"
            | "Map"
            | "WeakMap"
            | "WeakSet"
            | "BigInt"
            | "parseInt"
            | "parseFloat"
            | "isNaN"
            | "isFinite"
            | "decodeURI"
            | "decodeURIComponent"
            | "encodeURI"
            | "encodeURIComponent"
            | "arguments"
            | "console"
            | "window"
            | "document"
            | "navigator"
            | "globalThis"
            | "require"
            | "import"
            | "exports"
            | "module"
            | "_ctx"
            | "_cache"
            | "_push"
            | "_parent"
            | "$event"
            | "_toNumber"
    )
}
