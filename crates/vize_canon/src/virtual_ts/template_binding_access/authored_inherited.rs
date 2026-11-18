//! The authored template instance of a component that extends or mixes in
//! an SFC (#13270); see [`crate::virtual_ts::inherited_members`].

use super::TemplateBindingAccess;
use crate::virtual_ts::inherited_members::{
    AUTHORED_INSTANCE, INHERITED_AUTHORED_INSTANCE, authored_template_inherited,
    emit_authored_template_instance,
};

impl TemplateBindingAccess<'_> {
    /// Record the SFC `mixins:` / `extends:` members of an authored-instance
    /// template's `script`.
    pub(super) fn with_authored_inherited(mut self, script: Option<&str>) -> Self {
        self.authored_inherited = script
            .and_then(authored_template_inherited)
            .map(|members| std::rc::Rc::from(members.as_str()));
        self
    }

    /// Declare the inherited authored instance before any receiver reads it.
    pub(super) fn emit_authored_inherited(&self, output: &mut crate::virtual_ts::TextBuf) {
        if let Some(members) = self.authored_inherited.as_deref() {
            emit_authored_template_instance(output, members);
        }
    }

    /// The type an authored template instance receiver is declared with.
    pub(crate) fn authored_instance_type(&self) -> &'static str {
        if self.authored_inherited.is_some() {
            INHERITED_AUTHORED_INSTANCE
        } else {
            AUTHORED_INSTANCE
        }
    }
}
