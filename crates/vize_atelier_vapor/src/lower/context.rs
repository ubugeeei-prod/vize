//! Transform context for tracking state during AST-to-IR transformation.

use crate::ir::{BlockIRNode, IREffect, OperationNode};
use vize_atelier_core::{ElementNode, TextNode, codegen::spanned::SpannedText};
use vize_carton::{Allocator, FxHashMap, FxHashSet, String, Vec, interner::Interner};

/// Template anchors, collected only for map-requesting compiles (P3-9).
pub(crate) use crate::generate::spans::TemplateSpans;

/// Transform context
pub(crate) struct TransformContext<'a> {
    pub(crate) allocator: &'a Allocator,
    /// The source string node-loc spans index into, used to recover covered
    /// text from a `SourceLocation`.
    pub(crate) source: &'a str,
    pub(crate) scope_id: Option<String>,
    /// Atoms for the names lowering computes rather than slices out of the
    /// source (camelized prop keys, builtin directive names).
    pub(crate) interner: Interner<'a>,
    temp_id: usize,
    pub(crate) templates: Vec<'a, &'a str>,
    pub(crate) element_template_map: FxHashMap<usize, usize>,
    pub(crate) standalone_text_elements: FxHashSet<usize>,
    non_reactive_scopes: usize,
    pub(crate) diagnostics: std::vec::Vec<String>,
    /// `Some` when a source map is requested.
    pub(crate) template_spans: Option<TemplateSpans>,
}

impl<'a> TransformContext<'a> {
    pub(crate) fn new(allocator: &'a Allocator, source: &'a str) -> Self {
        Self {
            allocator,
            source,
            scope_id: None,
            interner: Interner::new(allocator),
            temp_id: 0,
            templates: Vec::new_in(&allocator),
            element_template_map: FxHashMap::default(),
            standalone_text_elements: FxHashSet::default(),
            non_reactive_scopes: 0,
            diagnostics: std::vec::Vec::new(),
            template_spans: None,
        }
    }

    pub(crate) fn next_id(&mut self) -> usize {
        let id = self.temp_id;
        self.temp_id += 1;
        id
    }

    pub(crate) fn add_template(
        &mut self,
        element_id: usize,
        template: impl Into<SpannedText>,
    ) -> usize {
        let template: SpannedText = template.into();
        let template_index = self.templates.len();
        if let Some(spans) = self.template_spans.as_mut()
            && !template.anchors().is_empty()
        {
            spans.insert(template_index, template.anchors().to_vec());
        }
        // Template strings are assembled per element and effectively unique,
        // so they are frozen with a single arena copy rather than interned.
        self.templates
            .push(self.allocator.alloc_str(template.as_str()));
        self.element_template_map.insert(element_id, template_index);
        template_index
    }

    /// An element's template string, anchored when a map is requested.
    pub(crate) fn element_template(&self, el: &ElementNode<'_>) -> SpannedText {
        let scope_id = self.scope_id.as_deref();
        if self.template_spans.is_some() {
            super::element::template::generate_element_template_spanned(el, scope_id)
        } else {
            super::element::template::generate_element_template(el, scope_id).into()
        }
    }

    /// A standalone text node's template string, anchored when requested.
    pub(crate) fn text_template(&self, text: &TextNode<'_>) -> SpannedText {
        match self.template_spans {
            Some(_) => SpannedText::mapped(text.content, text.loc.span.start),
            None => SpannedText::plain(text.content),
        }
    }

    pub(crate) fn enter_non_reactive_scope(&mut self) {
        self.non_reactive_scopes += 1;
    }

    pub(crate) fn exit_non_reactive_scope(&mut self) {
        self.non_reactive_scopes = self.non_reactive_scopes.saturating_sub(1);
    }

    pub(crate) fn is_non_reactive(&self) -> bool {
        self.non_reactive_scopes > 0
    }

    pub(crate) fn push_dynamic_operation(
        &mut self,
        block: &mut BlockIRNode<'a>,
        operation: OperationNode<'a>,
    ) {
        if self.is_non_reactive() {
            block.operation.push(operation);
            return;
        }

        let mut effect_ops = Vec::new_in(&self.allocator);
        effect_ops.push(operation);
        block.effect.push(IREffect {
            operations: effect_ops,
        });
    }

    pub(crate) fn push_diagnostic(&mut self, message: impl Into<String>) {
        self.diagnostics.push(message.into());
    }
}
