//! A Svelte 5 client target over the L3 program.
//!
//! Static markup becomes one `$.from_html` template per block. Nodes that an
//! action touches get a reference through `$.child` / `$.sibling`. L3 effect
//! scopes become one `$.template_effect` per block; a text run with dynamic
//! parts is one text node set from a template literal. `ui.if` and `ui.for`
//! become `$.if` and `$.each` blocks, `ui.model` becomes `$.bind_value`, and
//! element events use Svelte's delegation.

use poc_support::{Error, Node, View, escape_html, indent, js, js_string};
use vize_l0::{Span, String, append, cstr, is_void_tag};
use vize_l3::op::OpKind;
use vize_l3::operand::{OperandRole, ValueKind};

use crate::runes::{self, Runes};

pub(crate) struct Module<'r> {
    runes: &'r Runes,
    pub(crate) templates: Vec<String>,
    pub(crate) events: Vec<String>,
    names: Vec<(String, usize)>,
}

/// One block's statements; effects run together in its `$.template_effect`.
#[derive(Default)]
struct Code {
    lines: Vec<String>,
    effects: Vec<String>,
}

impl<'r> Module<'r> {
    pub(crate) fn new(runes: &'r Runes) -> Self {
        Self {
            runes,
            templates: Vec::new(),
            events: Vec::new(),
            names: Vec::new(),
        }
    }

    /// Statements that build the markup and append it to `$$anchor`.
    pub(crate) fn mount(&mut self, view: View<'_, '_>) -> Result<Vec<String>, Error> {
        self.block(view.roots().collect(), &[])
    }

    /// A fresh local name; Svelte's own `text`, `text_1`, ... scheme.
    fn name(&mut self, base: &str) -> String {
        loop {
            let count = match self.names.iter_mut().find(|(name, _)| name == base) {
                Some((_, count)) => {
                    *count += 1;
                    *count
                }
                None => {
                    self.names.push((String::from(base), 0));
                    0
                }
            };
            let name = match count {
                0 => String::from(base),
                n => cstr!("{base}_{n}"),
            };
            if !self.runes.declares(&name) {
                return name;
            }
        }
    }

    fn block(&mut self, nodes: Vec<Node<'_, '_>>, signals: &[&str]) -> Result<Vec<String>, Error> {
        let [node] = nodes.as_slice() else {
            let span = nodes.first().map_or(Span::new(0, 0), |node| node.span());
            return Err(Error::new("a block renders exactly one element", span));
        };
        if node.kind() != OpKind::InsertNode {
            return Err(unsupported(*node));
        }
        let index = self.templates.len();
        self.templates.push(String::default());
        let template = match index {
            0 => String::from("root"),
            n => cstr!("root_{n}"),
        };
        let var = self.name(node.literal(OperandRole::Tag).unwrap_or("node"));
        let mut code = Code::default();
        code.lines.push(cstr!("var {var} = {template}();"));
        let mut html = String::default();
        self.markup(*node, &var, &mut html, &mut code, signals)?;
        if let Some(slot) = self.templates.get_mut(index) {
            *slot = html;
        }
        let mut lines = code.lines;
        match code.effects.as_slice() {
            [] => {}
            [effect] => lines.push(cstr!("$.template_effect(() => {effect});")),
            effects => {
                let mut block = String::from("$.template_effect(() => {");
                for effect in effects {
                    append!(block, "\n  {};", indent(effect, "  "));
                }
                block.push_str("\n});");
                lines.push(block);
            }
        }
        lines.push(cstr!("$.append($$anchor, {var});"));
        Ok(lines)
    }

    fn markup(
        &mut self,
        node: Node<'_, '_>,
        var: &str,
        html: &mut String,
        code: &mut Code,
        signals: &[&str],
    ) -> Result<(), Error> {
        let tag = node
            .literal(OperandRole::Tag)
            .ok_or_else(|| unsupported(node))?;
        append!(*html, "<{tag}");
        for attribute in node.operands(OperandRole::Attribute) {
            append!(*html, " {}", attribute.name.unwrap_or_default());
            if attribute.value.kind == ValueKind::Literal {
                append!(*html, "=\"{}\"", escape_html(attribute.value.text));
            }
        }
        html.push('>');
        for binding in node.bindings() {
            self.binding(binding, var, code, signals)?;
        }
        let children: Vec<Node<'_, '_>> = node.children().collect();
        let mut anchor: Option<String> = None;
        let mut skipped = 0usize;
        let mut rest = children.as_slice();
        while let Some((child, tail)) = rest.split_first() {
            let mut next = tail;
            match child.kind() {
                OpKind::SetText => {
                    let run = rest
                        .iter()
                        .take_while(|node| node.kind() == OpKind::SetText)
                        .count();
                    let (texts, after) = rest.split_at_checked(run).unwrap_or((rest, &[]));
                    next = after;
                    if texts
                        .iter()
                        .all(|text| text.literal(OperandRole::Text).is_some())
                    {
                        for text in texts {
                            html.push_str(&escape_html(
                                text.literal(OperandRole::Text).unwrap_or_default(),
                            ));
                        }
                        skipped += 1;
                    } else {
                        html.push(' ');
                        let text = self.name("text");
                        code.lines.push(cstr!(
                            "var {text} = {};",
                            nav(var, anchor.as_deref(), skipped)
                        ));
                        let value = self.text_run(texts, signals)?;
                        code.effects.push(cstr!("$.set_text({text}, {value})"));
                        anchor = Some(text);
                        skipped = 0;
                    }
                }
                OpKind::InsertNode if touched_below(*child) => {
                    let element = self.name(child.literal(OperandRole::Tag).unwrap_or("node"));
                    code.lines.push(cstr!(
                        "var {element} = {};",
                        nav(var, anchor.as_deref(), skipped)
                    ));
                    self.markup(*child, &element, html, code, signals)?;
                    anchor = Some(element);
                    skipped = 0;
                }
                OpKind::InsertNode => {
                    self.markup(*child, "", html, code, signals)?;
                    skipped += 1;
                }
                OpKind::If | OpKind::For => {
                    html.push_str("<!>");
                    let at = self.name("node");
                    code.lines.push(cstr!(
                        "var {at} = {};",
                        nav(var, anchor.as_deref(), skipped)
                    ));
                    let block = if child.kind() == OpKind::If {
                        self.if_block(*child, &at, signals)?
                    } else {
                        self.each_block(*child, &at, signals)?
                    };
                    code.lines.extend(block);
                    anchor = Some(at);
                    skipped = 0;
                }
                _ => return Err(unsupported(*child)),
            }
            rest = next;
        }
        if !is_void_tag(tag) {
            append!(*html, "</{tag}>");
        }
        Ok(())
    }

    fn binding(
        &mut self,
        binding: Node<'_, '_>,
        var: &str,
        code: &mut Code,
        signals: &[&str],
    ) -> Result<(), Error> {
        let name = binding
            .literal(OperandRole::Name)
            .ok_or_else(|| unsupported(binding))?;
        match binding.binding_kind() {
            Some("on") => {
                let handler = self.expression(binding, OperandRole::Value, signals)?;
                if !self.events.iter().any(|event| event == name) {
                    self.events.push(String::from(name));
                }
                code.lines.push(cstr!("{var}.__{name} = {handler};"));
            }
            Some("bind") => {
                let value = self.expression(binding, OperandRole::Value, signals)?;
                let set = cstr!("$.set_attribute({var}, {}, {value})", js_string(name));
                if binding.is_dynamic() {
                    code.effects.push(set);
                } else {
                    code.lines.push(cstr!("{set};"));
                }
            }
            Some("model") if name == "value" => {
                let read = self.expression(binding, OperandRole::ModelRead, signals)?;
                let target = binding
                    .operand(OperandRole::ModelWrite)
                    .ok_or_else(|| unsupported(binding))?;
                let assignment = cstr!("{} = $$value", js(target)?);
                let write = runes::expression(&assignment, target.value.span, self.runes, signals)?;
                code.lines.push(cstr!(
                    "$.bind_value({var}, () => {read}, ($$value) => {write});"
                ));
            }
            _ => return Err(unsupported(binding)),
        }
        Ok(())
    }

    /// One text node's value: a template literal over its static and dynamic parts.
    fn text_run(&self, texts: &[Node<'_, '_>], signals: &[&str]) -> Result<String, Error> {
        let mut value = String::from("`");
        for text in texts {
            match text.literal(OperandRole::Text) {
                Some(literal) => {
                    let escaped = literal
                        .replace('\\', "\\\\")
                        .replace('`', "\\`")
                        .replace("${", "\\${");
                    value.push_str(&escaped);
                }
                None => {
                    let expression = self.expression(*text, OperandRole::Text, signals)?;
                    append!(value, "${{{expression} ?? \"\"}}");
                }
            }
        }
        value.push('`');
        Ok(value)
    }

    fn if_block(
        &mut self,
        node: Node<'_, '_>,
        at: &str,
        signals: &[&str],
    ) -> Result<Vec<String>, Error> {
        let regions: Vec<_> = node.regions().collect();
        let condition = |region| {
            node.operands(OperandRole::Condition)
                .find(|operand| operand.region == Some(region))
        };
        let (consequent, alternate) = match regions.as_slice() {
            [then] => (*then, None),
            [then, otherwise]
                if condition(*otherwise).is_some_and(|c| c.value.kind == ValueKind::Absent) =>
            {
                (*then, Some(*otherwise))
            }
            _ => {
                return Err(Error::new(
                    "only {#if} with an optional {:else}",
                    node.span(),
                ));
            }
        };
        let test = condition(consequent).ok_or_else(|| unsupported(node))?;
        let test = runes::expression(js(test)?, test.value.span, self.runes, signals)?;
        let mut lines = Vec::new();
        let then = self.name("consequent");
        let body = self.block(node.region(consequent).collect(), signals)?;
        lines.push(render_fn(&then, "($$anchor)", &body));
        let mut decide = cstr!("if ({test}) $$render({then});");
        if let Some(otherwise) = alternate {
            let name = self.name("alternate");
            let body = self.block(node.region(otherwise).collect(), signals)?;
            lines.push(render_fn(&name, "($$anchor)", &body));
            append!(decide, " else $$render({name}, false);");
        }
        lines.push(cstr!("$.if({at}, ($$render) => {{\n  {decide}\n}});"));
        Ok(lines)
    }

    fn each_block(
        &mut self,
        node: Node<'_, '_>,
        at: &str,
        signals: &[&str],
    ) -> Result<Vec<String>, Error> {
        let source = node
            .operand(OperandRole::ForSource)
            .ok_or_else(|| unsupported(node))?;
        let source = runes::expression(js(source)?, source.value.span, self.runes, signals)?;
        let item = node
            .operand(OperandRole::ForValue)
            .ok_or_else(|| unsupported(node))
            .and_then(|operand| js(operand))?;
        let mut inner: Vec<&str> = signals.to_vec();
        inner.push(item);
        let body = self.block(node.children().collect(), &inner)?;
        let mut text = cstr!("$.each({at}, 1, () => {source}, $.index, ($$anchor, {item}) => {{");
        for line in &body {
            append!(text, "\n  {}", indent(line, "  "));
        }
        text.push_str("\n});");
        Ok(vec![text])
    }

    fn expression(
        &self,
        node: Node<'_, '_>,
        role: OperandRole,
        signals: &[&str],
    ) -> Result<String, Error> {
        let operand = node.operand(role).ok_or_else(|| unsupported(node))?;
        runes::expression(js(operand)?, operand.value.span, self.runes, signals)
    }

    pub(crate) fn delegate(&self) -> Option<String> {
        if self.events.is_empty() {
            return None;
        }
        let events: Vec<String> = self.events.iter().map(|event| js_string(event)).collect();
        Some(cstr!("$.delegate([{}]);", events.join(", ")))
    }
}

fn render_fn(name: &str, params: &str, body: &[String]) -> String {
    let mut text = cstr!("var {name} = {params} => {{");
    for line in body {
        append!(text, "\n  {}", indent(line, "  "));
    }
    text.push_str("\n};");
    text
}

/// The DOM walk from `parent` past `anchor` (the last referenced child) and
/// `skipped` untouched nodes.
fn nav(parent: &str, anchor: Option<&str>, skipped: usize) -> String {
    match (anchor, skipped) {
        (None, 0) => cstr!("$.child({parent})"),
        (None, n) => cstr!("$.sibling($.child({parent}), {n})"),
        (Some(anchor), 0) => cstr!("$.sibling({anchor})"),
        (Some(anchor), n) => cstr!("$.sibling({anchor}, {})", n + 1),
    }
}

fn touched(node: Node<'_, '_>) -> bool {
    node.bindings().next().is_some()
        || node.children().any(|child| match child.kind() {
            OpKind::SetText => child.literal(OperandRole::Text).is_none(),
            OpKind::If | OpKind::For => true,
            _ => false,
        })
}

fn touched_below(node: Node<'_, '_>) -> bool {
    touched(node)
        || node
            .children()
            .any(|child| child.kind() == OpKind::InsertNode && touched_below(child))
}

fn unsupported(node: Node<'_, '_>) -> Error {
    Error::new(
        cstr!(
            "{} is outside this proof of concept",
            node.kind().mnemonic()
        ),
        node.span(),
    )
}
