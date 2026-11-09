//! The Vue render-function target the script-language examples share.
//!
//! Those examples keep Vue as the framework and change only the language of
//! the script and the template expressions (the Lang axis), so they share this
//! L4 target. Foreign expressions go through the language's own
//! [`ExprDialect::emit`]; the JavaScript it returns is then bound to the
//! component: `ref` bindings read `.value`, props read `__props`.

use vize_l0::{Allocator, Span, String, Vec as ArenaVec, append, cstr, is_void_tag};
use vize_l1::container::ContainerFormat;
use vize_l2::expr::{ExprDialect, ExprRef, ForeignExpr};
use vize_l3::op::OpKind;
use vize_l3::operand::{Operand, OperandRole, ValueKind};

use crate::{Error, Node, View, js_string};

mod bind;
use bind::bind;
pub use bind::{Binding, Bindings};

/// Render functions for one component whose expressions are in `D`'s language.
pub struct Target<'d, D> {
    dialect: &'d D,
    bindings: &'d Bindings,
    helpers: Vec<&'static str>,
}

impl<'d, D: ExprDialect> Target<'d, D> {
    pub fn new(dialect: &'d D, bindings: &'d Bindings) -> Self {
        Self {
            dialect,
            bindings,
            helpers: Vec::new(),
        }
    }

    /// The component module: runtime imports, the script's own imports, and
    /// a `setup` that runs the script and returns the render function.
    pub fn module(
        &mut self,
        view: View<'_, '_>,
        imports: &str,
        script: &str,
    ) -> Result<String, Error> {
        let roots = view.roots().collect::<Vec<_>>();
        let render = match roots.as_slice() {
            [root] => self.vnode(*root, &[])?,
            roots => self.list(roots, &[])?,
        };
        let mut out = String::default();
        let helpers: Vec<String> = self
            .helpers
            .iter()
            .map(|helper| cstr!("{helper} as _{helper}"))
            .collect();
        append!(out, "import {{ {} }} from \"vue\";\n", helpers.join(", "));
        out.push_str(imports);
        out.push_str("\nexport default {\n");
        let props: Vec<String> = self.bindings.props().map(js_string).collect();
        if !props.is_empty() {
            append!(out, "  props: [{}],\n", props.join(", "));
        }
        out.push_str("  setup(__props) {\n");
        for line in script.lines() {
            if line.is_empty() {
                out.push('\n');
            } else {
                append!(out, "    {line}\n");
            }
        }
        append!(
            out,
            "    return () => {};\n  }},\n}};\n",
            crate::indent(&render, "    ")
        );
        Ok(out)
    }

    fn helper(&mut self, name: &'static str) -> String {
        if !self.helpers.contains(&name) {
            self.helpers.push(name);
        }
        cstr!("_{name}")
    }

    fn list(&mut self, nodes: &[Node<'_, '_>], locals: &[&str]) -> Result<String, Error> {
        let items = nodes
            .iter()
            .map(|node| self.vnode(*node, locals))
            .collect::<Result<Vec<_>, _>>()?;
        if let [only] = items.as_slice() {
            return Ok(cstr!("[{only}]"));
        }
        let mut out = String::from("[");
        for item in &items {
            append!(out, "\n  {},", crate::indent(item, "  "));
        }
        out.push_str("\n]");
        Ok(out)
    }

    fn vnode(&mut self, node: Node<'_, '_>, locals: &[&str]) -> Result<String, Error> {
        match node.kind() {
            OpKind::InsertNode => self.element(node, locals),
            OpKind::SetText => self.text(node, locals),
            OpKind::If => self.conditional(node, locals),
            OpKind::For => {
                let source = self.expression(operand(node, OperandRole::ForSource)?, locals)?;
                let item = operand(node, OperandRole::ForValue)?.value.text;
                let mut inner = locals.to_vec();
                inner.push(item);
                let body = self.only(node.children().collect(), &inner)?;
                let fragment = self.helper("Fragment");
                let h = self.helper("h");
                let list = self.helper("renderList");
                Ok(cstr!(
                    "{h}({fragment}, null, {list}({source}, ({item}) => {body}))"
                ))
            }
            _ => Err(unsupported(node)),
        }
    }

    fn only(&mut self, nodes: Vec<Node<'_, '_>>, locals: &[&str]) -> Result<String, Error> {
        match nodes.as_slice() {
            [only] => self.vnode(*only, locals),
            nodes => self.list(nodes, locals),
        }
    }

    fn element(&mut self, node: Node<'_, '_>, locals: &[&str]) -> Result<String, Error> {
        let tag = node
            .literal(OperandRole::Tag)
            .ok_or_else(|| unsupported(node))?;
        let mut props = Vec::new();
        for attribute in node.operands(OperandRole::Attribute) {
            let value = match attribute.value.kind {
                ValueKind::Literal => js_string(attribute.value.text),
                _ => String::from("\"\""),
            };
            props.push(cstr!(
                "{}: {value}",
                key(attribute.name.unwrap_or_default())
            ));
        }
        let mut model = None;
        for binding in node.bindings() {
            match binding.binding_kind() {
                Some("bind") => {
                    let name = binding
                        .literal(OperandRole::Name)
                        .ok_or_else(|| unsupported(binding))?;
                    let value = self.expression(operand(binding, OperandRole::Value)?, locals)?;
                    props.push(cstr!("{}: {value}", key(name)));
                }
                Some("on") => {
                    let name = binding
                        .literal(OperandRole::Name)
                        .ok_or_else(|| unsupported(binding))?;
                    let value = self.expression(operand(binding, OperandRole::Value)?, locals)?;
                    let handler = if is_member_path(&value) {
                        value
                    } else {
                        cstr!("($event) => ({value})")
                    };
                    let mut event = String::from("on");
                    let mut chars = name.chars();
                    event.extend(chars.next().map(|first| first.to_ascii_uppercase()));
                    event.push_str(chars.as_str());
                    props.push(cstr!("{event}: {handler}"));
                }
                Some("model") if tag == "input" => {
                    let read =
                        self.expression(operand(binding, OperandRole::ModelRead)?, locals)?;
                    let write =
                        self.expression(operand(binding, OperandRole::ModelWrite)?, locals)?;
                    props.push(cstr!(
                        "\"onUpdate:modelValue\": ($event) => ({write} = $event)"
                    ));
                    model = Some(read);
                }
                _ => return Err(unsupported(binding)),
            }
        }
        let h = self.helper("h");
        let props = if props.is_empty() {
            String::from("null")
        } else {
            cstr!("{{ {} }}", props.join(", "))
        };
        let children: Vec<Node<'_, '_>> = node.children().collect();
        let mut vnode = match children.as_slice() {
            [] if is_void_tag(tag) => cstr!("{h}({}, {props})", js_string(tag)),
            [text] if text.kind() == OpKind::SetText => {
                cstr!(
                    "{h}({}, {props}, {})",
                    js_string(tag),
                    self.text(*text, locals)?
                )
            }
            children => cstr!(
                "{h}({}, {props}, {})",
                js_string(tag),
                self.list(children, locals)?
            ),
        };
        if let Some(read) = model {
            let directives = self.helper("withDirectives");
            let text = self.helper("vModelText");
            vnode = cstr!("{directives}({vnode}, [[{text}, {read}]])");
        }
        Ok(vnode)
    }

    /// Static text, one value, or a compound run split on Vue's delimiters.
    fn text(&mut self, node: Node<'_, '_>, locals: &[&str]) -> Result<String, Error> {
        let value = operand(node, OperandRole::Text)?;
        if value.value.kind == ValueKind::Literal {
            return Ok(js_string(value.value.text));
        }
        if value.value.kind != ValueKind::Opaque {
            let display = self.helper("toDisplayString");
            return Ok(cstr!("{display}({})", self.expression(value, locals)?));
        }
        let mut parts = Vec::new();
        let mut rest = value.value.text;
        let mut offset = value.value.span.start;
        while let Some((before, after)) = rest.split_once("{{") {
            if !before.is_empty() {
                parts.push(js_string(before));
            }
            let (inner, tail) = after
                .split_once("}}")
                .ok_or_else(|| Error::new("unclosed interpolation", value.value.span))?;
            let start = offset + u32::try_from(rest.len() - after.len()).unwrap_or(0);
            let source = inner.trim();
            let leading = u32::try_from(inner.len() - inner.trim_start().len()).unwrap_or(0);
            let span = Span::new(
                start + leading,
                start + leading + u32::try_from(source.len()).unwrap_or(0),
            );
            let js = self.foreign(source, value.value.qualifier, span, locals)?;
            let display = self.helper("toDisplayString");
            parts.push(cstr!("{display}({js})"));
            offset = start + u32::try_from(after.len() - tail.len()).unwrap_or(0);
            rest = tail;
        }
        if !rest.is_empty() {
            parts.push(js_string(rest));
        }
        Ok(String::from(parts.join(" + ")))
    }

    fn conditional(&mut self, node: Node<'_, '_>, locals: &[&str]) -> Result<String, Error> {
        let mut out = String::default();
        let mut closed = false;
        for region in node.regions().collect::<Vec<_>>() {
            let body = self.only(node.region(region).collect(), locals)?;
            let condition = node
                .operands(OperandRole::Condition)
                .find(|operand| operand.region == Some(region))
                .ok_or_else(|| unsupported(node))?;
            if condition.value.kind == ValueKind::Absent {
                out.push_str(&body);
                closed = true;
                break;
            }
            append!(out, "{} ? {body} : ", self.expression(condition, locals)?);
        }
        if !closed {
            let comment = self.helper("createCommentVNode");
            append!(out, "{comment}(\"v-if\", true)");
        }
        Ok(out)
    }

    fn expression(&self, operand: &Operand<'_>, locals: &[&str]) -> Result<String, Error> {
        match operand.value.kind {
            ValueKind::Foreign => self.foreign(
                operand.value.text,
                operand.value.qualifier,
                operand.value.span,
                locals,
            ),
            ValueKind::Js => bind(
                operand.value.text,
                operand.value.span,
                self.bindings,
                locals,
            ),
            kind => Err(Error::new(
                cstr!("unsupported {} value", kind.as_str()),
                operand.value.span,
            )),
        }
    }

    /// Translate through the language's own capability, then bind the result.
    fn foreign(
        &self,
        source: &str,
        dialect: &str,
        span: Span,
        locals: &[&str],
    ) -> Result<String, Error> {
        let allocator = Allocator::default();
        let arena = &allocator;
        let foreign = ForeignExpr {
            dialect,
            source,
            span,
            facts: ArenaVec::new_in(&arena),
        };
        let mut js = String::default();
        self.dialect
            .emit(ExprRef::Foreign(&foreign), &mut js)
            .map_err(|_| Error::new(cstr!("{dialect} refused this expression"), span))?;
        bind(&js, span, self.bindings, locals)
    }
}

/// A script translator: the block's text and file offset to its imports, its
/// JavaScript body and its bindings.
pub type Script = fn(&str, u32) -> Result<(String, String, Bindings), Error>;

/// Compile a Vue SFC whose `<script setup lang>` and template expressions are
/// in `D`'s language: the L1 Vue container and markup, the L1 to L2 foreign
/// expression hook, the shared L2 to L3 lowering, then this target.
pub fn compile<D: ExprDialect + Default>(
    source: &str,
    lang: &str,
    script: Script,
) -> Result<String, Error> {
    let allocator = Allocator::default();
    let file = vize_l1::container::Vue.split(&allocator, source);
    if let Some(error) = file.errors.first() {
        return Err(Error::new(
            cstr!("{:?}", error.code),
            Span::new(error.offset, error.offset),
        ));
    }
    let block = |name: &str| file.blocks.iter().find(|block| block.name == name);
    let text = |span: Span| {
        source
            .get(span.start as usize..span.end as usize)
            .unwrap_or_default()
    };
    let (Some(script_block), Some(template)) = (block("script"), block("template")) else {
        return Err(Error::new(
            "expected a <script setup> and a <template>",
            Span::new(0, 0),
        ));
    };
    if script_block.attr("lang").and_then(|attr| attr.value) != Some(lang)
        || script_block.attr("setup").is_none()
    {
        return Err(Error::new(
            cstr!("expected <script setup lang=\"{lang}\">"),
            script_block.open_tag,
        ));
    }
    let (imports, body, bindings) = script(text(script_block.content), script_block.content.start)?;
    let markup = text(template.content);
    let root = vize_l0::SourceRoot::new(source)
        .map_err(|_| Error::new("source too large", Span::new(0, 0)))?;
    let frame = root
        .block(markup, template.content.start)
        .map_err(|_| Error::new("template out of bounds", template.content))?;
    let (tree, errors) = vize_l1::markup::parse(&allocator, markup);
    if let Some(error) = errors.first() {
        let at = template.content.start + error.offset;
        return Err(Error::new(
            cstr!("markup: {:?}", error.code),
            Span::new(at, at),
        ));
    }
    let hook = vize_l1_to_l2::ForeignDialect::new::<D>(lang)
        .ok_or_else(|| Error::new("a language id is a lowercase word", script_block.open_tag))?;
    let caps = vize_l1::dialect::LegacyCaps::VUE3;
    let lowered = vize_l1_to_l2::lower_source_block_with_foreign_expressions(
        &allocator, &tree, &errors, frame, caps, hook,
    );
    if let Some(diagnostic) = lowered.diagnostics.first() {
        return Err(Error::new(cstr!("{diagnostic:?}"), template.content));
    }
    let program = crate::lower(&allocator, &lowered.artifact.root)?;
    Target::new(&D::default(), &bindings).module(View::new(&program.program), &imports, &body)
}

fn is_member_path(js: &str) -> bool {
    !js.is_empty()
        && js.split('.').all(|part| {
            part.starts_with(|ch: char| ch.is_ascii_alphabetic() || ch == '_' || ch == '$')
                && part
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
        })
}

fn key(name: &str) -> String {
    if is_member_path(name) && !name.contains('.') {
        String::from(name)
    } else {
        js_string(name)
    }
}

fn operand<'p, 'a>(node: Node<'p, 'a>, role: OperandRole) -> Result<&'p Operand<'a>, Error> {
    node.operand(role).ok_or_else(|| unsupported(node))
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
