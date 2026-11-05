//! Shared internal inputs for component prop value checks.
use super::{
    ComponentPropSource, FxHashSet, String, TemplateBindingAccess, VizeMapping,
    prop_name_source_range, prop_value_source_range,
};

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

impl super::ComponentPropSource<'_> {
    pub(crate) fn record_prop_projection(
        self,
        usage: &vize_croquis::croquis::ComponentUsage,
        prop: &vize_croquis::croquis::PassedProp,
        name: std::ops::Range<usize>,
        value: std::ops::Range<usize>,
    ) {
        self.record_prop_projection_inner(usage, prop, name, value, None);
    }

    pub(crate) fn record_prop_projection_with_receivers(
        self,
        usage: &vize_croquis::croquis::ComponentUsage,
        prop: &vize_croquis::croquis::PassedProp,
        name: std::ops::Range<usize>,
        value: std::ops::Range<usize>,
        generated: (&str, bool),
    ) {
        self.record_prop_projection_inner(usage, prop, name, value, Some(generated));
    }

    fn record_prop_projection_inner(
        self,
        usage: &vize_croquis::croquis::ComponentUsage,
        prop: &vize_croquis::croquis::PassedProp,
        name: std::ops::Range<usize>,
        value: std::ops::Range<usize>,
        generated: Option<(&str, bool)>,
    ) {
        use crate::virtual_ts::diagnostic_ownership::{InputOwner, InputRole};
        let Some(collector) = self.diagnostic_ownership else {
            return;
        };
        let ordinary_owner = InputOwner::for_prop(usage, prop);
        if ordinary_owner.is_none()
            && let Some((text, primary)) = generated
        {
            self.record_independent_receivers(usage, prop, text, value.clone(), primary);
        }
        let Some(owner) = ordinary_owner.or_else(|| {
            (self.expected_packet || self.inert)
                .then(|| InputOwner::for_expected_projection(usage, prop))
                .flatten()
        }) else {
            return;
        };
        let name_source = prop_name_source_range(self, prop);
        let value_source = prop_value_source_range(self, prop).or_else(|| name_source.clone());
        // Do not retain a RefCell borrow while another emitter runs.
        let mut ledger = collector.borrow_mut();
        // The props literal repeats a `v-model` target that its write-back
        // already reports; only the literal's key keeps its own diagnostics.
        let value_echo = !self.inert
            && ordinary_owner.is_some()
            && matches!(generated, Some((_, false)))
            && ledger.writes_back_model(owner);
        let mut record = |owner, generated, source, inference: bool| {
            if self.inert {
                ledger.record_inert(owner, generated, source);
            } else if ordinary_owner.is_some() {
                ledger.record(owner, generated, source, inference);
            } else {
                // The root relocation proof classifies the callback's creation.
                // Its contextual body still owns diagnostics; this record only
                // prevents an inference mapping from widening authored EXPECT.
                ledger.record_expect_geometry(owner, generated, source, self.inference_only);
            }
        };
        if let Some(source) = name_source {
            record(
                owner.with_role(InputRole::Name),
                name,
                source,
                self.inference_only,
            );
        }
        if let Some(source) = value_source {
            record(owner, value, source, self.inference_only || value_echo);
        }
    }

    /// Record the `strictVModel` write-back of `prop`'s bound target.
    pub(crate) fn record_model_write_back(
        self,
        usage: &vize_croquis::croquis::ComponentUsage,
        prop: &vize_croquis::croquis::PassedProp,
        target: std::ops::Range<usize>,
    ) {
        let (Some(collector), Some(owner), Some(source)) = (
            self.diagnostic_ownership,
            crate::virtual_ts::diagnostic_ownership::InputOwner::for_prop(usage, prop),
            prop_value_source_range(self, prop),
        ) else {
            return;
        };
        if !self.inference_only {
            collector
                .borrow_mut()
                .record_model_write_back(owner, target, source);
        }
    }

    /// Only byte-identical parser-owned receivers can share a diagnostic row.
    /// The contextual function and both of its whole contracts stay untouched.
    pub(crate) fn record_independent_receivers(
        self,
        usage: &vize_croquis::croquis::ComponentUsage,
        prop: &vize_croquis::croquis::PassedProp,
        generated: &str,
        range: std::ops::Range<usize>,
        primary: bool,
    ) {
        if self.expected_packet || self.inference_only || self.inert {
            return;
        }
        let (Some(collector), Some(authored), Some(value)) = (
            self.diagnostic_ownership,
            self.value_source_range(prop),
            generated.get(range.clone()),
        ) else {
            return;
        };
        collector
            .borrow_mut()
            .record_independent_receivers(usage, prop, value, range, authored, !primary);
    }
}
