//! One parse per SFC script block, shared by every rule and the Croquis
//! analysis that read it.

use oxc_allocator::Allocator;
use oxc_parser::ParserReturn;
use oxc_span::SourceType;
use vize_atelier_sfc::SfcDescriptor;
use vize_atelier_sfc::croquis::SfcScriptParses;

/// The descriptor's script parses, in the lint language.
pub(crate) type SfcScripts<'a> = SfcScriptParses<'a>;

/// Lint reads JSX in every block except `lang="ts"` (#9222).
fn lint_language(lang: Option<&str>) -> SourceType {
    crate::rules::script::script_source_type().with_jsx(matches!(
        lang,
        None | Some("js" | "javascript" | "jsx" | "tsx")
    ))
}

/// Each script block of `descriptor`, parsed in its lint language on first read.
pub(crate) fn lint_scripts<'a>(
    allocator: &'a Allocator,
    descriptor: &'a SfcDescriptor<'_>,
) -> SfcScripts<'a> {
    SfcScriptParses::new(allocator, descriptor, lint_language)
}

/// The shared parses, as a lint context holds them.
pub(crate) trait ScriptParses {
    /// The `<script>` block's parse.
    fn script(&self) -> Option<&ParserReturn<'_>>;
    /// The `<script setup>` block's parse.
    fn script_setup(&self) -> Option<&ParserReturn<'_>>;
}

impl ScriptParses for SfcScriptParses<'_> {
    fn script(&self) -> Option<&ParserReturn<'_>> {
        SfcScriptParses::script(self)
    }

    fn script_setup(&self) -> Option<&ParserReturn<'_>> {
        SfcScriptParses::script_setup(self)
    }
}
