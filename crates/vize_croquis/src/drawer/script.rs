use oxc_ast::ast::Program;
use vize_carton::profile;

use super::Drawer;

impl Drawer {
    /// Draw script setup source code.
    ///
    /// This uses OXC parser to extract:
    /// - defineProps/defineEmits/defineModel calls
    /// - Top-level bindings (const, let, function, class)
    /// - Import statements
    /// - Reactivity wrappers (ref, reactive, computed)
    ///
    /// Performance: OXC provides high-performance AST parsing with accurate span tracking.
    pub fn draw_script(&mut self, source: &str) -> &mut Self {
        self.draw_script_setup(source)
    }

    /// Draw script setup source code.
    pub fn draw_script_setup(&mut self, source: &str) -> &mut Self {
        self.draw_script_setup_with_generic(source, None)
    }

    /// Draw script setup source code with an optional generic parameter.
    ///
    /// `generic` is the value from `<script setup generic="T">` attribute, if present.
    pub fn draw_script_setup_with_generic(
        &mut self,
        source: &str,
        generic: Option<&str>,
    ) -> &mut Self {
        self.draw_script_setup_with_generic_and_jsx(source, generic, false)
    }

    /// Draw script setup source code with an optional generic parameter and JSX parser dialect.
    pub fn draw_script_setup_with_generic_and_jsx(
        &mut self,
        source: &str,
        generic: Option<&str>,
        jsx: bool,
    ) -> &mut Self {
        self.draw_script_setup_with_builtin::<false>(source, generic, jsx)
    }

    /// Collect source-owned primitive hints during the existing editor AST walk.
    pub fn draw_script_setup_with_builtin_types(&mut self, source: &str) -> &mut Self {
        self.draw_script_setup_with_builtin::<true>(source, None, false)
    }

    fn draw_script_setup_with_builtin<const BUILTIN_TYPES: bool>(
        &mut self,
        source: &str,
        generic: Option<&str>,
        jsx: bool,
    ) -> &mut Self {
        if !self.options.analyze_script {
            return self;
        }
        crate::script_parser::with_parsed_script_setup(source, jsx, |repaired| {
            self.draw_repaired_script_setup::<BUILTIN_TYPES>(source, generic, repaired);
        });
        self
    }

    /// Draw a script setup block from its authored parse, in the dialect the
    /// caller lints or compiles it as. A failed parse is repaired as
    /// [`crate::script_parser::parse_program_for_analysis`] repairs it.
    pub fn draw_script_setup_parsed(
        &mut self,
        source: &str,
        generic: Option<&str>,
        parsed: &oxc_parser::ParserReturn<'_>,
    ) -> &mut Self {
        let allocator = oxc_allocator::Allocator::default();
        let recovered =
            crate::script_parser::recover_program_for_analysis(&allocator, source, parsed);
        self.draw_repaired_script_setup::<false>(
            source,
            generic,
            recovered.as_ref().unwrap_or(parsed),
        )
    }

    fn draw_repaired_script_setup<const BUILTIN_TYPES: bool>(
        &mut self,
        source: &str,
        generic: Option<&str>,
        repaired: &oxc_parser::ParserReturn<'_>,
    ) -> &mut Self {
        if !self.options.analyze_script {
            return self;
        }

        self.script_drawn = true;

        let result = profile!(
            "croquis.drawer.script_setup",
            crate::script_parser::analyze_repaired_script_setup::<BUILTIN_TYPES>(
                repaired,
                source,
                generic,
                self.track_unused_bindings,
                !self.options.detect_undefined,
                self.occurrence_capture.is_some(),
                self.module_export_capture.as_mut(),
                false,
            )
        );

        let mut result = result;
        self.take_script_occurrences(&mut result, source);
        result.apply_to_croquis(&mut self.croquis);

        self
    }

    /// Draw an already-parsed script setup program.
    ///
    /// This is the parse-free equivalent of [`Self::draw_script_setup_with_generic`]
    /// for callers that already parsed the source with a dialect Croquis should
    /// not reparse, such as JSX/TSX.
    pub fn draw_script_setup_program(
        &mut self,
        program: &Program<'_>,
        source: &str,
        generic: Option<&str>,
    ) -> &mut Self {
        if !self.options.analyze_script {
            return self;
        }

        self.script_drawn = true;

        // A caller-owned Program does not include original parser admission.
        if let Some(capture) = self.module_export_capture.as_mut() {
            capture.refuse();
        }

        let result = profile!(
            "croquis.drawer.script_setup_program",
            crate::script_parser::analyze_script_setup_program_demand::<false>(
                program,
                source,
                generic,
                !self.options.detect_undefined,
                self.occurrence_capture.is_some(),
                None,
                false,
            )
        );

        let mut result = result;
        if self.track_unused_bindings {
            result.unused_bindings =
                crate::script_parser::unused_setup_bindings(program, &result, generic);
        }
        self.take_script_occurrences(&mut result, source);
        result.apply_to_croquis(&mut self.croquis);

        self
    }

    /// Draw non-script-setup (Options API) source code.
    pub fn draw_script_plain(&mut self, source: &str) -> &mut Self {
        self.draw_script_plain_with_jsx(source, false)
    }

    /// Draw non-script-setup (Options API) source code with an explicit JSX
    /// parser dialect.
    pub fn draw_script_plain_with_jsx(&mut self, source: &str, jsx: bool) -> &mut Self {
        if !self.options.analyze_script {
            return self;
        }
        let path = if jsx { "script.tsx" } else { "script.ts" };
        let source_type = oxc_span::SourceType::from_path(path).unwrap_or_default();
        crate::script_parser::with_parsed_script_plain(source, source_type, |repaired| {
            self.draw_repaired_script_plain(source, repaired);
        });
        self
    }

    /// Draw a non-script-setup block from its authored parse, in the dialect
    /// the caller lints or compiles it as.
    pub fn draw_script_plain_parsed(
        &mut self,
        source: &str,
        parsed: &oxc_parser::ParserReturn<'_>,
    ) -> &mut Self {
        let allocator = oxc_allocator::Allocator::default();
        let recovered =
            crate::script_parser::recover_program_for_analysis(&allocator, source, parsed);
        self.draw_repaired_script_plain(source, recovered.as_ref().unwrap_or(parsed))
    }

    fn draw_repaired_script_plain(
        &mut self,
        source: &str,
        repaired: &oxc_parser::ParserReturn<'_>,
    ) -> &mut Self {
        if !self.options.analyze_script {
            return self;
        }

        self.script_drawn = true;

        let result = profile!(
            "croquis.drawer.script_plain",
            crate::script_parser::analyze_repaired_script_plain(
                repaired,
                source,
                crate::script_parser::ScriptParserOptions {
                    options_api: self.options_api,
                    legacy_vue2: self.legacy_vue2,
                },
                !self.options.detect_undefined,
                self.occurrence_capture.is_some(),
                self.track_unused_bindings,
                self.module_export_capture.as_mut(),
            )
        );

        let mut result = result;
        self.take_script_occurrences(&mut result, source);
        result.apply_to_croquis(&mut self.croquis);

        self
    }
}

impl Drawer {
    /// Compatibility wrapper for the old Analyzer naming.
    #[inline]
    pub fn analyze_script(&mut self, source: &str) -> &mut Self {
        self.draw_script(source)
    }

    /// Compatibility wrapper for the old Analyzer naming.
    #[inline]
    pub fn analyze_script_setup(&mut self, source: &str) -> &mut Self {
        self.draw_script_setup(source)
    }

    /// Compatibility wrapper for the old Analyzer naming.
    #[inline]
    pub fn analyze_script_setup_with_generic(
        &mut self,
        source: &str,
        generic: Option<&str>,
    ) -> &mut Self {
        self.draw_script_setup_with_generic(source, generic)
    }

    /// Compatibility wrapper for callers that already know script setup needs JSX.
    #[inline]
    pub fn analyze_script_setup_with_generic_jsx(
        &mut self,
        source: &str,
        generic: Option<&str>,
    ) -> &mut Self {
        self.draw_script_setup_with_generic_and_jsx(source, generic, true)
    }

    /// Compatibility wrapper for the old Analyzer naming.
    #[inline]
    pub fn analyze_script_setup_program(
        &mut self,
        program: &Program<'_>,
        source: &str,
        generic: Option<&str>,
    ) -> &mut Self {
        self.draw_script_setup_program(program, source, generic)
    }

    /// Compatibility wrapper for the old Analyzer naming.
    #[inline]
    pub fn analyze_script_plain(&mut self, source: &str) -> &mut Self {
        self.draw_script_plain(source)
    }

    /// Compatibility wrapper for callers that already know the plain script
    /// should be parsed with JSX enabled.
    #[inline]
    pub fn analyze_script_plain_jsx(&mut self, source: &str) -> &mut Self {
        self.draw_script_plain_with_jsx(source, true)
    }
}
