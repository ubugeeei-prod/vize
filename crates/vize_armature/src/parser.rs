//! Vue template parser.
//!
//! This parser uses the tokenizer to convert Vue templates into an AST.
//! It is split into submodules for organization:
//! - `element` - Element, text, interpolation, comment, and error processing
//! - `attribute` - Attribute and directive processing
//! - `callbacks` - Tokenizer callback implementation
//! - `whitespace` - Whitespace condensing logic

mod attribute;
mod callbacks;
mod constructor;
mod delimiters;
mod element;
mod entry;
#[cfg(test)]
mod experimental_tests;
mod expression;
mod pending_text;
mod whitespace;

pub use entry::*;

#[cfg(test)]
mod tests;

use vize_relief::{
    ElementNode, Namespace, PropNode, RootNode, SourceLocation, TemplateChildNode,
    errors::{CompilerError, ErrorCode},
    options::{CustomElementMatcher, ParserOptions, TemplateSyntaxMode, WhitespaceStrategy},
};
use vize_s0::{Allocator, String, Vec, interner::Interner};

use element::{note_html_tree_element_close, note_html_tree_element_open};
pub(in crate::parser) use pending_text::{PendingText, TextSlot};
use whitespace::condense_whitespace;

pub struct Parser<'a> {
    allocator: &'a Allocator,
    /// The compile's oxc arena pool: retained expression ASTs (Davinci P1-5)
    /// are parsed into it so they share the template tree's lifetime.
    oxc_allocator: &'a oxc_allocator::Allocator,
    source: &'a str,
    options: ParserOptions,
    custom_elements: CustomElementMatcher,
    /// Template syntax compatibility mode.
    template_syntax: TemplateSyntaxMode,
    /// Per-compile atoms for the computed names the parser synthesizes
    /// (camelized shorthand arguments, reconstructed `v-pre` attribute
    /// names). Verbatim names are source slices and never reach it.
    interner: Interner<'a>,
    /// Text run whose decoded bytes diverge from the source (entities), being
    /// accumulated before it is frozen into the node's `&'a str`. Buffering
    /// here keeps a run of N entities linear instead of recopying the run per
    /// entity; [`Parser::flush_pending_text`] runs at every tokenizer callback
    /// boundary that is not itself text, so nothing ever observes a stale node.
    pending_text: Option<PendingText>,
    /// Current node stack
    stack: Vec<'a, ParserStackEntry<'a>>,
    /// Tags of the elements the nesting limit refused to descend into, in source
    /// order. They are attached to the tree as leaves instead of being pushed
    /// onto `stack`, so without this their end tags would find nothing to close
    /// and be reported as `InvalidEndTag` even though the source is correct.
    flattened_tags: Vec<'a, &'a str>,
    /// Root node
    root: Option<RootNode<'a>>,
    /// Current element being parsed
    current_element: Option<CurrentElement<'a>>,
    /// Current attribute being parsed
    current_attr: Option<CurrentAttribute<'a>>,
    /// Current directive being parsed
    current_dir: Option<CurrentDirective<'a>>,
    /// Errors collected during parsing.
    ///
    /// Diagnostics own their message text (the arena/cache contract keeps
    /// owned strings out of arena containers), so this is a plain heap vector.
    errors: std::vec::Vec<CompilerError>,
    /// Whether in pre block
    in_pre: bool,
    /// Whether in v-pre block
    in_v_pre: bool,
    open_table_count: usize,
    open_p_count: usize,
    open_a_count: usize,
    open_button_count: usize,
    open_form_count: usize,
    /// Whether the parser is in full-HTML-document mode (petite-vue / standalone
    /// HTML). When set, the tokenizer tolerates the leading doctype declaration
    /// so a real document's `<!DOCTYPE html>` is not reported as a parse error.
    /// SFC `<template>` parsing leaves this `false` and stays byte-identical.
    document: bool,
}

/// Stack entry for tracking parent elements
#[derive(Debug)]
pub(super) struct ParserStackEntry<'a> {
    pub(super) element: ElementNode<'a>,
    pub(super) in_pre: bool,
    pub(super) in_v_pre: bool,
    pub(super) insertion: StackInsertion,
    pub(super) implicit: bool,
    pub(super) fostered_before: Vec<'a, TemplateChildNode<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StackInsertion {
    Normal,
    Fostered,
}

/// Current element being parsed
pub(super) struct CurrentElement<'a> {
    pub(super) tag: &'a str,
    pub(super) tag_start: usize,
    #[allow(dead_code)]
    pub(super) tag_end: usize,
    pub(super) ns: Namespace,
    pub(super) is_self_closing: bool,
    pub(super) props: Vec<'a, PropNode<'a>>,
}

/// Current attribute being parsed
pub(super) struct CurrentAttribute<'a> {
    pub(super) name: &'a str,
    pub(super) name_start: usize,
    pub(super) name_end: usize,
    pub(super) value_start: Option<usize>,
    pub(super) value_end: Option<usize>,
    pub(super) value_content: Option<String>,
    pub(super) _marker: std::marker::PhantomData<&'a ()>,
}

/// Current directive being parsed
pub(super) struct CurrentDirective<'a> {
    pub(super) name: &'a str,
    pub(super) raw_name: &'a str,
    pub(super) name_start: usize,
    #[allow(dead_code)]
    pub(super) name_end: usize,
    pub(super) arg: Option<(&'a str, usize, usize, bool)>, // (content, start, end, is_dynamic)
    pub(super) modifiers: Vec<'a, (&'a str, usize, usize)>,
    pub(super) value_start: Option<usize>,
    pub(super) value_end: Option<usize>,
    pub(super) value_content: Option<String>,
    pub(super) _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Parser<'a> {
    /// Parse the source and return the AST
    pub fn parse(mut self) -> (RootNode<'a>, std::vec::Vec<CompilerError>) {
        // Initialize root node
        let allocator = self.allocator;
        let root = RootNode::new(allocator, self.source);
        self.root = Some(root);

        if !self.tokenize_template() {
            return self.into_result();
        }

        // Freeze the last text run before anything walks the finished tree.
        self.flush_pending_text();

        // Handle any unclosed elements
        self.handle_unclosed_elements();

        // Condense whitespace if needed
        if let Some(ref mut root) = self.root
            && self.options.whitespace == WhitespaceStrategy::Condense
        {
            condense_whitespace(allocator, &mut root.children, self.options.is_pre_tag);
        }

        self.into_result()
    }

    /// Get source slice
    fn get_source(&self, start: usize, end: usize) -> &str {
        let (start, end) = self.normalize_span(start, end);
        &self.source[start..end]
    }

    /// Get a source slice tied to the arena lifetime (`get_source` narrows to
    /// `&self`); retained expression ASTs parse from `'a` text.
    fn get_source_retained(&self, start: usize, end: usize) -> &'a str {
        let (start, end) = self.normalize_span(start, end);
        &self.source[start..end]
    }

    fn normalize_span(&self, start: usize, end: usize) -> (usize, usize) {
        let mut start = self.clamp_to_char_boundary(start);
        let end = self.clamp_to_char_boundary(end);
        if start > end {
            start = end;
        }
        (start, end)
    }

    fn clamp_to_char_boundary(&self, offset: usize) -> usize {
        let mut offset = offset.min(self.source.len());
        while offset > 0 && !self.source.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }

    /// Create a source location
    fn create_loc(&self, start: usize, end: usize) -> SourceLocation {
        let (start, end) = self.normalize_span(start, end);
        SourceLocation::new(start as u32, end as u32)
    }

    /// Add child to current context (stack top or root)
    fn add_child(&mut self, child: TemplateChildNode<'a>) {
        if let Some(entry) = self.stack.last_mut() {
            entry.element.children.push(child);
        } else if let Some(ref mut root) = self.root {
            root.children.push(child);
        }
    }

    fn add_fostered_child(&mut self, child: TemplateChildNode<'a>) {
        if let Some(table_index) = self.nearest_table_index() {
            self.stack[table_index].fostered_before.push(child);
        } else {
            self.add_child(child);
        }
    }

    pub(super) fn push_stack_entry(&mut self, entry: ParserStackEntry<'a>) {
        note_html_tree_element_open(self, &entry.element);
        self.stack.push(entry);
    }

    pub(super) fn pop_stack_entry(&mut self) -> Option<ParserStackEntry<'a>> {
        let entry = self.stack.pop()?;
        note_html_tree_element_close(self, &entry.element);
        Some(entry)
    }

    /// Handle unclosed elements at end of parsing
    fn handle_unclosed_elements(&mut self) {
        while let Some(entry) = self.pop_stack_entry() {
            if !entry.implicit && !Self::can_omit_end_tag(entry.element.tag) {
                let loc = entry.element.loc.clone();
                self.errors
                    .push(CompilerError::new(ErrorCode::MissingEndTag, Some(loc)));
            }

            self.emit_stack_entry(entry);
        }
    }
}
