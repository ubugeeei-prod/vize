use super::{
    NoAsyncInComputed, NoDeepDestructureInProps, NoDupeKeys, NoGetCurrentInstance,
    NoImportCompilerMacros, NoInternalImports, NoNextTick, NoOptionsApi, NoPageMetaRuntimeValues,
    NoReactiveDestructure, NoReservedIdentifiers, NoSideEffectsInComputed, NoTopLevelRefInScript,
    NoWithDefaults, PiniaPreferStoreToRefs, PreferComputed, PreferImportFromVue, PreferImportMeta,
    PreferRefOverReactive, PreferUseAttrs, PreferUseId, PreferUseSlots, PreferUseTemplateRef,
    RequireFunctionReturnType, RequireSymbolProvide, ScriptLintResult, ScriptLinter, ScriptRule,
    SfcScriptContext, VueRouterPreferNamedPush, VueTestUtilsNoHtmlSnapshot, script_source_type,
};
use memchr::memmem;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use vize_l0::profile;

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
                Box::new(PreferImportMeta),
                Box::new(NoPageMetaRuntimeValues),
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

    /// Lint a script block.
    ///
    /// AST-based rules share a **single** oxc parse of the source (one
    /// [`Allocator`] + one [`oxc_ast::ast::Program`]) via [`ScriptRule::check_program`],
    /// collapsing N redundant per-rule parses into one. Byte-only rules scan
    /// the raw source directly via [`ScriptRule::check`].
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
        // AST rules only run when parsing succeeded, as per-rule guards did.
        let program = parsed.as_ref().and_then(|parsed| {
            if parsed.panicked || !parsed.diagnostics.is_empty() {
                None
            } else {
                Some(&parsed.program)
            }
        });

        for rule in &self.rules {
            profile!("patina.script_linter.rule.check", {
                if rule.uses_ast() {
                    if let Some(program) = program {
                        // Standalone script linting has no SFC, so cross-block context is empty.
                        rule.check_program_with_sfc(
                            program,
                            source,
                            offset,
                            SfcScriptContext::default(),
                            &mut result,
                        );
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
