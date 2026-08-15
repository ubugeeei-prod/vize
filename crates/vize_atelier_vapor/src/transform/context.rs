//! Transform context for tracking state during AST-to-IR transformation.

use crate::ir::{BlockIRNode, IREffect, OperationNode};
use vize_carton::{Allocator, FxHashMap, FxHashSet, String, Vec, interner::Interner};

/// Transform context
pub(crate) struct TransformContext<'a> {
    pub(crate) allocator: &'a Allocator,
    /// The source string node-loc spans index into, used to recover covered
    /// text from a `SourceLocation`.
    pub(crate) source: &'a str,
    /// Atoms for the names lowering computes rather than slices out of the
    /// source (camelized prop keys, builtin directive names).
    pub(crate) interner: Interner<'a>,
    temp_id: usize,
    pub(crate) templates: Vec<'a, &'a str>,
    pub(crate) element_template_map: FxHashMap<usize, usize>,
    pub(crate) standalone_text_elements: FxHashSet<usize>,
    non_reactive_scopes: usize,
    pub(crate) diagnostics: std::vec::Vec<String>,
}

impl<'a> TransformContext<'a> {
    pub(crate) fn new(allocator: &'a Allocator, source: &'a str) -> Self {
        Self {
            allocator,
            source,
            interner: Interner::new(allocator),
            temp_id: 0,
            templates: Vec::new_in(&allocator),
            element_template_map: FxHashMap::default(),
            standalone_text_elements: FxHashSet::default(),
            non_reactive_scopes: 0,
            diagnostics: std::vec::Vec::new(),
        }
    }

    pub(crate) fn next_id(&mut self) -> usize {
        let id = self.temp_id;
        self.temp_id += 1;
        id
    }

    pub(crate) fn add_template(&mut self, element_id: usize, template: String) -> usize {
        let template_index = self.templates.len();
        // Template strings are assembled per element and effectively unique,
        // so they are frozen with a single arena copy rather than interned.
        self.templates.push(self.allocator.alloc_str(&template));
        self.element_template_map.insert(element_id, template_index);
        template_index
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
