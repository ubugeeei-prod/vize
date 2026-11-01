//! Shared internal inputs for component prop value checks.
use super::{ComponentPropSource, FxHashSet, String, TemplateBindingAccess, VizeMapping};

pub(crate) struct ComponentPropCheckContext<'a, 'b> {
    pub(crate) inference_prelude:
        Option<&'a std::cell::RefCell<crate::virtual_ts::scope::RootOwnerBuffer>>,
    pub(crate) ts: &'b mut String,
    pub(crate) mappings: &'b mut Vec<VizeMapping>,
    pub(crate) template_binding_access: &'a TemplateBindingAccess<'a>,
    pub(crate) source_context: ComponentPropSource<'a>,
    pub(crate) indent: &'b str,
    pub(crate) strict_v_model: bool,
    pub(crate) typed_fallthrough_values: bool,
    pub(crate) hoist_shared_preamble: bool,
    /// Supplemental checks never emit the legacy usage's callback indices.
    pub(crate) legacy_scalar_callback_props: Option<&'a FxHashSet<usize>>,
}

impl<'a, 'b> ComponentPropCheckContext<'a, 'b> {
    pub(crate) fn new(
        ts: &'b mut String,
        mappings: &'b mut Vec<VizeMapping>,
        template_binding_access: &'a TemplateBindingAccess<'a>,
        source_context: ComponentPropSource<'a>,
        indent: &'b str,
    ) -> Self {
        Self {
            inference_prelude: None,
            ts,
            mappings,
            template_binding_access,
            source_context,
            indent,
            strict_v_model: false,
            typed_fallthrough_values: false,
            hoist_shared_preamble: false,
            legacy_scalar_callback_props: None,
        }
    }

    /// Also check what a `v-model` writes back (`strictVModel`).
    pub(crate) fn strict_v_model(mut self, enabled: bool) -> Self {
        self.strict_v_model = enabled;
        self
    }

    /// Also validate values contributed by inherited fallthrough spread keys.
    pub(crate) fn typed_fallthrough_values(mut self, enabled: bool) -> Self {
        self.typed_fallthrough_values = enabled;
        self
    }

    /// Resolve batch-shared helpers through their collision-proof module.
    pub(crate) fn hoist_shared_preamble(mut self, enabled: bool) -> Self {
        self.hoist_shared_preamble = enabled;
        self
    }
}
