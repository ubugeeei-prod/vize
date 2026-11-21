//! Read the original file-rename visitor's literals from retained syntax.
use super::{ModuleSpecifierCollector, SourceType, Visit, script_source_type};
use crate::source_project::{ScriptModuleSpecifier, ScriptQueryProducts};
use vize_l0::Span;
use vize_l1::embed::authored_program::AuthoredProgram;

impl ScriptQueryProducts {
    pub(crate) fn script_type(lang: Option<&str>) -> SourceType {
        script_source_type(lang)
    }

    pub(crate) fn read_specifiers(&mut self, syntax: &AuthoredProgram<'_>) {
        let mut collector = ModuleSpecifierCollector::default();
        collector.visit_program(&syntax.parsed().program);
        let base = syntax.block().span().start;
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
