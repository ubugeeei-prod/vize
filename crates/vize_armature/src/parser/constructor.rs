use super::Parser;
use vize_relief::options::{CustomElementMatcher, ParserOptions, TemplateSyntaxMode};
use vize_s0::{Allocator, Vec, interner::Interner};

impl<'a> Parser<'a> {
    /// Create a new parser.
    pub fn new(allocator: &'a Allocator, source: &'a str) -> Self {
        Self::with_options(allocator, source, ParserOptions::default())
    }

    /// Create a new parser with options.
    pub fn with_options(allocator: &'a Allocator, source: &'a str, options: ParserOptions) -> Self {
        Self::with_options_and_template_syntax(
            allocator,
            source,
            options,
            TemplateSyntaxMode::Standard,
        )
    }

    /// Create a new parser with options and invalid HTML self-closing compatibility.
    #[deprecated(note = "use with_options_and_template_syntax instead")]
    pub fn with_options_and_invalid_html_self_closing(
        allocator: &'a Allocator,
        source: &'a str,
        options: ParserOptions,
        allow_invalid_html_self_closing: bool,
    ) -> Self {
        let syntax = if allow_invalid_html_self_closing {
            TemplateSyntaxMode::Quirks
        } else {
            TemplateSyntaxMode::Standard
        };
        Self::with_options_and_template_syntax(allocator, source, options, syntax)
    }

    /// Create a new parser with options and template syntax compatibility.
    pub fn with_options_and_template_syntax(
        allocator: &'a Allocator,
        source: &'a str,
        options: ParserOptions,
        template_syntax: TemplateSyntaxMode,
    ) -> Self {
        Self::with_options_custom_elements_and_template_syntax(
            allocator,
            source,
            options,
            CustomElementMatcher::default(),
            template_syntax,
        )
    }

    /// Create a new parser with options, custom-element patterns and syntax compatibility.
    #[doc(hidden)]
    pub fn with_options_custom_elements_and_template_syntax(
        allocator: &'a Allocator,
        source: &'a str,
        options: ParserOptions,
        custom_elements: CustomElementMatcher,
        template_syntax: TemplateSyntaxMode,
    ) -> Self {
        Self {
            allocator,
            oxc_allocator: allocator.as_oxc(),
            source,
            options,
            custom_elements,
            template_syntax,
            interner: Interner::new(allocator),
            pending_text: None,
            stack: Vec::new_in(&allocator),
            flattened_tags: Vec::new_in(&allocator),
            implicitly_closed_tags: Vec::new_in(&allocator),
            root: None,
            current_element: None,
            current_attr: None,
            current_dir: None,
            errors: std::vec::Vec::new(),
            in_pre: false,
            in_v_pre: false,
            open_table_count: 0,
            open_p_count: 0,
            open_a_count: 0,
            open_button_count: 0,
            open_form_count: 0,
            document: false,
        }
    }

    /// Create a new parser in full-HTML-document mode.
    ///
    /// Document mode is additive: it parses an entire HTML document (doctype +
    /// `<html>/<head>/<body>`, with `<script>`/`<style>` kept as raw text) into
    /// the same template AST, so downstream analysis (lint/scope) can run over a
    /// petite-vue HTML page where directives (`v-scope`, `v-effect`, `@click`)
    /// live on ordinary elements. The only behavioral difference from
    /// [`Parser::with_options`] is doctype tolerance; SFC `<template>` parsing is
    /// unaffected.
    pub fn new_document(allocator: &'a Allocator, source: &'a str) -> Self {
        Self::document_with_options(allocator, source, ParserOptions::default())
    }

    /// Create a new document-mode parser with options.
    pub fn document_with_options(
        allocator: &'a Allocator,
        source: &'a str,
        options: ParserOptions,
    ) -> Self {
        let mut parser = Self::with_options_and_template_syntax(
            allocator,
            source,
            options,
            TemplateSyntaxMode::Standard,
        );
        parser.document = true;
        parser
    }
}
