//! A lexical component is one call with one ordered props object. The L2
//! lowering keeps literal attributes and bindings in two source-ordered
//! slices. Merge those slices directly so a spread keeps its precedence.

use vize_l0::{Span, source_frame::SourceRoot};
use vize_l2::expr::ExprRef;
use vize_l2::op::{Attribute, BindOp, BindingOp, ComponentOp, DynamicName};

use super::helpers::{COMPONENT, COMPONENT_SPREAD};
use super::module::{Cx, ModuleProjectionError};
use super::ops::{self, Items};
use super::output::LinkRole;

type Result<T> = core::result::Result<T, ModuleProjectionError>;

pub(super) fn write(cx: &mut Cx<'_>, component: &ComponentOp<'_>, items: &mut Items) -> Result<()> {
    let (tag, bindings) = lexical_owner(cx, component)?;
    items.next(cx);
    cx.out.push(COMPONENT);
    cx.out.push("(");
    cx.verbatim(tag, LinkRole::ComponentTag)?;
    cx.out.push(")(");
    let reserved_reads = prop_object(cx, component, tag, bindings)?;
    cx.out.push(")");
    // Reserved keys do not contribute to the props type. Their expressions
    // still type-check at the original lexical scope.
    if reserved_reads {
        for binding in bindings {
            if let BindingOp::Bind(bind) = binding
                && matches!(bind.name, Some(DynamicName::Static("key" | "ref")))
                && let Some(value) = &bind.value
            {
                items.next(cx);
                cx.out.push("(");
                ops::expression(cx, value)?;
                cx.out.push(")");
            }
        }
    }
    ops::region(cx, &component.children, items)
}

/// Join the original lexical callee with the canonical component selector.
fn lexical_owner<'b, 'a>(
    cx: &Cx<'_>,
    component: &'b ComponentOp<'a>,
) -> Result<(Span, &'b [BindingOp<'a>])> {
    let refusal = ModuleProjectionError::Unsupported("ui.component");
    let Some((BindingOp::Bind(owner), bindings)) = component.bindings.split_first() else {
        return Err(refusal);
    };
    let Some(value) = &owner.value else {
        return Err(refusal);
    };
    let tag = value.span();
    // A lexical tag must be an identifier or a member expression from this
    // opening. A recovered or synthesized expression cannot establish it.
    let ExprRef::Js(js) = value else {
        return Err(ModuleProjectionError::Expression);
    };
    if !matches!(
        js.ast,
        oxc_ast::ast::Expression::Identifier(_)
            | oxc_ast::ast::Expression::StaticMemberExpression(_)
    ) {
        return Err(ModuleProjectionError::Expression);
    }
    let text = js.source;
    if component.name != "component"
        || !matches!(owner.name, Some(DynamicName::Static("is")))
        || !owner.modifiers.is_empty()
        || owner.span != tag
        || cx
            .component_checks
            .and_then(|sites| sites.callee(component.span.start))
            != Some(tag)
        || cx.source.get(tag.start as usize..tag.end as usize) != Some(text)
    {
        return Err(refusal);
    }
    Ok((tag, bindings))
}

/// Canon maps a whole props-literal diagnostic by its width inside the
/// original tag. In particular, `{}` covers the first two bytes of a long
/// tag. The preceding verbatim link always retains the complete tag name.
fn prop_object(
    cx: &mut Cx<'_>,
    component: &ComponentOp<'_>,
    tag: Span,
    bindings: &[BindingOp<'_>],
) -> Result<bool> {
    let start = cx.out.len();
    let link = cx.out.open_anchor(LinkRole::ComponentTag);
    cx.out.push("{");
    let reads = props(cx, component, tag.end, bindings)?;
    cx.out.push("}");
    let width = (cx.out.len() - start).min(tag.len() as usize) as u32;
    let authored = Span::new(tag.start, tag.start + width);
    if !cx.source.is_char_boundary(authored.end as usize) {
        return Err(ModuleProjectionError::Span);
    }
    cx.out.close_anchor(link, authored)?;
    Ok(reads)
}

fn props(
    cx: &mut Cx<'_>,
    component: &ComponentOp<'_>,
    mut cursor: u32,
    mut bindings: &[BindingOp<'_>],
) -> Result<bool> {
    let mut attributes = component.attributes.as_slice();
    let mut entries = Items::new();
    let mut reserved_reads = false;
    loop {
        let bind = match bindings.split_first() {
            Some((BindingOp::Bind(bind), rest)) => Some((&**bind, rest)),
            Some((other, _)) => return Err(ModuleProjectionError::Unsupported(other.mnemonic())),
            None => None,
        };
        if let Some((attr, rest)) = attributes.split_first()
            && bind.is_none_or(|(next, _)| attr.span.start < next.span.start)
        {
            advance(&mut cursor, attr.span, component.span)?;
            attribute(cx, attr, &mut entries)?;
            attributes = rest;
        } else if let Some((bind, rest)) = bind {
            advance(&mut cursor, bind.span, component.span)?;
            reserved_reads |= binding(cx, bind, &mut entries)?;
            bindings = rest;
        } else {
            return Ok(reserved_reads);
        }
    }
}

fn advance(cursor: &mut u32, span: Span, owner: Span) -> Result<()> {
    if span.start < *cursor || span.is_empty() || span.end > owner.end {
        return Err(ModuleProjectionError::Span);
    }
    *cursor = span.end;
    Ok(())
}

fn attribute(cx: &mut Cx<'_>, attr: &Attribute<'_>, entries: &mut Items) -> Result<()> {
    let key = name_span(cx, attr.name, attr.span)?;
    if matches!(attr.name, "key" | "ref") {
        return Ok(());
    }
    entries.next(cx);
    cx.anchor(attr.span, LinkRole::Attribute, |cx| {
        prop_name(cx, attr.name, key)?;
        if let Some(value) = attr.value {
            let authored = cx
                .component_checks
                .and_then(|sites| sites.literal(attr.span))
                .ok_or(ModuleProjectionError::Span)?;
            let quoted = cx
                .source
                .get(authored.start as usize..authored.end as usize)
                .ok_or(ModuleProjectionError::Span)?;
            literal(cx, value, authored, LinkRole::Expression, Some(quoted))?;
        } else if key == attr.span {
            cx.out.push("true");
        } else {
            return Err(ModuleProjectionError::Unsupported("ui.attribute"));
        }
        Ok(())
    })
}

fn binding(cx: &mut Cx<'_>, bind: &BindOp<'_>, entries: &mut Items) -> Result<bool> {
    let Some(value) = &bind.value else {
        return Err(ModuleProjectionError::Unsupported("ui.bind"));
    };
    if !bind.modifiers.is_empty()
        || value.span().start < bind.span.start
        || value.span().end > bind.span.end
    {
        return Err(ModuleProjectionError::Unsupported("ui.bind"));
    }
    let name = match &bind.name {
        Some(DynamicName::Static(name)) => {
            let key = name_span(cx, name, bind.span)?;
            if matches!(*name, "key" | "ref") {
                return Ok(true);
            }
            Some((*name, key))
        }
        Some(DynamicName::Dynamic(_)) => return Err(ModuleProjectionError::Unsupported("ui.bind")),
        None => None,
    };
    entries.next(cx);
    cx.anchor(bind.span, LinkRole::Attribute, |cx| {
        if let Some((name, key)) = name {
            prop_name(cx, name, key)?;
            prop_expression(cx, value)?;
        } else {
            cx.out.push("...");
            cx.out.push(COMPONENT_SPREAD);
            cx.out.push("(");
            prop_expression(cx, value)?;
            cx.out.push(")");
        }
        Ok(())
    })?;
    Ok(false)
}

/// Generated object entries and spread calls both take an AssignmentExpression.
/// An authored sequence needs its original parentheses to be usable here.
fn prop_expression(cx: &mut Cx<'_>, value: &ExprRef<'_>) -> Result<()> {
    match value {
        ExprRef::Js(js) if !matches!(js.ast, oxc_ast::ast::Expression::SequenceExpression(_)) => {
            ops::expression(cx, value)
        }
        _ => Err(ModuleProjectionError::Expression),
    }
}

fn name_span(cx: &Cx<'_>, name: &str, attr: Span) -> Result<Span> {
    let root = SourceRoot::new(cx.source).map_err(|_| ModuleProjectionError::SourceTooLarge)?;
    root.whole_block()
        .span_of(name)
        .filter(|span| span.start == attr.start && !span.is_empty() && span.end <= attr.end)
        .ok_or(ModuleProjectionError::Span)
}

/// Canon's JSX contract accepts Vue's kebab aliases and leaves data/ARIA
/// keys literal. Uppercasing a Unicode scalar can write multiple scalars.
fn prop_name(cx: &mut Cx<'_>, name: &str, span: Span) -> Result<()> {
    let mut canonical = vize_l0::String::default();
    let name = if name.contains('-') && !name.starts_with("data-") && !name.starts_with("aria-") {
        let mut uppercase = false;
        for ch in name.chars() {
            if ch == '-' {
                uppercase = true;
            } else if uppercase {
                canonical.extend(ch.to_uppercase());
                uppercase = false;
            } else {
                canonical.push(ch);
            }
        }
        canonical.as_str()
    } else {
        name
    };
    literal(cx, name, span, LinkRole::Name, None)?;
    cx.out.push(": ");
    Ok(())
}

/// JSON spelling is verbatim between quotes unless there is a control, quote,
/// or backslash. The common property path writes straight into the document.
fn literal(
    cx: &mut Cx<'_>,
    value: &str,
    authored: Span,
    role: LinkRole,
    original: Option<&str>,
) -> Result<()> {
    if value
        .bytes()
        .any(|byte| byte < 0x20 || matches!(byte, b'"' | b'\\'))
    {
        let escaped =
            serde_json::to_string(value).map_err(|_| ModuleProjectionError::Expression)?;
        if original == Some(escaped.as_str()) {
            return cx.verbatim(authored, role);
        }
        cx.out.rewritten(&escaped, authored, role);
        return Ok(());
    }
    if original.and_then(|s| s.strip_prefix('"')?.strip_suffix('"')) == Some(value) {
        return cx.verbatim(authored, role);
    }
    let link = cx.out.open_anchor(role);
    cx.out.push_char('"');
    cx.out.push(value);
    cx.out.push_char('"');
    cx.out.close_anchor(link, authored)
}
