//! A Solid DOM target over the L3 program.
//!
//! Static structure becomes one cloned `template`. L3 effect scopes decide
//! what stays reactive: child values are inserted before a comment marker,
//! dynamic attributes run in an `effect`, and dynamic component props become
//! getters so Solid tracks them lazily. Element events use Solid's delegation.

use poc_support::{Error, Node, View, escape_html, indent, js, js_string};
use vize_l0::{Span, String, append, cstr, is_void_tag};
use vize_l3::op::OpKind;
use vize_l3::operand::{OperandRole, ValueKind};

#[derive(Default)]
pub(crate) struct Module {
    helpers: Vec<&'static str>,
    templates: Vec<String>,
    events: Vec<String>,
    next_var: usize,
}

/// Reference declarations run on the pristine clone, before any action.
#[derive(Default)]
struct Code {
    declarations: Vec<String>,
    actions: Vec<String>,
}

impl Module {
    /// The expression that replaces one JSX root.
    pub(crate) fn root(&mut self, view: View<'_, '_>) -> Result<String, Error> {
        let mut roots = view.roots();
        match (roots.next(), roots.next()) {
            (Some(root), None) => self.value(root),
            _ => Err(Error::new("expected one root op", Span::new(0, 0))),
        }
    }

    pub(crate) fn preamble(&self) -> String {
        if self.helpers.is_empty() {
            return String::default();
        }
        let names: Vec<String> = self
            .helpers
            .iter()
            .map(|helper| cstr!("{helper} as _${helper}"))
            .collect();
        let mut out = cstr!("import {{ {} }} from \"solid-js/web\";\n", names.join(", "));
        for (index, html) in self.templates.iter().enumerate() {
            let literal = html
                .replace('\\', "\\\\")
                .replace('`', "\\`")
                .replace("${", "\\${");
            append!(
                out,
                "const {} = /*#__PURE__*/ _$template(`{literal}`);\n",
                template_name(index)
            );
        }
        out
    }

    pub(crate) fn epilogue(&self) -> String {
        if self.events.is_empty() {
            return String::default();
        }
        let events: Vec<String> = self.events.iter().map(|event| js_string(event)).collect();
        cstr!("_$delegateEvents([{}]);\n", events.join(", "))
    }

    fn helper(&mut self, name: &'static str) -> String {
        if !self.helpers.contains(&name) {
            self.helpers.push(name);
        }
        cstr!("_${name}")
    }

    fn var(&mut self) -> String {
        self.next_var += 1;
        match self.next_var {
            1 => String::from("_el$"),
            n => cstr!("_el${n}"),
        }
    }

    fn value(&mut self, node: Node<'_, '_>) -> Result<String, Error> {
        match node.kind() {
            OpKind::InsertNode => self.element(node),
            OpKind::CreateComponent => self.component(node),
            OpKind::SetText => static_text(node).map(js_string),
            OpKind::SetChildValue => self.insertion(node),
            _ => Err(unsupported(node)),
        }
    }

    fn element(&mut self, node: Node<'_, '_>) -> Result<String, Error> {
        let index = self.templates.len();
        self.templates.push(String::default());
        self.helper("template");
        let template = template_name(index);
        let mut html = String::default();
        if !touched_below(node) {
            self.markup(
                node,
                String::default(),
                false,
                &mut html,
                &mut Code::default(),
            )?;
            self.set_template(index, html);
            return Ok(cstr!("{template}()"));
        }
        let root = self.var();
        let mut code = Code::default();
        self.markup(node, root.clone(), true, &mut html, &mut code)?;
        self.set_template(index, html);
        let mut out = cstr!("(() => {{\n  const {root} = {template}();\n");
        for line in code.declarations.iter().chain(&code.actions) {
            append!(out, "  {}\n", indent(line, "  "));
        }
        append!(out, "  return {root};\n}})()");
        Ok(out)
    }

    fn set_template(&mut self, index: usize, html: String) {
        if let Some(slot) = self.templates.get_mut(index) {
            *slot = html;
        }
    }

    /// Append `node`'s markup and return the expression later siblings chain
    /// from. `access` reaches the node; a node an action touches gets its own
    /// reference so actions never walk a tree that earlier actions changed.
    fn markup(
        &mut self,
        node: Node<'_, '_>,
        access: String,
        root: bool,
        html: &mut String,
        code: &mut Code,
    ) -> Result<String, Error> {
        let tag = node
            .literal(OperandRole::Tag)
            .ok_or_else(|| unsupported(node))?;
        html.push('<');
        html.push_str(tag);
        for attribute in node.operands(OperandRole::Attribute) {
            html.push(' ');
            html.push_str(attribute.name.unwrap_or_default());
            if attribute.value.kind == ValueKind::Literal {
                append!(*html, "=\"{}\"", escape_html(attribute.value.text));
            }
        }
        html.push('>');
        let base = if touched(node) && !root {
            let var = self.var();
            code.declarations.push(cstr!("const {var} = {access};"));
            var
        } else {
            access
        };
        for binding in node.bindings() {
            self.binding(binding, &base, code)?;
        }
        let mut previous: Option<String> = None;
        for child in node.children() {
            let access = match &previous {
                None => cstr!("{base}.firstChild"),
                Some(previous) => cstr!("{previous}.nextSibling"),
            };
            previous = Some(match child.kind() {
                OpKind::InsertNode => self.markup(child, access, false, html, code)?,
                OpKind::SetText => {
                    html.push_str(&escape_html(static_text(child)?));
                    access
                }
                OpKind::SetChildValue | OpKind::CreateComponent => {
                    html.push_str("<!>");
                    let marker = self.var();
                    code.declarations.push(cstr!("const {marker} = {access};"));
                    let value = self.insertion(child)?;
                    let insert = self.helper("insert");
                    code.actions
                        .push(cstr!("{insert}({base}, {value}, {marker});"));
                    marker
                }
                _ => return Err(unsupported(child)),
            });
        }
        if !is_void_tag(tag) {
            append!(*html, "</{tag}>");
        }
        Ok(base)
    }

    fn insertion(&mut self, node: Node<'_, '_>) -> Result<String, Error> {
        if node.kind() == OpKind::CreateComponent {
            return self.component(node);
        }
        let operand = node
            .operand(OperandRole::ChildValue)
            .ok_or_else(|| unsupported(node))?;
        let value = js(operand)?;
        Ok(if node.is_dynamic() {
            cstr!("() => {value}")
        } else {
            String::from(value)
        })
    }

    fn binding(&mut self, binding: Node<'_, '_>, base: &str, code: &mut Code) -> Result<(), Error> {
        let (name, value) = named_value(binding)?;
        match binding.binding_kind() {
            Some("on") => {
                if self.events.is_empty() {
                    self.helper("delegateEvents");
                }
                if !self.events.iter().any(|event| event == name) {
                    self.events.push(String::from(name));
                }
                code.actions.push(cstr!("{base}.$${name} = {value};"));
            }
            Some("bind") => {
                let set = if name == "class" {
                    cstr!("{}({base}, {value})", self.helper("className"))
                } else {
                    let helper = self.helper("setAttribute");
                    cstr!("{helper}({base}, {}, {value})", js_string(name))
                };
                code.actions.push(if binding.is_dynamic() {
                    cstr!("{}(() => {set});", self.helper("effect"))
                } else {
                    cstr!("{set};")
                });
            }
            _ => return Err(unsupported(binding)),
        }
        Ok(())
    }

    fn component(&mut self, node: Node<'_, '_>) -> Result<String, Error> {
        let name = node
            .literal(OperandRole::Tag)
            .ok_or_else(|| unsupported(node))?;
        let mut props = Vec::new();
        for attribute in node.operands(OperandRole::Attribute) {
            let value = match attribute.value.kind {
                ValueKind::Literal => js_string(attribute.value.text),
                _ => String::from("true"),
            };
            props.push(cstr!("{}: {value}", attribute.name.unwrap_or_default()));
        }
        for binding in node.bindings() {
            if binding.binding_kind() != Some("bind") {
                return Err(unsupported(binding));
            }
            let (key, value) = named_value(binding)?;
            props.push(if binding.is_dynamic() {
                cstr!("get {key}() {{ return {value}; }}")
            } else {
                cstr!("{key}: {value}")
            });
        }
        let children = node
            .children()
            .map(|child| self.value(child))
            .collect::<Result<Vec<_>, _>>()?;
        let children = match children.as_slice() {
            [] => None,
            [only] => Some(only.clone()),
            many => Some(cstr!("[{}]", many.join(", "))),
        };
        if let Some(children) = children {
            props.push(cstr!("get children() {{ return {children}; }}"));
        }
        let create = self.helper("createComponent");
        Ok(match props.as_slice() {
            [] => cstr!("{create}({name}, {{}})"),
            props => cstr!("{create}({name}, {{ {} }})", props.join(", ")),
        })
    }
}

/// Whether an action touches the node itself.
fn touched(node: Node<'_, '_>) -> bool {
    node.bindings().next().is_some()
        || node.children().any(|child| {
            matches!(
                child.kind(),
                OpKind::SetChildValue | OpKind::CreateComponent
            )
        })
}

fn touched_below(node: Node<'_, '_>) -> bool {
    touched(node)
        || node
            .children()
            .any(|child| child.kind() == OpKind::InsertNode && touched_below(child))
}

fn static_text<'a>(node: Node<'_, 'a>) -> Result<&'a str, Error> {
    node.literal(OperandRole::Text)
        .ok_or_else(|| unsupported(node))
}

fn named_value<'a>(binding: Node<'_, 'a>) -> Result<(&'a str, &'a str), Error> {
    let name = binding
        .literal(OperandRole::Name)
        .ok_or_else(|| unsupported(binding))?;
    let value = binding
        .operand(OperandRole::Value)
        .ok_or_else(|| unsupported(binding))?;
    Ok((name, js(value)?))
}

fn template_name(index: usize) -> String {
    match index {
        0 => String::from("_tmpl$"),
        n => cstr!("_tmpl${}", n + 1),
    }
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
