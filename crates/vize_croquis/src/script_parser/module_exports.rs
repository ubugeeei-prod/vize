//! Optional export admission from the same original script parser result.

use oxc_ast::ast::Statement;
use oxc_parser::ParserReturn;
use oxc_syntax::module_record::ExportExportName;

use super::parse::{parse_script_plain_with_exports, parse_script_setup_with_exports};
use super::{ScriptParseResult, ScriptParserOptions};

#[derive(Default)]
pub(crate) struct ModuleExportCapture {
    observed: bool,
    unknown: bool,
}

impl ModuleExportCapture {
    #[inline]
    pub(crate) fn observe_if(capture: Option<&mut Self>, parsed: &ParserReturn<'_>) {
        if let Some(capture) = capture {
            capture.observe(parsed);
        }
    }

    #[inline]
    pub(crate) fn observe_statement_if(
        capture: Option<&mut Self>,
        statement: &Statement<'_>,
        setup: bool,
    ) {
        if let Some(capture) = capture {
            capture.observe_statement(statement);
            if setup
                && matches!(
                    statement,
                    Statement::ExportAllDeclaration(_)
                        | Statement::ExportNamedDeclaration(_)
                        | Statement::ExportDefaultDeclaration(_)
                )
            {
                capture.refuse();
            }
        }
    }

    pub(crate) fn observe(&mut self, parsed: &ParserReturn<'_>) {
        self.observed = true;
        self.unknown |= parsed.panicked
            || !parsed.diagnostics.is_empty()
            || !parsed.module_record.indirect_export_entries.is_empty()
            || !parsed.module_record.star_export_entries.is_empty()
            || parsed.module_record.local_export_entries.iter().any(|entry| {
                !matches!(&entry.export_name, ExportExportName::Default(_))
                    && !matches!(&entry.export_name, ExportExportName::Name(name) if name.name == "default")
            });
    }

    pub(crate) fn observe_statement(&mut self, statement: &Statement<'_>) {
        self.unknown |= matches!(
            statement,
            Statement::TSExportAssignment(_)
                | Statement::TSNamespaceExportDeclaration(_)
                | Statement::TSModuleDeclaration(_)
                | Statement::TSImportEqualsDeclaration(_)
        );
    }

    pub(crate) fn refuse(&mut self) {
        self.unknown = true;
    }

    pub(crate) fn known_empty(&self) -> bool {
        self.observed && !self.unknown
    }
}

pub(crate) fn parse_script_setup_for_unused<const BUILTIN_TYPES: bool>(
    source: &str,
    generic: Option<&str>,
    jsx: bool,
    unused: bool,
    skip_diagnostics: bool,
    occurrences: bool,
) -> ScriptParseResult {
    parse_script_setup_with_exports::<BUILTIN_TYPES>(
        source,
        generic,
        jsx,
        unused,
        skip_diagnostics,
        occurrences,
        None,
    )
}

pub(crate) fn parse_script_plain(
    source: &str,
    options: ScriptParserOptions,
    jsx: bool,
    skip_diagnostics: bool,
    occurrences: bool,
) -> ScriptParseResult {
    parse_script_plain_with_exports(source, options, jsx, skip_diagnostics, occurrences, None)
}
