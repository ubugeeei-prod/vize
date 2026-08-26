//! Free-function entry points for the template parser.
//!
//! These are thin wrappers over the [`Parser`](super::Parser) constructors. They
//! live in their own module so `parser.rs` stays within the repository's
//! per-file source budget.

use vize_relief::{
    RootNode,
    errors::CompilerError,
    options::{CustomElementMatcher, ParserOptions, TemplateSyntaxMode},
};
use vize_s0::Allocator;

use super::Parser;

/// Parse a Vue template
pub fn parse<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (RootNode<'a>, std::vec::Vec<CompilerError>) {
    Parser::new(allocator, source).parse()
}

/// Parse a full HTML document (petite-vue / standalone HTML) into the template AST.
///
/// Unlike [`parse`], which expects an SFC `<template>` block, this entry point
/// tolerates a leading `<!DOCTYPE html>` declaration and parses the whole
/// document (`<html>/<head>/<body>`, `<script>`/`<style>` as raw text) so
/// downstream lint/scope analysis can run on petite-vue pages whose directives
/// sit on ordinary DOM elements. Additive: existing template parsing is
/// unchanged.
pub fn parse_document<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (RootNode<'a>, std::vec::Vec<CompilerError>) {
    Parser::new_document(allocator, source).parse()
}

/// Parse a full HTML document with options. See [`parse_document`].
pub fn parse_document_with_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: ParserOptions,
) -> (RootNode<'a>, std::vec::Vec<CompilerError>) {
    Parser::document_with_options(allocator, source, options).parse()
}

/// Parse a Vue template with options
pub fn parse_with_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: ParserOptions,
) -> (RootNode<'a>, std::vec::Vec<CompilerError>) {
    Parser::with_options(allocator, source, options).parse()
}

/// Parse a Vue template with options and invalid HTML self-closing compatibility.
#[deprecated(note = "use parse_with_options_and_template_syntax instead")]
pub fn parse_with_options_and_invalid_html_self_closing<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: ParserOptions,
    allow_invalid_html_self_closing: bool,
) -> (RootNode<'a>, std::vec::Vec<CompilerError>) {
    Parser::with_options_and_template_syntax(
        allocator,
        source,
        options,
        if allow_invalid_html_self_closing {
            TemplateSyntaxMode::Quirks
        } else {
            TemplateSyntaxMode::Standard
        },
    )
    .parse()
}

/// Parse a Vue template with options and template syntax compatibility.
pub fn parse_with_options_and_template_syntax<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: ParserOptions,
    template_syntax: TemplateSyntaxMode,
) -> (RootNode<'a>, std::vec::Vec<CompilerError>) {
    Parser::with_options_and_template_syntax(allocator, source, options, template_syntax).parse()
}

/// Parse with declarative custom-element patterns without growing [`ParserOptions`].
#[doc(hidden)]
pub fn parse_with_options_custom_elements_and_template_syntax<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: ParserOptions,
    custom_elements: CustomElementMatcher,
    template_syntax: TemplateSyntaxMode,
) -> (RootNode<'a>, std::vec::Vec<CompilerError>) {
    Parser::with_options_custom_elements_and_template_syntax(
        allocator,
        source,
        options,
        custom_elements,
        template_syntax,
    )
    .parse()
}
