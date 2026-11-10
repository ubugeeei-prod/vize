//! Directive, embedded-script, and output helpers for TSRX markup layout.

use oxc_ast::ast::Expression;
use oxc_span::{GetSpan, SourceType, Span as OxcSpan};
use vize_l0::{Span, String, ToCompactString};
use vize_l1::markup::tsrx::{Block, Catch, For, If, Node, Switch, TagName, Try};

use crate::{FormatError, format_script_with_source_type, script};

use super::{MarkupFormatter, layout_error};

impl MarkupFormatter<'_, '_> {
    pub(super) fn block(
        &self,
        block: &Block<'_>,
        depth: usize,
        at: bool,
    ) -> Result<String, FormatError> {
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

    pub(super) fn if_directive(
        &self,
        directive: &If<'_>,
        depth: usize,
    ) -> Result<String, FormatError> {
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

    pub(super) fn for_directive(
        &self,
        directive: &For<'_>,
        depth: usize,
    ) -> Result<String, FormatError> {
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

    pub(super) fn switch_directive(
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

    pub(super) fn try_directive(
        &self,
        directive: &Try<'_>,
        depth: usize,
    ) -> Result<String, FormatError> {
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

    pub(super) fn expression(&self, expression: &Expression<'_>) -> Result<String, FormatError> {
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

    pub(super) fn tag_name(&self, name: &TagName<'_>) -> Result<String, FormatError> {
        match name {
            TagName::Static(span) => Ok(String::from(self.slice(*span)?)),
            TagName::Dynamic { expression, .. } => {
                Ok(vize_l0::cstr!("{{{}}}", self.expression(expression)?))
            }
        }
    }

    pub(super) fn closing_name(&self, name: &TagName<'_>) -> Result<String, FormatError> {
        self.tag_name(name)
    }

    pub(super) fn slice(&self, span: Span) -> Result<&str, FormatError> {
        self.source
            .get(span.start as usize..span.end as usize)
            .ok_or_else(layout_error)
    }

    fn slice_oxc(&self, span: OxcSpan) -> Result<&str, FormatError> {
        self.source
            .get(span.start as usize..span.end as usize)
            .ok_or_else(layout_error)
    }

    pub(super) fn newline(&self, output: &mut String) {
        output.push_str(self.options.newline_string());
    }

    pub(super) fn indent(&self, output: &mut String, depth: usize) {
        for _ in 0..depth {
            output.push_str(self.options.indent_string().as_str());
        }
    }

    pub(super) fn write_indented_multiline(&self, output: &mut String, value: &str, depth: usize) {
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

    pub(super) fn write_raw_lines(&self, output: &mut String, value: &str, depth: usize) {
        for (index, line) in value.lines().enumerate() {
            if index > 0 {
                self.newline(output);
            }
            self.indent(output, depth);
            output.push_str(line.trim());
        }
    }
}
