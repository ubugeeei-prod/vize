//! Structure-preserving layout for the markup regions in one TSRX module.

use vize_l0::{Allocator, Span, String, ToCompactString};
use vize_l1::markup::tsrx::{self, Attribute, AttributeValue, Child, Element, Fragment, Node};

use crate::{FormatError, FormatOptions};

mod directives;
mod structure;

pub(super) use structure::require_same_structure;

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
