//! Block code generation for Vapor mode.

use crate::ir::{BlockIRNode, IREffect, OperationNode};
use vize_carton::{cstr, String};

/// Context for code generation
pub struct GenerateContext {
    pub code: String,
    pub indent_level: u32,
    pub temp_count: usize,
}

impl GenerateContext {
    pub fn new() -> Self {
        Self {
            code: String::with_capacity(4096),
            indent_level: 0,
            temp_count: 0,
        }
    }

    pub fn push(&mut self, s: &str) {
        self.code.push_str(s);
    }

    pub fn push_line(&mut self, s: &str) {
        self.push_indent();
        self.code.push_str(s);
        self.code.push('\n');
    }

    pub fn push_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.code.push_str("  ");
        }
    }

    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn deindent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    pub fn next_temp(&mut self) -> String {
        let name = cstr!("_t{}", self.temp_count);
        self.temp_count += 1;
        name
    }

    pub fn newline(&mut self) {
        self.code.push('\n');
    }

    /// Push string to buffer (compatible with `append!` macro)
    pub fn push_str(&mut self, s: &str) {
        self.code.push_str(s);
    }

    /// Push formatted line (format_args! + newline with indentation)
    pub fn push_line_fmt(&mut self, args: std::fmt::Arguments<'_>) {
        self.push_indent();
        use std::fmt::Write as _;
        self.write_fmt(args).unwrap();
        self.code.push('\n');
    }
}

impl Default for GenerateContext {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Write for GenerateContext {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.code.push_str(s);
        Ok(())
    }
}

/// Generate block code
pub fn generate_block(
    ctx: &mut GenerateContext,
    block: &BlockIRNode<'_>,
    generate_operation: impl Fn(&mut GenerateContext, &OperationNode<'_>),
    generate_effect: impl Fn(&mut GenerateContext, &IREffect<'_>),
) {
    // Generate operations
    for op in block.operation.iter() {
        generate_operation(ctx, op);
    }

    // Generate effects
    for effect in block.effect.iter() {
        generate_effect(ctx, effect);
    }

    // Generate return statement
    if !block.returns.is_empty() {
        let returns = block
            .returns
            .iter()
            .map(|r| cstr!("_n{r}"))
            .collect::<Vec<_>>()
            .join(", ");

        if block.returns.len() == 1 {
            ctx.push_line_fmt(format_args!("return {returns}"));
        } else {
            ctx.push_line_fmt(format_args!("return [{returns}]"));
        }
    }
}

/// Generate effect wrapper
pub fn generate_effect_wrapper(
    ctx: &mut GenerateContext,
    operations: impl FnOnce(&mut GenerateContext),
) {
    ctx.push_line("_renderEffect(() => {");
    ctx.indent();
    operations(ctx);
    ctx.deindent();
    ctx.push_line("})");
}

/// Generate template instantiation
pub fn generate_template_instantiation(
    ctx: &mut GenerateContext,
    element_id: usize,
    template_index: usize,
) {
    ctx.push_line_fmt(format_args!(
        "const _n{element_id} = _tmpl${template_index}()"
    ));
}

/// Generate template declaration
pub fn generate_template_declaration(
    ctx: &mut GenerateContext,
    template_index: usize,
    template: &str,
) {
    ctx.push_line_fmt(format_args!(
        "const _tmpl${template_index} = _template(\"{}\")",
        escape_template(template)
    ));
}

/// Escape template string for JavaScript
pub fn escape_template(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .into()
}

#[cfg(test)]
mod tests {
    use super::{escape_template, GenerateContext};

    #[test]
    fn test_generate_context() {
        let mut ctx = GenerateContext::new();
        ctx.push_line("const x = 1");
        insta::assert_snapshot!(ctx.code.as_str());
    }

    #[test]
    fn test_escape_template() {
        assert_eq!(escape_template("hello"), "hello");
        assert_eq!(escape_template("hello\nworld"), "hello\\nworld");
    }
}
