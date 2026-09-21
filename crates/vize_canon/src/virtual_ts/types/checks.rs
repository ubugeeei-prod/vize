//! Execution switches and resolved Vue compiler options.

/// `resolveStyleClassNames`: which `<style>` blocks contribute class names to
/// the `__VLS_StyleScopedClasses` type a template may reference. Vue Language
/// Tools defaults to the scoped blocks; `true` takes every block.
#[cfg(feature = "native")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResolveStyleClassNames {
    None,
    Scoped,
    All,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct VirtualTsCheckOptions {
    pub(crate) check_props: bool,
    pub(crate) check_template_bindings: bool,
    pub(crate) check_emits: bool,
    pub(crate) check_unknown_props: bool,
    pub(crate) check_unknown_components: bool,
    pub(crate) check_unknown_events: bool,
    /// Check the value a component `v-model` writes back (`strictVModel`).
    pub(crate) strict_v_model: bool,
    /// The component is compiled in Vapor mode (`vapor`).
    pub(crate) vapor: bool,
    pub(crate) infer_component_dollar_el: bool,
    pub(crate) infer_template_dollar_el: bool,
    pub(crate) infer_template_dollar_slots: bool,
    pub(crate) infer_template_dollar_attrs: bool,
    pub(crate) fallthrough_attributes: bool,
    /// `checkRequiredFallthroughAttributes`: a root's required props that the
    /// component does not bind itself become required props of the component,
    /// and the root usage stops reporting them as missing.
    pub(crate) check_required_fallthrough_attributes: bool,
    #[cfg(feature = "native")]
    pub(crate) resolve_style_class_names: ResolveStyleClassNames,
    /// `resolveStyleImports`: a CSS module's `src` and `@import` targets
    /// contribute their default export's class names to the module type.
    #[cfg(feature = "native")]
    pub(crate) resolve_style_imports: bool,
    pub(crate) jsx_slots: bool,
    #[cfg(feature = "native")]
    pub(crate) strict_css_modules: bool,
}

impl VirtualTsCheckOptions {
    pub(crate) fn any_enabled(self) -> bool {
        self.check_props || self.check_template_bindings || self.check_emits
    }

    pub(crate) fn check_event_handlers(self) -> bool {
        self.check_emits || self.check_template_bindings
    }
}

impl Default for VirtualTsCheckOptions {
    fn default() -> Self {
        Self {
            check_props: true,
            check_template_bindings: true,
            check_emits: true,
            check_unknown_props: true,
            check_unknown_components: false,
            check_unknown_events: false,
            strict_v_model: false,
            vapor: false,
            infer_component_dollar_el: false,
            infer_template_dollar_el: false,
            infer_template_dollar_slots: false,
            infer_template_dollar_attrs: false,
            fallthrough_attributes: false,
            check_required_fallthrough_attributes: false,
            #[cfg(feature = "native")]
            resolve_style_class_names: ResolveStyleClassNames::Scoped,
            #[cfg(feature = "native")]
            resolve_style_imports: false,
            jsx_slots: false,
            #[cfg(feature = "native")]
            strict_css_modules: false,
        }
    }
}
