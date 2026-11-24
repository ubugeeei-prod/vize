//! Read the original file-rename visitor's literals from retained syntax.
use super::{ModuleSpecifierCollector, SourceType, Visit, script_source_type};
use crate::source_project::{ScriptModuleSpecifier, ScriptQueryProducts};
use memchr::memmem::Finder;
use std::sync::LazyLock;
use vize_l0::Span;
use vize_l1::embed::authored_program::AuthoredProgram;

mod authored;

// `memmem::find` builds a searcher per call once the haystack reaches 64 bytes,
// which costs more than visiting a small Program. Build each needle once.
static ESCAPE: LazyLock<Finder<'static>> = LazyLock::new(|| Finder::new(br"\u"));
static REQUIRE: LazyLock<Finder<'static>> = LazyLock::new(|| Finder::new(b"require"));

impl ScriptQueryProducts {
    pub(crate) fn script_type(lang: Option<&str>) -> SourceType {
        script_source_type(lang)
    }

    pub(crate) fn read_specifiers(&mut self, syntax: &AuthoredProgram<'_>) {
        let mut collector = ModuleSpecifierCollector::default();
        collector.visit_program(&syntax.parsed().program);
        let base = syntax.block().start();
        self.module_specifiers
            .extend(
                collector
                    .specifiers
                    .into_iter()
                    .map(|item| ScriptModuleSpecifier {
                        span: Span::new(base + item.start as u32, base + item.end as u32),
                        specifier: item.specifier,
                    }),
            );
    }
}

impl ScriptQueryProducts {
    /// Static module declarations are already ordered by the Program. A clean
    /// JavaScript tree can add nested literals only through dynamic import or
    /// a call to require. Escaped identifiers and recovered trees keep the visitor.
    #[cfg(feature = "experimental-checker-script-queries")]
    pub(crate) fn read_selected_specifiers(&mut self, syntax: &AuthoredProgram<'_>) {
        if !Self::static_specifiers_only(syntax) {
            self.read_specifiers(syntax);
            return;
        }
        let base = syntax.block().start();
        for statement in &syntax.parsed().program.body {
            self.read_static_specifier(base, statement);
        }
    }

    pub(crate) fn static_specifiers_only(syntax: &AuthoredProgram<'_>) -> bool {
        let parsed = syntax.parsed();
        let bytes = syntax.block().source().as_bytes();
        parsed.diagnostics.is_empty()
            && !parsed.panicked
            && (memchr::memchr(b'(', bytes).is_none()
                || (!syntax.source_type().is_typescript()
                    && parsed.module_record.dynamic_imports.is_empty()
                    && ESCAPE.find(bytes).is_none()
                    && REQUIRE.find(bytes).is_none()))
    }

    pub(crate) fn read_static_specifier(
        &mut self,
        base: u32,
        statement: &oxc_ast::ast::Statement<'_>,
    ) {
        let Some(literal) = statement
            .as_module_declaration()
            .and_then(|decl| decl.source())
        else {
            return;
        };
        self.module_specifiers.push(ScriptModuleSpecifier {
            span: Span::new(base + literal.span.start + 1, base + literal.span.end - 1),
            specifier: literal.value.as_str().into(),
        });
    }
}

#[cfg(test)]
impl ScriptQueryProducts {
    pub(crate) fn reference_specifiers(
        descriptor: &vize_croquis::sfc::SfcDescriptor<'_>,
    ) -> Vec<ScriptModuleSpecifier> {
        let mut selected = Vec::new();
        for block in [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
            .into_iter()
            .flatten()
        {
            let arena = super::Allocator::default();
            let parsed = super::Parser::new(
                &arena,
                &block.content,
                script_source_type(block.lang.as_deref()),
            )
            .parse();
            let mut collector = ModuleSpecifierCollector::default();
            collector.visit_program(&parsed.program);
            selected.extend(
                collector
                    .specifiers
                    .into_iter()
                    .map(|item| ScriptModuleSpecifier {
                        span: Span::new(
                            (block.loc.start + item.start) as u32,
                            (block.loc.start + item.end) as u32,
                        ),
                        specifier: item.specifier,
                    }),
            );
        }
        selected
    }
}
