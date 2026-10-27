//! Content mapper generation options and installed-helper capabilities.

use vize_l0::config::VueVersion;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ContentMapperTransformOptions {
    pub(super) options_api: bool,
    pub(super) preserve_unused_diagnostics: bool,
    pub(super) check_javascript: bool,
    pub(super) external_component_attrs_module: bool,
    pub(super) external_component_type_helpers: bool,
    pub(in crate::batch::virtual_project) check_unknown_components: bool,
    pub(in crate::batch::virtual_project) check_unknown_props: bool,
    pub(in crate::batch::virtual_project) check_unknown_fallthrough_props: bool,
    pub(in crate::batch::virtual_project) check_unknown_directives: bool,
    pub(in crate::batch::virtual_project) strict_css_modules: bool,
    /// Resolved `vueCompilerOptions.target`. `None` leaves intrinsic names checked.
    pub(in crate::batch::virtual_project) vue_compiler_target: Option<VueVersion>,
}

impl Default for ContentMapperTransformOptions {
    fn default() -> Self {
        Self {
            options_api: true,
            preserve_unused_diagnostics: false,
            check_javascript: false,
            external_component_attrs_module: false,
            external_component_type_helpers: false,
            check_unknown_components: false,
            check_unknown_props: true,
            check_unknown_fallthrough_props: false,
            check_unknown_directives: false,
            strict_css_modules: false,
            vue_compiler_target: None,
        }
    }
}

impl ContentMapperTransformOptions {
    /// Enable or disable Vue Options API instance bindings.
    #[must_use]
    pub const fn with_options_api(mut self, enabled: bool) -> Self {
        self.options_api = enabled;
        self
    }

    /// Whether Vue Options API instance bindings are enabled.
    #[must_use]
    pub const fn options_api(self) -> bool {
        self.options_api
    }

    /// Preserve diagnostics for user-authored unused locals.
    #[must_use]
    pub const fn with_preserve_unused_diagnostics(mut self, enabled: bool) -> Self {
        self.preserve_unused_diagnostics = enabled;
        self
    }

    /// Check JavaScript SFC script blocks as TypeScript input.
    #[must_use]
    pub const fn with_check_javascript(mut self, enabled: bool) -> Self {
        self.check_javascript = enabled;
        self
    }

    /// Import shared component-attribute aliases from the installed `vize`
    /// package instead of embedding them in every virtual module.
    #[must_use]
    pub const fn with_external_component_attrs_module(mut self, enabled: bool) -> Self {
        self.external_component_attrs_module = enabled;
        self
    }

    /// Keep component-input aliases embedded when an older installed package
    /// exposes only the shared attribute helpers.
    #[must_use]
    pub const fn with_external_component_type_helpers(mut self, enabled: bool) -> Self {
        self.external_component_type_helpers = enabled;
        self
    }
}
