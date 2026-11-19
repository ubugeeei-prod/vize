//! Vue binding-kind and contract queries over facts observed in the File's walk.
//!
//! The setup and Options API observers already record these facts inside the
//! one L2 Program walk; the model only attaches them. A family the caller did
//! not observe is a typed refusal, never an empty answer, and a binding is
//! matched to its Vue row by its own declaration site, never by name.

use super::Model;
use crate::file::BindingRef;
use crate::framework::vue::{
    ComponentContract, ComponentOptions, MacroCall, OptionsFacts, SetupBinding, SetupFacts,
};
use vize_l0::Span;

/// A fact family a model can carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// `<script setup>` facts: bindings, macro calls and the contract.
    Setup,
    /// Options API facts of an ordinary `<script>`.
    Options,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VueQueryError {
    /// The caller did not attach this family to the model.
    NotObserved(Family),
    /// The binding belongs to another File.
    ForeignBinding,
    /// The binding is declared inside a `defineProps` destructure the setup
    /// facts leave unclassified.
    Unclassified(Span),
}

impl<'f, 'a> Model<'f, 'a> {
    /// Attach the `<script setup>` facts observed in this File's Program walk.
    #[must_use]
    pub const fn with_setup(mut self, facts: &'f SetupFacts) -> Self {
        self.setup = Some(facts);
        self
    }

    /// Attach the Options API facts observed in this File's Program walk.
    #[must_use]
    pub const fn with_options(mut self, facts: &'f OptionsFacts) -> Self {
        self.options = Some(facts);
        self
    }

    /// The attached `<script setup>` facts.
    pub fn setup_facts(self) -> Result<&'f SetupFacts, VueQueryError> {
        self.setup.ok_or(VueQueryError::NotObserved(Family::Setup))
    }

    /// The attached Options API facts.
    pub fn options_facts(self) -> Result<&'f OptionsFacts, VueQueryError> {
        self.options
            .ok_or(VueQueryError::NotObserved(Family::Options))
    }

    /// The `<script setup>` row of a File binding: its Vue binding kind and
    /// the reactivity API that initializes it.
    ///
    /// The row is the one whose identifier is the binding's own declaration
    /// site; a binding without such a row (a nested declaration, a parameter,
    /// a binding of another Program) has none. A declaration inside an
    /// unclassified `defineProps` destructure is refused.
    pub fn setup_binding(
        self,
        binding: BindingRef<'_, '_>,
    ) -> Result<Option<&'f SetupBinding>, VueQueryError> {
        let facts = self.setup_facts()?;
        if !core::ptr::eq(self.file, binding.file()) {
            return Err(VueQueryError::ForeignBinding);
        }
        let Some(site) = binding.declaration().map(|row| row.span) else {
            return Ok(None);
        };
        if let Some(pattern) = facts
            .unclassified()
            .iter()
            .find(|pattern| pattern.start <= site.start && site.end <= pattern.end)
        {
            return Err(VueQueryError::Unclassified(*pattern));
        }
        Ok(facts.bindings().iter().find(|row| row.name == site))
    }

    /// Top-level macro calls in source order.
    pub fn macro_calls(self) -> Result<&'f [MacroCall], VueQueryError> {
        Ok(self.setup_facts()?.macro_calls())
    }

    /// The component contract the setup macros declare.
    pub fn contract(self) -> Result<&'f ComponentContract, VueQueryError> {
        Ok(self.setup_facts()?.contract())
    }

    /// The Options API component, when the script exports one.
    pub fn component_options(self) -> Result<Option<&'f ComponentOptions>, VueQueryError> {
        Ok(self.options_facts()?.component())
    }
}

#[cfg(test)]
mod tests;
