//! `ui.bind` accessors shared by props admission and object emit.

use alloc::vec::Vec as StdVec;

use oxc_ast::ast::{BindingIdentifier, Expression, IdentifierReference};
use oxc_ast_visit::Visit;
use vize_s0::{String, camelize};
use vize_s2::expr::{ExprRef, JsExpr};
use vize_s2::op::{BindOp, DynamicName};

use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::buf::Buf;
use super::js::{is_valid_js_identifier, js_expr_source};
use super::props_value::bind_value;

/// Whether static bind keys should use their ordinary casing or the
/// `<slot>` outlet casing rule.
#[derive(Clone, Copy)]
pub(super) enum StaticBindKeyCasing {
    Preserve,
    Camelize,
}

/// The DOM prop key after static `v-bind` modifiers have been realized.
pub(super) enum StaticBindKey<'a> {
    Borrowed(&'a str),
    Owned(String),
}

impl StaticBindKey<'_> {
    pub(super) fn as_str(&self) -> &str {
        match self {
            Self::Borrowed(text) => text,
            Self::Owned(text) => text.as_str(),
        }
    }
}

pub(super) enum BindName<'a> {
    Static(&'a str),
    Dynamic(&'a JsExpr<'a>),
    Spread,
}

pub(super) fn bind_name<'a>(bind: &'a BindOp<'a>) -> Result<BindName<'a>, EmitError> {
    match bind.name {
        Some(DynamicName::Static(name)) => Ok(BindName::Static(name)),
        Some(DynamicName::Dynamic(ExprRef::Js(js))) => Ok(BindName::Dynamic(js)),
        Some(DynamicName::Dynamic(expr)) => Err(EmitError::unsupported_at(
            Reason::BindNameNotJs,
            expr.span(),
        )),
        None => Ok(BindName::Spread),
    }
}

pub(super) fn static_bind_name<'a>(bind: &'a BindOp<'a>) -> Result<&'a str, EmitError> {
    match bind_name(bind)? {
        BindName::Static(name) => Ok(name),
        BindName::Dynamic(_) | BindName::Spread => Err(EmitError::unsupported_at(
            Reason::BindRequiresStaticName,
            bind.span,
        )),
    }
}

pub(super) fn is_dynamic_bind_name(bind: &BindOp<'_>) -> bool {
    matches!(bind_name(bind), Ok(BindName::Dynamic(_)))
}

pub(super) fn is_key_bind_name(bind: &BindOp<'_>) -> bool {
    match bind.name {
        Some(DynamicName::Static("key")) => true,
        Some(DynamicName::Dynamic(ExprRef::Js(js))) => js.source == "key",
        _ => false,
    }
}

pub(super) fn is_emitted_key_bind(bind: &BindOp<'_>, if_key: Option<&str>) -> bool {
    is_key_bind_name(bind) && js_value(bind).is_ok_and(|js| if_key == Some(js.source))
}

pub(super) fn static_bind_key<'a>(
    bind: &'a BindOp<'a>,
    casing: StaticBindKeyCasing,
) -> Result<StaticBindKey<'a>, EmitError> {
    let raw = static_bind_name(bind)?;
    let mods = StaticBindModifiers::of(bind);
    let mut key = if mods.camel || matches!(casing, StaticBindKeyCasing::Camelize) {
        StaticBindKey::Owned(camelize(raw))
    } else {
        StaticBindKey::Borrowed(raw)
    };
    if mods.prop {
        key = prefixed('.', key);
    } else if mods.attr {
        key = prefixed('^', key);
    }
    Ok(key)
}

pub(super) fn has_prop_modifier(bind: &BindOp<'_>) -> bool {
    StaticBindModifiers::of(bind).prop
}

pub(super) fn emit_dynamic_bind_key(
    cx: &mut EmitCx<'_>,
    bind: &BindOp<'_>,
) -> Result<(), EmitError> {
    let BindName::Dynamic(js) = bind_name(bind)? else {
        return Err(EmitError::unsupported_at(Reason::BindNameNotJs, bind.span));
    };
    let mods = StaticBindModifiers::of(bind);
    cx.buf.push("[");
    if mods.attr {
        cx.buf.push("`^${");
    }
    if mods.prop {
        cx.buf.push("`.${");
    }
    if mods.camel {
        cx.buf.use_camelize();
        cx.buf.push(Buf::camelize_alias());
        cx.buf.push("(");
    }
    emit_dynamic_key_source(cx, js);
    cx.buf.push(" || \"\"");
    if mods.camel {
        cx.buf.push(")");
    }
    if mods.prop {
        cx.buf.push("}`");
    }
    if mods.attr {
        cx.buf.push("}`");
    }
    cx.buf.push("]");
    Ok(())
}

pub(super) fn emit_dynamic_bind_pair(
    cx: &mut EmitCx<'_>,
    bind: &BindOp<'_>,
) -> Result<bool, EmitError> {
    if !is_dynamic_bind_name(bind) {
        return Ok(false);
    }
    let value = bind_value(bind)?;
    emit_dynamic_bind_key(cx, bind)?;
    cx.buf.push(": ");
    value.emit(cx, bind)?;
    Ok(true)
}

fn emit_dynamic_key_source(cx: &mut EmitCx<'_>, js: &JsExpr<'_>) {
    if cx.prefixing() {
        let text = cx.prefixed_dynamic_arg(js);
        cx.buf.push(text.as_str());
        return;
    }
    if matches!(js.ast, Expression::TemplateLiteral(_)) {
        emit_template_literal_key_source(cx, js);
        return;
    }
    let source = js_expr_source(js);
    let original = js.source;
    if is_valid_js_identifier(original) && !cx.is_scope_name(original) {
        cx.buf.push("_ctx.");
    }
    cx.buf.push(source.as_str());
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

pub(super) fn is_global_key_name(name: &str) -> bool {
    super::prefix::is_global_allowed(name)
}

pub(super) fn js_value<'a>(bind: &'a BindOp<'a>) -> Result<&'a JsExpr<'a>, EmitError> {
    match bind.value {
        Some(ExprRef::Js(js)) => Ok(js),
        Some(expr) => Err(EmitError::unsupported_at(
            Reason::BindValueNotJs,
            expr.span(),
        )),
        None => Err(EmitError::unsupported_at(Reason::BindValueNotJs, bind.span)),
    }
}

struct StaticBindModifiers {
    camel: bool,
    prop: bool,
    attr: bool,
}

impl StaticBindModifiers {
    fn of(bind: &BindOp<'_>) -> Self {
        let mut out = Self {
            camel: false,
            prop: false,
            attr: false,
        };
        for modifier in bind.modifiers.iter() {
            match *modifier {
                "camel" => out.camel = true,
                "prop" => out.prop = true,
                "attr" => out.attr = true,
                _ => {}
            }
        }
        out
    }
}

fn prefixed(prefix: char, key: StaticBindKey<'_>) -> StaticBindKey<'_> {
    let key = key.as_str();
    let mut text = String::with_capacity(prefix.len_utf8() + key.len());
    text.push(prefix);
    text.push_str(key);
    StaticBindKey::Owned(text)
}
