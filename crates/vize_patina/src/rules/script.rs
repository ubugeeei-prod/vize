//! Script-level lint rules for Vue.js SFC files. These rules check
//! TypeScript/JavaScript code in `<script>` and `<script setup>` blocks and
//! are **opt-in**: enable them via `[rules.script]` in your configuration.

mod component_options_name_casing;
mod define_emits_declaration;
mod define_macros_order;
mod define_props_declaration;
mod define_props_destructuring;
mod no_arrow_functions_in_watch;
mod no_async_in_computed;
mod no_boolean_default;
mod no_deep_destructure_in_props;
mod no_deprecated_data_object_declaration;
mod no_deprecated_dollar_listeners_api;
mod no_deprecated_dollar_scopedslots_api;
mod no_deprecated_events_api;
mod no_deprecated_props_default_this;
mod no_dupe_keys;
mod no_export_in_script_setup;
mod no_get_current_instance;
mod no_import_compiler_macros;
mod no_internal_imports;
mod no_next_tick;
mod no_options_api;
mod no_potential_component_option_typo;
mod no_reactive_destructure;
mod no_required_prop_with_default;
mod no_reserved_identifiers;
mod no_reserved_keys;
mod no_side_effects_in_computed;
mod no_top_level_ref_in_script;
mod no_use_computed_property_like_method;
mod no_with_defaults;
mod pinia_prefer_store_to_refs;
mod prefer_computed;
mod prefer_define_options;
mod prefer_import_from_vue;
mod prefer_ref_over_reactive;
mod prefer_use_attrs;
mod prefer_use_id;
mod prefer_use_slots;
mod prefer_use_template_ref;
mod require_function_return_type;
mod require_prop_type_constructor;
mod require_symbol_provide;
mod require_typed_ref;
mod return_in_computed_property;
mod valid_define_options;
mod valid_next_tick;
mod vue_router_prefer_named_push;
mod vue_test_utils_no_html_snapshot;

use memchr::memmem;
use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::diagnostic::{LintDiagnostic, Severity};
use vize_carton::profile;

pub use component_options_name_casing::ComponentOptionsNameCasing;
pub use define_emits_declaration::DefineEmitsDeclaration;
pub use define_macros_order::DefineMacrosOrder;
pub use define_props_declaration::DefinePropsDeclaration;
pub use define_props_destructuring::DefinePropsDestructuring;
pub use no_arrow_functions_in_watch::NoArrowFunctionsInWatch;
pub use no_async_in_computed::NoAsyncInComputed;
pub use no_boolean_default::NoBooleanDefault;
pub use no_deep_destructure_in_props::NoDeepDestructureInProps;
pub use no_deprecated_data_object_declaration::NoDeprecatedDataObjectDeclaration;
pub use no_deprecated_dollar_listeners_api::NoDeprecatedDollarListenersApi;
pub use no_deprecated_dollar_scopedslots_api::NoDeprecatedDollarScopedSlotsApi;
pub use no_deprecated_events_api::NoDeprecatedEventsApi;
pub use no_deprecated_props_default_this::NoDeprecatedPropsDefaultThis;
pub use no_dupe_keys::NoDupeKeys;
pub use no_export_in_script_setup::NoExportInScriptSetup;
pub use no_get_current_instance::NoGetCurrentInstance;
pub use no_import_compiler_macros::NoImportCompilerMacros;
pub use no_internal_imports::NoInternalImports;
pub use no_next_tick::NoNextTick;
pub use no_options_api::NoOptionsApi;
pub use no_potential_component_option_typo::NoPotentialComponentOptionTypo;
pub use no_reactive_destructure::NoReactiveDestructure;
pub use no_required_prop_with_default::NoRequiredPropWithDefault;
pub use no_reserved_identifiers::NoReservedIdentifiers;
pub use no_reserved_keys::NoReservedKeys;
pub use no_side_effects_in_computed::NoSideEffectsInComputed;
pub use no_top_level_ref_in_script::NoTopLevelRefInScript;
pub use no_use_computed_property_like_method::NoUseComputedPropertyLikeMethod;
pub use no_with_defaults::NoWithDefaults;
pub use pinia_prefer_store_to_refs::PiniaPreferStoreToRefs;
pub use prefer_computed::PreferComputed;
pub use prefer_define_options::PreferDefineOptions;
pub use prefer_import_from_vue::PreferImportFromVue;
pub use prefer_ref_over_reactive::PreferRefOverReactive;
pub use prefer_use_attrs::PreferUseAttrs;
pub use prefer_use_id::PreferUseId;
pub use prefer_use_slots::PreferUseSlots;
pub use prefer_use_template_ref::PreferUseTemplateRef;
pub use require_function_return_type::RequireFunctionReturnType;
pub use require_prop_type_constructor::RequirePropTypeConstructor;
pub use require_symbol_provide::RequireSymbolProvide;
pub use require_typed_ref::RequireTypedRef;
pub use return_in_computed_property::ReturnInComputedProperty;
pub use valid_define_options::ValidDefineOptions;
pub use valid_next_tick::ValidNextTick;
pub use vue_router_prefer_named_push::VueRouterPreferNamedPush;
pub use vue_test_utils_no_html_snapshot::VueTestUtilsNoHtmlSnapshot;

/// Metadata for a script-level rule
pub struct ScriptRuleMeta {
    /// Rule name (e.g., "script/prefer-import-from-vue")
    pub name: &'static str,
    /// Rule description
    pub description: &'static str,
    /// Default severity (if enabled)
    pub default_severity: Severity,
}

/// Result of linting a script block
#[derive(Debug, Default)]
pub struct ScriptLintResult {
    pub diagnostics: Vec<LintDiagnostic>,
    pub error_count: usize,
    pub warning_count: usize,
}

impl ScriptLintResult {
    pub fn add_diagnostic(&mut self, diagnostic: LintDiagnostic) {
        match diagnostic.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning => self.warning_count += 1,
        }
        self.diagnostics.push(diagnostic);
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    pub fn has_warnings(&self) -> bool {
        self.warning_count > 0
    }
}

/// Resolve the [`SourceType`] used for parsing script blocks.
///
/// All built-in script rules parse with TypeScript semantics (`component.ts`),
/// which yields a non-JSX standard TypeScript source type. This is shared so a
/// single oxc parse can be reused across every rule.
#[inline]
pub(crate) fn script_source_type() -> SourceType {
    SourceType::from_path("component.ts").unwrap_or_else(|_| SourceType::ts())
}

/// Trait for script-level lint rules
pub trait ScriptRule: Send + Sync {
    /// Get rule metadata
    fn meta(&self) -> &'static ScriptRuleMeta;

    /// Check the script content.
    ///
    /// This is a thin wrapper that parses the source once and delegates to
    /// [`ScriptRule::check_program`]. Rules that rely on an oxc parse should
    /// override `check_program` instead so a shared parse can be reused by
    /// [`ScriptLinter::lint`]. Rules that operate purely on the raw bytes (no
    /// oxc AST) override `check` directly and leave `check_program` empty.
    ///
    /// * `source` - The script block content
    /// * `offset` - The offset of the script block in the original file
    /// * `result` - Accumulator for diagnostics
    fn check(&self, source: &str, offset: usize, result: &mut ScriptLintResult) {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, script_source_type()).parse();
        if parsed.panicked || !parsed.errors.is_empty() {
            return;
        }
        self.check_program(&parsed.program, source, offset, result);
    }

    /// Check an already-parsed script program. AST-based rules override this
    /// (and [`ScriptRule::uses_ast`]) to reuse the shared parse from
    /// [`ScriptLinter::lint`]; byte-only rules leave it as the default no-op.
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let _ = (program, source, offset, result);
    }

    /// Whether this rule consumes the oxc AST via [`ScriptRule::check_program`].
    /// AST rules return `true` to receive a shared parse from
    /// [`ScriptLinter::lint`]; byte-only rules leave it `false`.
    #[inline]
    fn uses_ast(&self) -> bool {
        false
    }

    /// Whether this rule runs against `<script setup>` blocks (defaults to `true`).
    #[inline]
    fn runs_on_script_setup(&self) -> bool {
        true
    }
}

/// Linter for script blocks
pub struct ScriptLinter {
    rules: Vec<Box<dyn ScriptRule>>,
}

impl ScriptLinter {
    /// Create a new script linter with default rules (all disabled by default)
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Create a script linter with all available rules enabled
    pub fn with_all_rules() -> Self {
        Self {
            rules: vec![
                Box::new(NoOptionsApi),
                Box::new(NoGetCurrentInstance),
                Box::new(NoNextTick),
                Box::new(PiniaPreferStoreToRefs),
                Box::new(VueRouterPreferNamedPush),
                Box::new(VueTestUtilsNoHtmlSnapshot),
                Box::new(PreferComputed),
                Box::new(NoAsyncInComputed),
                Box::new(NoReactiveDestructure),
                Box::new(NoTopLevelRefInScript),
                Box::new(PreferRefOverReactive),
                Box::new(PreferUseTemplateRef),
                Box::new(PreferUseSlots),
                Box::new(PreferUseAttrs),
                Box::new(PreferUseId),
                Box::new(PreferImportFromVue),
                Box::new(NoWithDefaults),
                Box::new(NoDeepDestructureInProps::default()),
                Box::new(NoDupeKeys),
                Box::new(NoSideEffectsInComputed),
                Box::new(NoInternalImports),
                Box::new(NoImportCompilerMacros),
                Box::new(NoReservedIdentifiers),
                Box::new(RequireSymbolProvide),
                Box::new(RequireFunctionReturnType),
            ],
        }
    }

    /// Create a script linter preloaded with Vapor-specific rules
    /// (`no-options-api`, `no-get-current-instance`, `no-next-tick`).
    pub fn with_vapor_rules() -> Self {
        Self {
            rules: vec![
                Box::new(NoOptionsApi),
                Box::new(NoGetCurrentInstance),
                Box::new(NoNextTick),
            ],
        }
    }

    /// Add a rule to the linter
    pub fn add_rule(&mut self, rule: Box<dyn ScriptRule>) {
        self.rules.push(rule);
    }

    /// Lint a script block
    ///
    /// AST-based rules share a **single** oxc parse of the source (one
    /// [`Allocator`] + one [`Program`]) via [`ScriptRule::check_program`],
    /// collapsing what used to be N redundant parses (one per rule) into one.
    /// Byte-only rules continue to scan the raw source directly via
    /// [`ScriptRule::check`].
    pub fn lint(&self, source: &str, offset: usize) -> ScriptLintResult {
        let mut result = ScriptLintResult::default();

        if self.rules.is_empty() {
            return result;
        }

        // Parse once only if at least one rule actually consumes the AST.
        let needs_ast = self.rules.iter().any(|rule| rule.uses_ast());
        let allocator;
        let parsed = if needs_ast {
            allocator = Allocator::default();
            Some(profile!(
                "patina.script_linter.parse",
                Parser::new(&allocator, source, script_source_type()).parse()
            ))
        } else {
            None
        };
        // AST rules only run when parsing succeeded (matching the previous
        // per-rule `parsed.panicked || !errors.is_empty()` early-return).
        let program = parsed.as_ref().and_then(|parsed| {
            if parsed.panicked || !parsed.errors.is_empty() {
                None
            } else {
                Some(&parsed.program)
            }
        });

        for rule in &self.rules {
            profile!("patina.script_linter.rule.check", {
                if rule.uses_ast() {
                    if let Some(program) = program {
                        rule.check_program(program, source, offset, &mut result);
                    }
                } else {
                    rule.check(source, offset, &mut result);
                }
            });
        }

        result
    }

    /// Check if a script contains Vue imports (SIMD-accelerated)
    #[inline]
    pub fn has_vue_imports(source: &str) -> bool {
        let bytes = source.as_bytes();
        memmem::find(bytes, b"from 'vue'").is_some()
            || memmem::find(bytes, b"from \"vue\"").is_some()
            || memmem::find(bytes, b"from '@vue/").is_some()
            || memmem::find(bytes, b"from \"@vue/").is_some()
    }
}

impl Default for ScriptLinter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "script_tests.rs"]
mod tests;
