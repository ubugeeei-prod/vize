//! Execution switches and resolved Vue compiler options.

#[derive(Debug, Clone, Copy)]
pub(crate) struct VirtualTsCheckOptions {
    pub(crate) check_props: bool,
    pub(crate) check_template_bindings: bool,
    pub(crate) check_emits: bool,
    pub(crate) check_unknown_props: bool,
    pub(crate) infer_component_dollar_el: bool,
    pub(crate) infer_template_dollar_el: bool,
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
            infer_component_dollar_el: false,
            infer_template_dollar_el: false,
            jsx_slots: false,
            #[cfg(feature = "native")]
            strict_css_modules: false,
        }
    }
}
