//! Semantic analysis helpers and element traversal methods.
//!
//! Provides convenience methods on `LintContext` for accessing semantic
//! analysis data from croquis, managing element scope, and reporting
//! diagnostics with various severity and help levels.

use crate::diagnostic::LintDiagnostic;
use vize_carton::CompactString;
use vize_relief::{ast::SourceLocation, BindingType};

use super::{state::ElementContext, LintContext};

impl<'a> LintContext<'a> {
    /// Report an error at a location.
    #[inline]
    pub fn error(&mut self, message: impl Into<CompactString>, loc: &SourceLocation) {
        self.report(LintDiagnostic::error(
            self.current_rule,
            message,
            loc.start.offset,
            loc.end.offset,
        ));
    }

    /// Report a warning at a location.
    #[inline]
    pub fn warn(&mut self, message: impl Into<CompactString>, loc: &SourceLocation) {
        self.report(LintDiagnostic::warn(
            self.current_rule,
            message,
            loc.start.offset,
            loc.end.offset,
        ));
    }

    /// Report an error with help message.
    #[inline]
    pub fn error_with_help(
        &mut self,
        message: impl Into<CompactString>,
        loc: &SourceLocation,
        help: impl Into<CompactString>,
    ) {
        let mut diag =
            LintDiagnostic::error(self.current_rule, message, loc.start.offset, loc.end.offset);
        let help_str: CompactString = help.into();
        if let Some(processed) = self.help_level.process(help_str.as_str()) {
            diag = diag.with_help(processed);
        }
        self.report(diag);
    }

    /// Report a warning with help message.
    #[inline]
    pub fn warn_with_help(
        &mut self,
        message: impl Into<CompactString>,
        loc: &SourceLocation,
        help: impl Into<CompactString>,
    ) {
        let mut diag =
            LintDiagnostic::warn(self.current_rule, message, loc.start.offset, loc.end.offset);
        let help_str: CompactString = help.into();
        if let Some(processed) = self.help_level.process(help_str.as_str()) {
            diag = diag.with_help(processed);
        }
        self.report(diag);
    }

    /// Report a diagnostic with related label.
    #[inline]
    pub fn error_with_label(
        &mut self,
        message: impl Into<CompactString>,
        loc: &SourceLocation,
        label_message: impl Into<CompactString>,
        label_loc: &SourceLocation,
    ) {
        self.report(
            LintDiagnostic::error(self.current_rule, message, loc.start.offset, loc.end.offset)
                .with_label(label_message, label_loc.start.offset, label_loc.end.offset),
        );
    }

    /// Get collected diagnostics.
    #[inline]
    pub fn into_diagnostics(self) -> Vec<LintDiagnostic> {
        self.diagnostics
    }

    /// Get reference to collected diagnostics.
    #[inline]
    pub fn diagnostics(&self) -> &[LintDiagnostic] {
        &self.diagnostics
    }

    // =========================================================================
    // Element stack management
    // =========================================================================

    /// Push an element onto the context stack.
    #[inline]
    pub fn push_element(&mut self, ctx: ElementContext) {
        // Add v-for vars to scope
        for var in &ctx.v_for_vars {
            self.scope_variables.insert(var.clone());
        }
        self.element_stack.push(ctx);
    }

    /// Pop an element from the context stack.
    #[inline]
    pub fn pop_element(&mut self) -> Option<ElementContext> {
        if let Some(ctx) = self.element_stack.pop() {
            // Remove v-for vars from scope
            for var in &ctx.v_for_vars {
                self.scope_variables.remove(var);
            }
            Some(ctx)
        } else {
            None
        }
    }

    /// Check if inside a v-for loop.
    #[inline]
    pub fn is_in_v_for(&self) -> bool {
        self.element_stack.iter().any(|e| e.has_v_for)
    }

    /// Get all v-for variables in current scope.
    #[inline]
    pub fn v_for_vars(&self) -> impl Iterator<Item = &str> {
        self.element_stack
            .iter()
            .flat_map(|e| e.v_for_vars.iter().map(|s| s.as_str()))
    }

    /// Check if a variable is defined by a parent v-for.
    #[inline]
    pub fn is_v_for_var(&self, name: &str) -> bool {
        self.scope_variables.contains(name)
    }

    /// Check if a variable is defined by a PARENT v-for (excluding current element).
    ///
    /// This is useful for shadow detection where we want to check if a variable
    /// in the current v-for shadows a variable from an outer scope.
    #[inline]
    pub fn is_parent_v_for_var(&self, name: &str) -> bool {
        // Check all elements except the last one (current element)
        if self.element_stack.len() < 2 {
            return false;
        }
        for elem in self.element_stack.iter().take(self.element_stack.len() - 1) {
            for var in &elem.v_for_vars {
                if var.as_str() == name {
                    return true;
                }
            }
        }
        false
    }

    /// Get current element context (top of stack).
    #[inline]
    pub fn current_element(&self) -> Option<&ElementContext> {
        self.element_stack.last()
    }

    /// Get parent element context.
    #[inline]
    pub fn parent_element(&self) -> Option<&ElementContext> {
        if self.element_stack.len() >= 2 {
            self.element_stack.get(self.element_stack.len() - 2)
        } else {
            None
        }
    }

    /// Check if any ancestor element matches the given predicate.
    ///
    /// Searches the element stack from bottom to top (excluding the current element).
    /// Useful for detecting nested interactive elements or content model violations.
    #[inline]
    pub fn has_ancestor(&self, predicate: impl Fn(&ElementContext) -> bool) -> bool {
        if self.element_stack.len() < 2 {
            return false;
        }
        self.element_stack
            .iter()
            .take(self.element_stack.len() - 1)
            .any(predicate)
    }

    /// Get the error count (cached, O(1)).
    #[inline]
    pub fn error_count(&self) -> usize {
        self.error_count
    }

    /// Get the warning count (cached, O(1)).
    #[inline]
    pub fn warning_count(&self) -> usize {
        self.warning_count
    }

    // =========================================================================
    // Semantic Analysis Helpers
    // =========================================================================
    // These methods leverage croquis Croquis when available.
    // They provide fallback behavior when analysis is not available.

    /// Check if a variable is defined (in any scope or script binding).
    ///
    /// Uses semantic analysis if available, otherwise falls back to
    /// v-for variable tracking only.
    #[inline]
    pub fn is_variable_defined(&self, name: &str) -> bool {
        // First check template-local scope (v-for variables)
        if self.is_v_for_var(name) {
            return true;
        }

        // Then check semantic analysis if available
        if let Some(analysis) = &self.analysis {
            return analysis.is_defined(name);
        }

        false
    }

    /// Get the binding type for a variable.
    ///
    /// Returns None if analysis is not available or variable is not found.
    #[inline]
    pub fn get_binding_type(&self, name: &str) -> Option<BindingType> {
        self.analysis.and_then(|a| a.get_binding_type(name))
    }

    /// Check if a name refers to a script-level binding.
    #[inline]
    pub fn has_script_binding(&self, name: &str) -> bool {
        self.analysis
            .map(|a| a.bindings.contains(name))
            .unwrap_or(false)
    }

    /// Check if a component is registered or imported.
    #[inline]
    pub fn is_component_registered(&self, name: &str) -> bool {
        self.analysis
            .map(|a| a.is_component_registered(name))
            .unwrap_or(false)
    }

    /// Check if a prop is defined via defineProps.
    #[inline]
    pub fn has_prop(&self, name: &str) -> bool {
        self.analysis
            .map(|a| a.macros.props().iter().any(|p| p.name.as_str() == name))
            .unwrap_or(false)
    }

    /// Check if an emit is defined via defineEmits.
    #[inline]
    pub fn has_emit(&self, name: &str) -> bool {
        self.analysis
            .map(|a| a.macros.emits().iter().any(|e| e.name.as_str() == name))
            .unwrap_or(false)
    }

    /// Check if a model is defined via defineModel.
    #[inline]
    pub fn has_model(&self, name: &str) -> bool {
        self.analysis
            .map(|a| a.macros.models().iter().any(|m| m.name.as_str() == name))
            .unwrap_or(false)
    }

    /// Check if the component uses async setup (top-level await).
    #[inline]
    pub fn is_async_setup(&self) -> bool {
        self.analysis.map(|a| a.is_async()).unwrap_or(false)
    }

    /// Get all props defined in the component.
    pub fn get_props(&self) -> Vec<&str> {
        self.analysis
            .map(|a| a.macros.props().iter().map(|p| p.name.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get all emits defined in the component.
    pub fn get_emits(&self) -> Vec<&str> {
        self.analysis
            .map(|a| a.macros.emits().iter().map(|e| e.name.as_str()).collect())
            .unwrap_or_default()
    }
}
