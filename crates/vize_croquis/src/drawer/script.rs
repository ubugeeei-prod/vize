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
        if !self.options.analyze_script {
            return self;
        }

        self.script_drawn = true;

        // Use OXC-based parser for accurate AST drawing
        let result = profile!(
            "croquis.drawer.script_setup",
            crate::script_parser::parse_script_setup_with_generic(source, generic)
        );

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

        let result = profile!(
            "croquis.drawer.script_setup_program",
            crate::script_parser::analyze_script_setup_program(program, source, generic)
        );

        result.apply_to_croquis(&mut self.croquis);

        self
    }

    /// Draw non-script-setup (Options API) source code.
    pub fn draw_script_plain(&mut self, source: &str) -> &mut Self {
        if !self.options.analyze_script {
            return self;
        }

        self.script_drawn = true;

        // Use OXC-based parser for non-script-setup
        let result = profile!(
            "croquis.drawer.script_plain",
            crate::script_parser::parse_script_with_options(
                source,
                crate::script_parser::ScriptParserOptions {
                    options_api: self.options_api,
                    legacy_vue2: self.legacy_vue2,
                }
            )
        );

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
}
