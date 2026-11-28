//! Content mapper generation options and installed-helper capabilities.

use crate::virtual_ts::VirtualTsCheckOptions;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ContentMapperTransformOptions {
    pub(super) options_api: bool,
    pub(super) preserve_unused_diagnostics: bool,
    pub(super) check_javascript: bool,
    pub(super) external_component_attrs_module: bool,
    pub(super) external_component_type_helpers: bool,
    /// Hoist repeated helpers into the installed `vize/virtual-ts` module.
    pub(super) shared_virtual_module: bool,
    /// Load checker globals from the installed `vize/virtual-ts-checker`
    /// module while helpers stay embedded in each module.
    pub(in crate::batch::virtual_project) checker_globals_module: bool,
    /// Project only what diagnostics read, as `vize check` does.
    pub(super) diagnostics_only: bool,
    pub(in crate::batch::virtual_project) configured_global_template_ref:
        crate::virtual_ts::TemplateRefOwnership,
    pub(in crate::batch::virtual_project) global_template_ref_explicit: bool,
    /// The project's `vueCompilerOptions`, resolved exactly as `vize check`
    /// resolves them (`vue_check_options::resolve`, #11688).
    pub(in crate::batch::virtual_project) checks: VirtualTsCheckOptions,
}

impl Default for ContentMapperTransformOptions {
    fn default() -> Self {
        Self {
            options_api: true,
            preserve_unused_diagnostics: false,
            check_javascript: false,
            external_component_attrs_module: false,
            external_component_type_helpers: false,
            shared_virtual_module: false,
            checker_globals_module: false,
            diagnostics_only: false,
            configured_global_template_ref: crate::virtual_ts::TemplateRefOwnership {
                global: false,
                imports_alias: false,
            },
            global_template_ref_explicit: false,
            checks: VirtualTsCheckOptions::default(),
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

    /// Resolve hoisted helpers from the installed `vize/virtual-ts` module.
    #[must_use]
    pub const fn with_shared_virtual_module(mut self, enabled: bool) -> Self {
        self.shared_virtual_module = enabled;
        self
    }

    #[must_use]
    pub const fn shared_virtual_module(self) -> bool {
        self.shared_virtual_module
    }

    /// Load the checker's globals from the installed `vize/virtual-ts-checker`
    /// module through a type-only import that declaration emit drops, while
    /// every helper stays embedded in the module, where declaration emit keeps
    /// what the public surface names. Applies only without the shared module.
    #[must_use]
    pub const fn with_checker_globals_module(mut self, enabled: bool) -> Self {
        self.checker_globals_module = enabled;
        self
    }

    #[must_use]
    pub const fn checker_globals_module(self) -> bool {
        self.checker_globals_module
    }

    /// Drop editor-only navigation scaffolding for a host that never asks
    /// for hover, definition or rename: a command-line type-check.
    #[must_use]
    pub const fn with_diagnostics_only(mut self, enabled: bool) -> Self {
        self.diagnostics_only = enabled;
        self
    }

    /// Whether the projection carries only what diagnostics read.
    #[must_use]
    pub const fn diagnostics_only(self) -> bool {
        self.diagnostics_only
    }

    /// Admit a program declaration only when `vueCompilerOptions` did not
    /// explicitly decide which global names own template-ref inference.
    #[must_use]
    pub const fn with_ambient_global_template_ref(mut self, declared: bool) -> Self {
        if !self.global_template_ref_explicit {
            self.configured_global_template_ref.global = declared;
        }
        self
    }

    /// Admit a named import from `#imports` when that module re-exports Vue's
    /// own `useTemplateRef` (Nuxt's generated alias does).
    #[must_use]
    pub const fn with_imports_alias_template_ref(mut self, reexported: bool) -> Self {
        self.configured_global_template_ref.imports_alias = reexported;
        self
    }
}
