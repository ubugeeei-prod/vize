use super::{Cow, TemplateBindingAccess};

impl TemplateBindingAccess<'_> {
    /// Contextual keywords are valid JS identifiers: their value references
    /// belong to the semantic owner, including authored function scopes.
    pub(crate) fn contextual_reserved_bindings(&self) -> TemplateBindingAccess<'_> {
        TemplateBindingAccess {
            names: Cow::Owned(
                self.names
                    .iter()
                    .filter(|(name, _)| matches!(name.as_str(), "as" | "from" | "of"))
                    .map(|(name, receiver)| (name.clone(), *receiver))
                    .collect(),
            ),
            lexical: Cow::Borrowed(self.lexical.as_ref()),
            instance: None,
            scope_id: self.scope_id,
        }
    }
}
