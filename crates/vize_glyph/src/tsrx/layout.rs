//! Structure-preserving layout for the markup regions in one TSRX module.

use oxc_ast::ast::Expression;
use oxc_span::{ContentEq, GetSpan, SourceType, Span as OxcSpan};
use vize_l0::{Allocator, Span, String, ToCompactString};
use vize_l1::markup::tsrx::{
    self, Attribute, AttributeValue, Block, Catch, Child, Element, For, Fragment, If, Node, Switch,
    TagName, Try,
};

use crate::{FormatError, FormatOptions, format_script_with_source_type, script};

pub(super) fn format_regions(
    module: &tsrx::TsrxModule<'_>,
    source: &str,
    options: &FormatOptions,
    allocator: &Allocator,
) -> Result<String, FormatError> {
    let formatter = MarkupFormatter {
        module,
        source,
        options,
        allocator,
    };
    let mut output = String::with_capacity(source.len());
    let mut copied_until = 0;
    for region in module.regions() {
        let start = region.span.start as usize;
        let end = region.span.end as usize;
        if start < copied_until {
            continue;
        }
        let before = source.get(copied_until..start).ok_or_else(layout_error)?;
        output.push_str(before);
        let depth = line_depth(source, start, options);
        output.push_str(formatter.node(&region.node, depth)?.as_str());
        copied_until = end;
    }
    output.push_str(source.get(copied_until..).ok_or_else(layout_error)?);
    Ok(output)
}

struct MarkupFormatter<'a, 'm> {
    module: &'m tsrx::TsrxModule<'a>,
    source: &'a str,
    options: &'m FormatOptions,
    allocator: &'m Allocator,
}

impl MarkupFormatter<'_, '_> {
    fn node(&self, node: &Node<'_>, depth: usize) -> Result<String, FormatError> {
        match node {
            Node::Element(element) => self.element(element, depth),
            Node::Fragment(fragment) => self.fragment(fragment, depth),
            Node::CodeBlock(block) => self.block(block, depth, true),
            Node::If(directive) => self.if_directive(directive, depth),
            Node::For(directive) => self.for_directive(directive, depth),
            Node::Switch(directive) => self.switch_directive(directive, depth),
            Node::Try(directive) => self.try_directive(directive, depth),
        }
    }

    fn element(&self, element: &Element<'_>, depth: usize) -> Result<String, FormatError> {
        let opening = self.opening(element, depth)?;
        if element.is_self_closing() {
            return Ok(opening);
        }
        let close = self.closing_name(&element.name)?;
        if let Some(raw) = element.raw_text {
            let body = self.slice(raw)?;
            if !body.contains(['\n', '\r'])
                && line_width(depth, &opening, self.options) + body.len() + close.len() + 3
                    <= self.options.print_width as usize
            {
                return Ok(vize_l0::cstr!("{opening}{body}</{close}>"));
            }
            let mut output = opening;
            self.newline(&mut output);
            self.write_raw_lines(&mut output, body, depth + 1);
            self.newline(&mut output);
            self.indent(&mut output, depth);
            output.push_str("</");
            output.push_str(close.as_str());
            output.push('>');
            return Ok(output);
        }
        if let Some(children) = self.inline_children(&element.children, depth)? {
            let candidate = vize_l0::cstr!("{opening}{children}</{close}>");
            if !candidate.contains(['\n', '\r'])
                && line_width(depth, candidate.as_str(), self.options)
                    <= self.options.print_width as usize
            {
                return Ok(candidate);
            }
        }
        let mut output = opening;
        self.children(&mut output, &element.children, depth + 1)?;
        self.newline(&mut output);
        self.indent(&mut output, depth);
        output.push_str("</");
        output.push_str(close.as_str());
        output.push('>');
        Ok(output)
    }

    fn fragment(&self, fragment: &Fragment<'_>, depth: usize) -> Result<String, FormatError> {
        if let Some(children) = self.inline_children(&fragment.children, depth)? {
            let candidate = vize_l0::cstr!("<>{children}</>");
            if line_width(depth, candidate.as_str(), self.options)
                <= self.options.print_width as usize
            {
                return Ok(candidate);
            }
        }
        let mut output = String::from("<>");
        self.children(&mut output, &fragment.children, depth + 1)?;
        self.newline(&mut output);
        self.indent(&mut output, depth);
        output.push_str("</>");
        Ok(output)
    }

    fn opening(&self, element: &Element<'_>, depth: usize) -> Result<String, FormatError> {
        let mut head = String::from("<");
        head.push_str(self.tag_name(&element.name)?.as_str());
        let mut attributes = std::vec::Vec::with_capacity(element.attributes.len());
        let mut multiline = false;
        for attribute in &element.attributes {
            let formatted = self.attribute(attribute)?;
            multiline |= formatted.contains(['\n', '\r']);
            attributes.push(formatted);
        }
        let suffix = if element.is_self_closing() {
            " />"
        } else {
            ">"
        };
        let inline_len = head.len()
            + attributes
                .iter()
                .map(|attribute| attribute.len() + 1)
                .sum::<usize>()
            + suffix.len();
        let wrap = multiline
            || self.options.single_attribute_per_line
            || line_width(depth, "", self.options) + inline_len > self.options.print_width as usize;
        if !wrap {
            for attribute in attributes {
                head.push(' ');
                head.push_str(attribute.as_str());
            }
            head.push_str(suffix);
            return Ok(head);
        }
        for attribute in attributes {
            self.newline(&mut head);
            self.indent(&mut head, depth + 1);
            self.write_indented_multiline(&mut head, attribute.as_str(), depth + 1);
        }
        if self.options.bracket_same_line {
            head.push_str(suffix);
        } else {
            self.newline(&mut head);
            self.indent(&mut head, depth);
            head.push_str(suffix.trim_start());
        }
        Ok(head)
    }

    fn attribute(&self, attribute: &Attribute<'_>) -> Result<String, FormatError> {
        match attribute {
            Attribute::Named { name, value, .. } => {
                let mut output = String::from(self.slice(*name)?);
                if let Some(value) = value {
                    output.push('=');
                    match value {
                        AttributeValue::String(span) => output.push_str(self.slice(*span)?),
                        AttributeValue::Expression { expression, .. } => {
                            output.push('{');
                            output.push_str(self.expression(expression)?.as_str());
                            output.push('}');
                        }
                    }
                }
                Ok(output)
            }
            Attribute::Shorthand { expression, .. } => {
                Ok(vize_l0::cstr!("{{{}}}", self.expression(expression)?))
            }
            Attribute::Spread { expression, .. } => {
                Ok(vize_l0::cstr!("{{...{}}}", self.expression(expression)?))
            }
        }
    }

    fn inline_children(
        &self,
        children: &[Child<'_>],
        depth: usize,
    ) -> Result<Option<String>, FormatError> {
        let mut output = String::default();
        let mut node_count = 0;
        for child in children {
            match child {
                Child::Text(span) => output.push_str(normalize_text(self.slice(*span)?).as_str()),
                Child::Comment(_) => return Ok(None),
                Child::Expression(expression) => {
                    let value = match &expression.expression {
                        Some(expression) => self.expression(expression)?,
                        None => self
                            .slice(Span::new(
                                expression.span.start + 1,
                                expression.span.end - 1,
                            ))?
                            .trim()
                            .to_compact_string(),
                    };
                    if value.contains(['\n', '\r']) {
                        return Ok(None);
                    }
                    output.push('{');
                    output.push_str(value.as_str());
                    output.push('}');
                }
                Child::Node(node) => {
                    node_count += 1;
                    if node_count > 1 {
                        return Ok(None);
                    }
                    let value = self.node(node, depth)?;
                    if value.contains(['\n', '\r']) {
                        return Ok(None);
                    }
                    output.push_str(value.as_str());
                }
            }
        }
        Ok(Some(output.trim().to_compact_string()))
    }

    fn children(
        &self,
        output: &mut String,
        children: &[Child<'_>],
        depth: usize,
    ) -> Result<(), FormatError> {
        let mut wrote = false;
        for child in children {
            let (value, nested) = match child {
                Child::Text(span) => {
                    let value = normalize_text(self.slice(*span)?);
                    if value.trim().is_empty() {
                        continue;
                    }
                    (value.trim().to_compact_string(), false)
                }
                Child::Comment(comment) => {
                    (self.slice(comment.span)?.trim().to_compact_string(), false)
                }
                Child::Expression(expression) => {
                    let value = match &expression.expression {
                        Some(expression) => self.expression(expression)?,
                        None => self
                            .slice(Span::new(
                                expression.span.start + 1,
                                expression.span.end - 1,
                            ))?
                            .trim()
                            .to_compact_string(),
                    };
                    (vize_l0::cstr!("{{{value}}}"), false)
                }
                Child::Node(node) => (self.node(node, depth)?, true),
            };
            self.newline(output);
            self.indent(output, depth);
            if nested {
                output.push_str(value.as_str());
            } else {
                self.write_indented_multiline(output, value.as_str(), depth);
            }
            wrote = true;
        }
        if !wrote {
            // Preserve an explicit non-self-closing container as an empty pair.
        }
        Ok(())
    }

    fn block(&self, block: &Block<'_>, depth: usize, at: bool) -> Result<String, FormatError> {
        let mut output = String::from(if at { "@{" } else { "{" });
        let statements = self.block_statements(block)?;
        if !statements.is_empty() {
            self.newline(&mut output);
            self.write_lines_at_depth(&mut output, statements.as_str(), depth + 1);
        }
        if let Some(render) = &block.render {
            if !statements.is_empty() {
                self.newline(&mut output);
            }
            self.newline(&mut output);
            self.indent(&mut output, depth + 1);
            let render = self.node(render, depth + 1)?;
            output.push_str(render.as_str());
        }
        self.newline(&mut output);
        self.indent(&mut output, depth);
        output.push('}');
        Ok(output)
    }

    fn block_statements(&self, block: &Block<'_>) -> Result<String, FormatError> {
        if block.statements.is_empty() {
            return Ok(String::default());
        }
        let brace = self
            .source
            .get(block.span.start as usize..block.span.end as usize)
            .and_then(|body| body.find('{'))
            .map(|relative| block.span.start as usize + relative + 1)
            .ok_or_else(layout_error)?;
        let end = block
            .render
            .as_ref()
            .map_or(block.span.end as usize - 1, |render| {
                render.span().start as usize
            });
        let source = self.embedded_source(brace, end)?;
        let formatted = format_script_with_source_type(
            source.trim(),
            self.options,
            self.allocator,
            SourceType::tsx().with_module(true),
        )?;
        Ok(formatted.trim().to_compact_string())
    }

    fn if_directive(&self, directive: &If<'_>, depth: usize) -> Result<String, FormatError> {
        let mut output = String::default();
        for (index, branch) in directive.branches.iter().enumerate() {
            if index == 0 {
                output.push_str("@if (");
            } else if branch.test.is_some() {
                output.push_str(" @else if (");
            } else {
                output.push_str(" @else ");
            }
            if let Some(test) = &branch.test {
                output.push_str(self.expression(test)?.as_str());
                output.push_str(") ");
            }
            output.push_str(self.block(&branch.body, depth, false)?.as_str());
        }
        Ok(output)
    }

    fn for_directive(&self, directive: &For<'_>, depth: usize) -> Result<String, FormatError> {
        let mut output = String::from("@for ");
        if directive.r#await {
            output.push_str("await ");
        }
        output.push('(');
        output.push_str(self.for_left(directive.left.span())?.as_str());
        output.push_str(" of ");
        output.push_str(self.expression(&directive.right)?.as_str());
        if let Some(index) = &directive.index {
            output.push_str("; index ");
            output.push_str(self.slice_oxc(index.span)?);
        }
        if let Some(key) = &directive.key {
            output.push_str("; key ");
            output.push_str(self.expression(key)?.as_str());
        }
        output.push_str(") ");
        output.push_str(self.block(&directive.body, depth, false)?.as_str());
        if let Some(empty) = &directive.empty {
            output.push_str(" @empty ");
            output.push_str(self.block(empty, depth, false)?.as_str());
        }
        Ok(output)
    }

    fn switch_directive(
        &self,
        directive: &Switch<'_>,
        depth: usize,
    ) -> Result<String, FormatError> {
        let mut output =
            vize_l0::cstr!("@switch ({}) {{", self.expression(&directive.discriminant)?);
        for case in &directive.cases {
            self.newline(&mut output);
            self.indent(&mut output, depth + 1);
            match &case.test {
                Some(test) => {
                    output.push_str("@case ");
                    output.push_str(self.expression(test)?.as_str());
                }
                None => output.push_str("@default"),
            }
            output.push_str(": ");
            output.push_str(self.block(&case.body, depth + 1, false)?.as_str());
        }
        self.newline(&mut output);
        self.indent(&mut output, depth);
        output.push('}');
        Ok(output)
    }

    fn try_directive(&self, directive: &Try<'_>, depth: usize) -> Result<String, FormatError> {
        let mut output = String::from("@try ");
        output.push_str(self.block(&directive.body, depth, false)?.as_str());
        if let Some(pending) = &directive.pending {
            output.push_str(" @pending ");
            output.push_str(self.block(pending, depth, false)?.as_str());
        }
        if let Some(catch) = &directive.catch {
            output.push_str(" @catch");
            output.push_str(self.catch_params(catch)?.as_str());
            output.push(' ');
            output.push_str(self.block(&catch.body, depth, false)?.as_str());
        }
        Ok(output)
    }

    fn catch_params(&self, catch: &Catch<'_>) -> Result<String, FormatError> {
        catch.params.as_ref().map_or_else(
            || Ok(String::default()),
            |params| {
                let authored = self.slice_oxc(params.span)?.trim();
                let wrapped = vize_l0::cstr!("function __vize{authored} {{}}");
                let formatted = format_script_with_source_type(
                    wrapped.as_str(),
                    self.options,
                    self.allocator,
                    SourceType::ts().with_module(true),
                )?;
                formatted
                    .trim()
                    .strip_prefix("function __vize")
                    .and_then(|function| function.split_once(" {").map(|(params, _)| params))
                    .filter(|params| !params.contains(['\n', '\r']))
                    .map(ToCompactString::to_compact_string)
                    .ok_or_else(layout_error)
            },
        )
    }

    fn for_left(&self, span: OxcSpan) -> Result<String, FormatError> {
        let source = self.slice(Span::new(span.start, span.end))?.trim();
        let wrapped = vize_l0::cstr!("for ({source} of __vize_value) {{}}");
        let formatted = format_script_with_source_type(
            wrapped.as_str(),
            self.options,
            self.allocator,
            SourceType::tsx().with_module(true),
        )?;
        formatted
            .trim()
            .strip_prefix("for (")
            .and_then(|body| body.split_once(" of __vize_value)"))
            .filter(|(_, tail)| tail.trim() == "{\n}")
            .map(|(binding, _)| binding)
            .map(ToCompactString::to_compact_string)
            .ok_or_else(|| {
                FormatError::ScriptFormatError(vize_l0::cstr!(
                    "could not isolate formatted TSRX @for binding: {formatted}"
                ))
            })
    }

    fn expression(&self, expression: &Expression<'_>) -> Result<String, FormatError> {
        let span = expression.span();
        let source = self.embedded_source(span.start as usize, span.end as usize)?;
        script::format_tsx_expression(source.trim(), self.options).ok_or_else(|| {
            FormatError::ScriptFormatError(vize_l0::cstr!(
                "could not format embedded TSRX expression: {source}"
            ))
        })
    }

    fn embedded_source(&self, start: usize, end: usize) -> Result<String, FormatError> {
        let mut output = String::with_capacity(end.saturating_sub(start));
        let mut copied_until = start;
        for region in self.module.regions() {
            let region_start = region.span.start as usize;
            let region_end = region.span.end as usize;
            if region_start < start || region_end > end || region_start < copied_until {
                continue;
            }
            output.push_str(
                self.source
                    .get(copied_until..region_start)
                    .ok_or_else(layout_error)?,
            );
            match &region.node {
                Node::Element(_) | Node::Fragment(_) => {
                    output.push_str(self.node(&region.node, 0)?.as_str());
                }
                _ => return Err(layout_error()),
            }
            copied_until = region_end;
        }
        output.push_str(
            self.source
                .get(copied_until..end)
                .ok_or_else(layout_error)?,
        );
        Ok(output)
    }

    fn tag_name(&self, name: &TagName<'_>) -> Result<String, FormatError> {
        match name {
            TagName::Static(span) => Ok(String::from(self.slice(*span)?)),
            TagName::Dynamic { expression, .. } => {
                Ok(vize_l0::cstr!("{{{}}}", self.expression(expression)?))
            }
        }
    }

    fn closing_name(&self, name: &TagName<'_>) -> Result<String, FormatError> {
        self.tag_name(name)
    }

    fn slice(&self, span: Span) -> Result<&str, FormatError> {
        self.source
            .get(span.start as usize..span.end as usize)
            .ok_or_else(layout_error)
    }

    fn slice_oxc(&self, span: OxcSpan) -> Result<&str, FormatError> {
        self.source
            .get(span.start as usize..span.end as usize)
            .ok_or_else(layout_error)
    }

    fn newline(&self, output: &mut String) {
        output.push_str(self.options.newline_string());
    }

    fn indent(&self, output: &mut String, depth: usize) {
        for _ in 0..depth {
            output.push_str(self.options.indent_string().as_str());
        }
    }

    fn write_indented_multiline(&self, output: &mut String, value: &str, depth: usize) {
        let mut lines = value.lines();
        if let Some(first) = lines.next() {
            output.push_str(first);
        }
        for line in lines {
            self.newline(output);
            self.indent(output, depth);
            output.push_str(line.trim_start());
        }
    }

    fn write_lines_at_depth(&self, output: &mut String, value: &str, depth: usize) {
        for (index, line) in value.lines().enumerate() {
            if index > 0 {
                self.newline(output);
            }
            self.indent(output, depth);
            output.push_str(line);
        }
    }

    fn write_raw_lines(&self, output: &mut String, value: &str, depth: usize) {
        for (index, line) in value.lines().enumerate() {
            if index > 0 {
                self.newline(output);
            }
            self.indent(output, depth);
            output.push_str(line.trim());
        }
    }
}

fn normalize_text(text: &str) -> String {
    if text.contains(['\n', '\r', '\t']) {
        let leading = text.chars().next().is_some_and(char::is_whitespace);
        let trailing = text.chars().next_back().is_some_and(char::is_whitespace);
        let words = text.split_whitespace().collect::<std::vec::Vec<_>>();
        if words.is_empty() {
            return String::from(" ");
        }
        let mut normalized = String::with_capacity(text.len());
        if leading {
            normalized.push(' ');
        }
        normalized.push_str(words.join(" ").as_str());
        if trailing {
            normalized.push(' ');
        }
        normalized
    } else {
        String::from(text)
    }
}

fn line_depth(source: &str, offset: usize, options: &FormatOptions) -> usize {
    let line_start = source
        .get(..offset)
        .and_then(|prefix| prefix.rfind(['\n', '\r']).map(|index| index + 1))
        .unwrap_or(0);
    let prefix = source.get(line_start..offset).unwrap_or_default();
    let mut columns = 0;
    for byte in prefix.bytes() {
        match byte {
            b' ' => columns += 1,
            b'\t' => columns += usize::from(options.tab_width.max(1)),
            _ => break,
        }
    }
    columns / usize::from(options.tab_width.max(1))
}

fn line_width(depth: usize, value: &str, options: &FormatOptions) -> usize {
    let indent = if options.use_tabs {
        depth
    } else {
        depth * usize::from(options.tab_width)
    };
    indent + value.len()
}

fn layout_error() -> FormatError {
    FormatError::ScriptFormatError(
        "TSRX construct could not be laid out without rewriting it".into(),
    )
}

pub(super) fn require_same_structure(
    before: &tsrx::TsrxModule<'_>,
    before_source: &str,
    after: &tsrx::TsrxModule<'_>,
    after_source: &str,
) -> Result<(), FormatError> {
    if !before.program().content_eq(after.program())
        || before.regions().len() != after.regions().len()
        || !before
            .regions()
            .iter()
            .zip(after.regions())
            .all(|(left, right)| {
                left.site == right.site
                    && node_eq(&left.node, before_source, &right.node, after_source)
            })
    {
        return Err(FormatError::ScriptFormatError(
            "TSRX layout changed the parsed L1 structure".into(),
        ));
    }
    Ok(())
}

fn node_eq(left: &Node<'_>, left_source: &str, right: &Node<'_>, right_source: &str) -> bool {
    match (left, right) {
        (Node::Element(left), Node::Element(right)) => {
            tag_eq(&left.name, left_source, &right.name, right_source)
                && left.attributes.len() == right.attributes.len()
                && left
                    .attributes
                    .iter()
                    .zip(&right.attributes)
                    .all(|(left, right)| attr_eq(left, left_source, right, right_source))
                && children_eq(&left.children, left_source, &right.children, right_source)
                && option_text_eq(left.raw_text, left_source, right.raw_text, right_source)
                && left.closing.is_some() == right.closing.is_some()
        }
        (Node::Fragment(left), Node::Fragment(right)) => {
            children_eq(&left.children, left_source, &right.children, right_source)
        }
        (Node::CodeBlock(left), Node::CodeBlock(right)) => {
            block_eq(left, left_source, right, right_source)
        }
        (Node::If(left), Node::If(right)) => {
            left.branches.len() == right.branches.len()
                && left
                    .branches
                    .iter()
                    .zip(&right.branches)
                    .all(|(left, right)| {
                        option_content_eq(left.test.as_ref(), right.test.as_ref())
                            && block_eq(&left.body, left_source, &right.body, right_source)
                    })
        }
        (Node::For(left), Node::For(right)) => {
            left.r#await == right.r#await
                && left.left.content_eq(&right.left)
                && left.right.content_eq(&right.right)
                && option_content_eq(left.index.as_ref(), right.index.as_ref())
                && option_content_eq(left.key.as_ref(), right.key.as_ref())
                && block_eq(&left.body, left_source, &right.body, right_source)
                && option_block_eq(
                    left.empty.as_ref(),
                    left_source,
                    right.empty.as_ref(),
                    right_source,
                )
        }
        (Node::Switch(left), Node::Switch(right)) => {
            left.discriminant.content_eq(&right.discriminant)
                && left.cases.len() == right.cases.len()
                && left.cases.iter().zip(&right.cases).all(|(left, right)| {
                    option_content_eq(left.test.as_ref(), right.test.as_ref())
                        && block_eq(&left.body, left_source, &right.body, right_source)
                })
        }
        (Node::Try(left), Node::Try(right)) => {
            block_eq(&left.body, left_source, &right.body, right_source)
                && option_block_eq(
                    left.pending.as_ref(),
                    left_source,
                    right.pending.as_ref(),
                    right_source,
                )
                && match (&left.catch, &right.catch) {
                    (Some(left), Some(right)) => {
                        option_content_eq(left.params.as_deref(), right.params.as_deref())
                            && block_eq(&left.body, left_source, &right.body, right_source)
                    }
                    (None, None) => true,
                    _ => false,
                }
        }
        _ => false,
    }
}

fn block_eq(left: &Block<'_>, left_source: &str, right: &Block<'_>, right_source: &str) -> bool {
    left.statements.content_eq(&right.statements)
        && match (&left.render, &right.render) {
            (Some(left), Some(right)) => node_eq(left, left_source, right, right_source),
            (None, None) => true,
            _ => false,
        }
}

fn option_block_eq(
    left: Option<&Block<'_>>,
    left_source: &str,
    right: Option<&Block<'_>>,
    right_source: &str,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => block_eq(left, left_source, right, right_source),
        (None, None) => true,
        _ => false,
    }
}

fn children_eq(
    left: &[Child<'_>],
    left_source: &str,
    right: &[Child<'_>],
    right_source: &str,
) -> bool {
    let left = semantic_children(left, left_source);
    let right = semantic_children(right, right_source);
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| match (left, right) {
                (SemanticChild::Text(left), SemanticChild::Text(right)) => left == right,
                (SemanticChild::Comment(left), SemanticChild::Comment(right)) => left == right,
                (SemanticChild::Expression(left), SemanticChild::Expression(right)) => {
                    option_content_eq(left.expression.as_ref(), right.expression.as_ref())
                }
                (SemanticChild::Node(left), SemanticChild::Node(right)) => {
                    node_eq(left, left_source, right, right_source)
                }
                _ => false,
            })
}

enum SemanticChild<'a, 'b> {
    Text(String),
    Comment(String),
    Expression(&'b tsrx::ExpressionChild<'a>),
    Node(&'b Node<'a>),
}

fn semantic_children<'a, 'b>(
    children: &'b [Child<'a>],
    source: &str,
) -> std::vec::Vec<SemanticChild<'a, 'b>> {
    children
        .iter()
        .filter_map(|child| match child {
            Child::Text(span) => {
                let text = source.get(span.start as usize..span.end as usize)?.trim();
                (!text.is_empty()).then(|| SemanticChild::Text(normalize_text(text)))
            }
            Child::Comment(comment) => source
                .get(comment.span.start as usize..comment.span.end as usize)
                .map(str::trim)
                .map(ToCompactString::to_compact_string)
                .map(SemanticChild::Comment),
            Child::Expression(expression) => Some(SemanticChild::Expression(expression)),
            Child::Node(node) => Some(SemanticChild::Node(node)),
        })
        .collect()
}

fn attr_eq(
    left: &Attribute<'_>,
    left_source: &str,
    right: &Attribute<'_>,
    right_source: &str,
) -> bool {
    match (left, right) {
        (
            Attribute::Named {
                name: left_name,
                value: left_value,
                ..
            },
            Attribute::Named {
                name: right_name,
                value: right_value,
                ..
            },
        ) => {
            text(left_source, *left_name) == text(right_source, *right_name)
                && match (left_value, right_value) {
                    (Some(AttributeValue::String(left)), Some(AttributeValue::String(right))) => {
                        text(left_source, *left) == text(right_source, *right)
                    }
                    (
                        Some(AttributeValue::Expression {
                            expression: left, ..
                        }),
                        Some(AttributeValue::Expression {
                            expression: right, ..
                        }),
                    ) => left.content_eq(right),
                    (None, None) => true,
                    _ => false,
                }
        }
        (
            Attribute::Shorthand {
                expression: left, ..
            },
            Attribute::Shorthand {
                expression: right, ..
            },
        )
        | (
            Attribute::Spread {
                expression: left, ..
            },
            Attribute::Spread {
                expression: right, ..
            },
        ) => left.content_eq(right),
        _ => false,
    }
}

fn tag_eq(left: &TagName<'_>, left_source: &str, right: &TagName<'_>, right_source: &str) -> bool {
    match (left, right) {
        (TagName::Static(left), TagName::Static(right)) => {
            text(left_source, *left) == text(right_source, *right)
        }
        (
            TagName::Dynamic {
                expression: left, ..
            },
            TagName::Dynamic {
                expression: right, ..
            },
        ) => left.content_eq(right),
        _ => false,
    }
}

fn option_text_eq(
    left: Option<Span>,
    left_source: &str,
    right: Option<Span>,
    right_source: &str,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => text(left_source, left) == text(right_source, right),
        (None, None) => true,
        _ => false,
    }
}

fn option_content_eq<T: ContentEq>(left: Option<&T>, right: Option<&T>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.content_eq(right),
        (None, None) => true,
        _ => false,
    }
}

fn text(source: &str, span: Span) -> &str {
    source
        .get(span.start as usize..span.end as usize)
        .unwrap_or_default()
}
