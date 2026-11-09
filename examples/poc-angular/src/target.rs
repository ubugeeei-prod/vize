//! An Angular Ivy target over the L3 program.
//!
//! Every view is one template function with Ivy's two passes. Ops L3 keeps
//! static go only to the creation pass (`rf & 1`): elements, static text,
//! listeners and embedded-view slots. Ops with an L3 effect scope go to the
//! update pass (`rf & 2`) at their slot: text interpolation, property and
//! class bindings, `ɵɵconditional` for `ui.if` and `ɵɵrepeater` for `ui.for`.
//! Template expressions read the component through the view context.

use oxc_ast::ast::{BindingIdentifier, IdentifierReference, ObjectProperty};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use poc_support::{Error, Node, View, indent, js, js_string};
use vize_l0::{Allocator, Span, String, append, cstr};
use vize_l3::op::OpKind;
use vize_l3::operand::{Operand, OperandRole, ValueKind};

pub(crate) struct Component<'s> {
    class: &'s str,
    /// Embedded view template functions, innermost first.
    pub(crate) functions: Vec<String>,
    pub(crate) consts: Vec<String>,
    contexts: usize,
}

/// One view's passes, slots and binding count.
struct ViewCode {
    base: String,
    create: Vec<String>,
    update: Vec<(u32, String)>,
    slots: u32,
    vars: u32,
    /// The component reference: `ctx` in the root view.
    component: String,
    parent_used: bool,
    /// The repeated item, read from `ctx.$implicit` in a `@for` view.
    item: Option<(String, String)>,
    item_used: bool,
}

impl ViewCode {
    fn new(base: String, component: String, item: Option<(String, String)>) -> Self {
        Self {
            base,
            create: Vec::new(),
            update: Vec::new(),
            slots: 0,
            vars: 0,
            component,
            parent_used: false,
            item,
            item_used: false,
        }
    }

    fn slot(&mut self) -> u32 {
        self.slots += 1;
        self.slots - 1
    }
}

impl<'s> Component<'s> {
    pub(crate) fn new(class: &'s str) -> Self {
        Self {
            class,
            functions: Vec::new(),
            consts: Vec::new(),
            contexts: 0,
        }
    }

    /// The root template function plus its `decls` and `vars`.
    pub(crate) fn root(&mut self, view: View<'_, '_>) -> Result<(String, u32, u32), Error> {
        let mut code = ViewCode::new(String::from(self.class), String::from("ctx"), None);
        for node in view.roots() {
            self.node(node, &mut code)?;
        }
        let name = cstr!("{}_Template", self.class);
        Ok((function(&name, &code), code.slots, code.vars))
    }

    fn node(&mut self, node: Node<'_, '_>, code: &mut ViewCode) -> Result<(), Error> {
        match node.kind() {
            OpKind::InsertNode => self.element(node, code),
            OpKind::If => self.conditional(node, code),
            OpKind::For => self.repeater(node, code),
            _ => Err(unsupported(node)),
        }
    }

    fn element(&mut self, node: Node<'_, '_>, code: &mut ViewCode) -> Result<(), Error> {
        let tag = node
            .literal(OperandRole::Tag)
            .ok_or_else(|| unsupported(node))?;
        let slot = code.slot();
        let consts = self.consts_index(node);
        let mut open = cstr!("{slot}, {}", js_string(tag));
        if let Some(index) = consts {
            append!(open, ", {index}");
        }
        let children: Vec<Node<'_, '_>> = node.children().collect();
        let mut listeners = Vec::new();
        for binding in node.bindings() {
            let name = binding
                .literal(OperandRole::Name)
                .ok_or_else(|| unsupported(binding))?;
            let value = operand(binding, OperandRole::Value)?;
            match binding.binding_kind() {
                Some("on") if code.item.is_none() && code.component == "ctx" => {
                    let handler = self.expression(value, code)?;
                    let function = cstr!("{}_{tag}_{name}_{slot}_listener", code.base);
                    let event = if handler.contains("$event") {
                        "$event"
                    } else {
                        ""
                    };
                    listeners.push(cstr!(
                        "i0.ɵɵlistener({}, function {function}({event}) {{ return {handler}; }});",
                        js_string(name)
                    ));
                }
                Some("bind") => {
                    let value = self.expression(value, code)?;
                    let (instruction, vars) = match name {
                        "class" => (cstr!("i0.ɵɵclassMap({value});"), 2),
                        "style" => (cstr!("i0.ɵɵstyleMap({value});"), 2),
                        _ => (cstr!("i0.ɵɵproperty({}, {value});", js_string(name)), 1),
                    };
                    code.update.push((slot, instruction));
                    code.vars += vars;
                }
                _ => return Err(unsupported(binding)),
            }
        }
        if children.is_empty() && listeners.is_empty() {
            code.create.push(cstr!("i0.ɵɵelement({open});"));
            return Ok(());
        }
        code.create.push(cstr!("i0.ɵɵelementStart({open});"));
        code.create.extend(listeners);
        let mut rest = children.as_slice();
        while let Some((child, tail)) = rest.split_first() {
            if child.kind() != OpKind::SetText {
                self.node(*child, code)?;
                rest = tail;
                continue;
            }
            let run = rest
                .iter()
                .take_while(|node| node.kind() == OpKind::SetText)
                .count();
            let (texts, after) = rest.split_at_checked(run).unwrap_or((rest, &[]));
            self.text(texts, code)?;
            rest = after;
        }
        code.create.push(String::from("i0.ɵɵelementEnd();"));
        Ok(())
    }

    /// One text node: static text, or Ivy's interpolation over the run.
    fn text(&mut self, texts: &[Node<'_, '_>], code: &mut ViewCode) -> Result<(), Error> {
        let slot = code.slot();
        let mut statics = vec![String::default()];
        let mut values = Vec::new();
        for text in texts {
            match text.literal(OperandRole::Text) {
                Some(literal) => {
                    if let Some(last) = statics.last_mut() {
                        last.push_str(literal);
                    }
                }
                None => {
                    values.push(self.expression(operand(*text, OperandRole::Text)?, code)?);
                    statics.push(String::default());
                }
            }
        }
        let quoted: Vec<String> = statics.iter().map(|text| js_string(text)).collect();
        match (values.as_slice(), quoted.as_slice()) {
            ([], [only]) => code.create.push(cstr!("i0.ɵɵtext({slot}, {only});")),
            ([value], [prefix, suffix]) if prefix == "\"\"" && suffix == "\"\"" => {
                code.create.push(cstr!("i0.ɵɵtext({slot});"));
                code.update
                    .push((slot, cstr!("i0.ɵɵtextInterpolate({value});")));
                code.vars += 1;
            }
            (values, _) if values.len() <= 8 => {
                code.create.push(cstr!("i0.ɵɵtext({slot});"));
                let mut args = Vec::new();
                for (index, text) in quoted.iter().enumerate() {
                    args.push(text.clone());
                    if let Some(value) = values.get(index) {
                        args.push(value.clone());
                    }
                }
                let instruction =
                    cstr!("i0.ɵɵtextInterpolate{}({});", values.len(), args.join(", "));
                code.update.push((slot, instruction));
                code.vars += u32::try_from(values.len()).unwrap_or(u32::MAX);
            }
            _ => {
                let span = texts.first().map_or(Span::new(0, 0), |text| text.span());
                return Err(Error::new(
                    "more than eight interpolations in one text",
                    span,
                ));
            }
        }
        Ok(())
    }

    fn conditional(&mut self, node: Node<'_, '_>, code: &mut ViewCode) -> Result<(), Error> {
        let mut decision = String::default();
        let mut first = None;
        let mut has_else = false;
        for region in node.regions().collect::<Vec<_>>() {
            let slot = code.slot();
            first.get_or_insert(slot);
            let base = cstr!("{}_Conditional_{slot}", code.base);
            let (decls, vars, tag) =
                self.embedded(&base, node.region(region).collect(), code, None)?;
            code.create.push(cstr!(
                "i0.ɵɵtemplate({slot}, {base}_Template, {decls}, {vars}{tag});"
            ));
            let condition = node
                .operands(OperandRole::Condition)
                .find(|operand| operand.region == Some(region))
                .ok_or_else(|| unsupported(node))?;
            if condition.value.kind == ValueKind::Absent {
                append!(decision, "{slot}");
                has_else = true;
            } else {
                append!(
                    decision,
                    "{} ? {slot} : ",
                    self.expression(condition, code)?
                );
            }
        }
        if !has_else {
            decision.push_str("-1");
        }
        code.update
            .push((first.unwrap_or(0), cstr!("i0.ɵɵconditional({decision});")));
        code.vars += 1;
        Ok(())
    }

    fn repeater(&mut self, node: Node<'_, '_>, code: &mut ViewCode) -> Result<(), Error> {
        let slot = code.slot();
        let list = self.expression(operand(node, OperandRole::ForSource)?, code)?;
        let item = js(operand(node, OperandRole::ForValue)?)?;
        let base = cstr!("{}_For_{slot}", code.base);
        self.contexts += 1;
        let local = cstr!("{item}_r{}", self.contexts);
        let (decls, vars, tag) = self.embedded(
            &base,
            node.children().collect(),
            code,
            Some((String::from(item), local)),
        )?;
        let tag = if tag.is_empty() {
            String::from(", null")
        } else {
            tag
        };
        code.create.push(cstr!(
            "i0.ɵɵrepeaterCreate({slot}, {base}_Template, {decls}, {vars}{tag}, null, i0.ɵɵrepeaterTrackByIdentity);"
        ));
        code.update.push((slot, cstr!("i0.ɵɵrepeater({list});")));
        code.vars += 1;
        Ok(())
    }

    /// Emit an embedded view's template function; returns its `decls`,
    /// `vars` and the `, "tag"` argument naming its single root element.
    fn embedded(
        &mut self,
        base: &str,
        nodes: Vec<Node<'_, '_>>,
        parent: &ViewCode,
        item: Option<(String, String)>,
    ) -> Result<(u32, u32, String), Error> {
        if parent.item.is_some() || parent.component != "ctx" {
            let span = nodes.first().map_or(Span::new(0, 0), |node| node.span());
            return Err(Error::new(
                "nested embedded views are outside this proof of concept",
                span,
            ));
        }
        self.contexts += 1;
        let component = cstr!("ctx_r{}", self.contexts);
        let mut code = ViewCode::new(String::from(base), component, item);
        let tag = match nodes.as_slice() {
            [only] => only
                .literal(OperandRole::Tag)
                .map(js_string)
                .map(|tag| cstr!(", {tag}")),
            _ => None,
        };
        for node in nodes {
            self.node(node, &mut code)?;
        }
        let name = cstr!("{base}_Template");
        self.functions.push(function(&name, &code));
        Ok((code.slots, code.vars, tag.unwrap_or_default()))
    }

    fn consts_index(&mut self, node: Node<'_, '_>) -> Option<usize> {
        let mut pairs = Vec::new();
        let mut classes = Vec::new();
        for attribute in node.operands(OperandRole::Attribute) {
            let name = attribute.name.unwrap_or_default();
            let value = match attribute.value.kind {
                ValueKind::Literal => attribute.value.text,
                _ => "",
            };
            if name == "class" {
                classes.extend(value.split_ascii_whitespace().map(js_string));
            } else {
                pairs.push(cstr!("{}, {}", js_string(name), js_string(value)));
            }
        }
        if !classes.is_empty() {
            pairs.push(cstr!("1, {}", classes.join(", ")));
        }
        if pairs.is_empty() {
            return None;
        }
        let array = cstr!("[{}]", pairs.join(", "));
        Some(
            match self.consts.iter().position(|existing| *existing == array) {
                Some(index) => index,
                None => {
                    self.consts.push(array);
                    self.consts.len() - 1
                }
            },
        )
    }

    /// Rewrite a template expression against the view context.
    fn expression(&self, operand: &Operand<'_>, code: &mut ViewCode) -> Result<String, Error> {
        let text = js(operand)?;
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, text, SourceType::mjs())
            .parse_expression()
            .map_err(|_| Error::new("unsupported expression", operand.value.span))?;
        let mut rewrite = Rewrite {
            code,
            edits: Vec::new(),
            refused: None,
        };
        rewrite.visit_expression(&parsed);
        if let Some(span) = rewrite.refused {
            let at = Span::new(
                operand.value.span.start + span.start,
                operand.value.span.start + span.end,
            );
            return Err(Error::new("unsupported template expression", at));
        }
        let mut edits = rewrite.edits;
        edits.sort_by_key(|(start, _, _)| *start);
        let mut out = String::default();
        let mut cursor = 0;
        for (start, end, replacement) in edits {
            out.push_str(text.get(cursor..start as usize).unwrap_or_default());
            out.push_str(&replacement);
            cursor = end as usize;
        }
        out.push_str(text.get(cursor..).unwrap_or_default());
        Ok(out)
    }
}

/// Free identifiers read the component; the `@for` item reads its local.
struct Rewrite<'c> {
    code: &'c mut ViewCode,
    edits: Vec<(u32, u32, String)>,
    refused: Option<oxc_span::Span>,
}

impl<'a> Visit<'a> for Rewrite<'_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        let name = it.name.as_str();
        let replacement = match &self.code.item {
            _ if name == "$event" => return,
            Some((item, local)) if item == name => {
                self.code.item_used = true;
                local.clone()
            }
            _ => {
                if self.code.component != "ctx" {
                    self.code.parent_used = true;
                }
                cstr!("{}.{name}", self.code.component)
            }
        };
        self.edits.push((it.span.start, it.span.end, replacement));
    }

    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        self.refused.get_or_insert(it.span);
    }

    fn visit_object_property(&mut self, it: &ObjectProperty<'a>) {
        if it.shorthand {
            self.refused.get_or_insert(it.span);
        }
        walk::walk_object_property(self, it);
    }
}

fn function(name: &str, code: &ViewCode) -> String {
    let mut text = cstr!("function {name}(rf, ctx) {{\n  if (rf & 1) {{");
    for line in &code.create {
        append!(text, "\n    {line}");
    }
    text.push_str("\n  }");
    if !code.update.is_empty() {
        text.push_str("\n  if (rf & 2) {");
        if let (Some((_, local)), true) = (&code.item, code.item_used) {
            append!(text, "\n    const {local} = ctx.$implicit;");
        }
        if code.parent_used {
            append!(text, "\n    const {} = i0.ɵɵnextContext();", code.component);
        }
        let mut selected = 0;
        for (slot, instruction) in &code.update {
            match slot.saturating_sub(selected) {
                0 => {}
                1 => text.push_str("\n    i0.ɵɵadvance();"),
                n => append!(text, "\n    i0.ɵɵadvance({n});"),
            }
            selected = *slot;
            append!(text, "\n    {}", indent(instruction, "    "));
        }
        text.push_str("\n  }");
    }
    text.push_str("\n}");
    text
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
