//! Constant-time original body lookup, never token discovery or another walk.

use super::{
    ModuleSourceError, ModuleSourceErrorKind, ModuleSourceKind, OriginalModuleSource,
    OriginalModuleSources, Row,
};
use crate::file::Namespace;
use oxc_ast::ast::{ImportOrExportKind, Statement};
use oxc_span::GetSpan;
use vize_l0::Span;

impl<'f, 'p, 'a> OriginalModuleSources<'f, 'p, 'a> {
    /// Visit each actual static source operand once, without allocation/sorting.
    /// Order is unspecified. A refusal requires discarding every visited prefix;
    /// only successful normal return grants a complete response for this unit.
    pub fn for_each(
        &self,
        mut visit: impl FnMut(OriginalModuleSource<'f, 'p, 'a>),
    ) -> Result<(), ModuleSourceError> {
        for row in self
            .file
            .imports()
            .iter()
            .filter(|row| row.unit == self.unit.id)
        {
            visit(self.read(Row::Import(row))?);
        }
        let mut previous = None;
        for row in self
            .file
            .exports()
            .iter()
            .filter(|row| row.unit == self.unit.id)
        {
            if row.source.is_none() {
                continue;
            }
            let source = self.read(Row::Export(row))?;
            if row.source_site != previous {
                visit(source);
            }
            previous = row.source_site;
        }
        Ok(())
    }

    fn read(&self, row: Row<'f>) -> Result<OriginalModuleSource<'f, 'p, 'a>, ModuleSourceError> {
        let (site, copied, row_span) = match row {
            Row::Import(row) => (row.source_site, Some(row.source.as_str()), row.span),
            Row::Export(row) => (row.source_site, row.source.as_deref(), row.span),
        };
        let reject = |kind| ModuleSourceError {
            unit: self.unit.id,
            span: row_span,
            kind,
        };
        let site = site.ok_or_else(|| reject(ModuleSourceErrorKind::OriginalSite))?;
        let statement = self
            .input
            .references
            .program()
            .body
            .get((site.statement.get() - 1) as usize)
            .ok_or_else(|| reject(ModuleSourceErrorKind::OriginalSite))?;
        let (literal, kind, namespace) = match (statement, &row) {
            (Statement::ImportDeclaration(original), Row::Import(_)) => (
                &original.source,
                ModuleSourceKind::Import,
                original.import_kind,
            ),
            (Statement::ExportNamedDeclaration(original), Row::Export(_)) => (
                original
                    .source
                    .as_ref()
                    .ok_or_else(|| reject(ModuleSourceErrorKind::OriginalSite))?,
                ModuleSourceKind::NamedReexport,
                original.export_kind,
            ),
            (Statement::ExportAllDeclaration(original), Row::Export(_)) => (
                &original.source,
                ModuleSourceKind::AllReexport,
                original.export_kind,
            ),
            _ => return Err(reject(ModuleSourceErrorKind::OriginalSite)),
        };
        if literal.lone_surrogates {
            return Err(reject(ModuleSourceErrorKind::LoneSurrogateRequest));
        }
        let raw = literal
            .raw
            .as_ref()
            .map(|raw| raw.as_str())
            .ok_or_else(|| reject(ModuleSourceErrorKind::OriginalSite))?;
        let quoted = self
            .authored(literal.span)
            .ok_or_else(|| reject(ModuleSourceErrorKind::OriginalSite))?;
        let original_raw = self
            .input
            .block
            .root_source()
            .get(quoted.start as usize..quoted.end as usize)
            .ok_or_else(|| reject(ModuleSourceErrorKind::OriginalSite))?;
        let quote = raw.as_bytes().first().copied();
        if !core::ptr::eq(raw, original_raw)
            || raw.len() < 2
            || !matches!(quote, Some(b'\'' | b'"'))
            || raw.as_bytes().last().copied() != quote
        {
            return Err(reject(ModuleSourceErrorKind::OriginalSite));
        }
        let content = Span::new(
            quoted
                .start
                .checked_add(1)
                .ok_or_else(|| reject(ModuleSourceErrorKind::OriginalSite))?,
            quoted
                .end
                .checked_sub(1)
                .ok_or_else(|| reject(ModuleSourceErrorKind::OriginalSite))?,
        );
        let authored = self
            .input
            .block
            .root_source()
            .get(content.start as usize..content.end as usize)
            .ok_or_else(|| reject(ModuleSourceErrorKind::OriginalSite))?;
        let copied = copied.ok_or_else(|| reject(ModuleSourceErrorKind::OriginalSite))?;
        if copied != literal.value.as_str() {
            return Err(reject(ModuleSourceErrorKind::OriginalSite));
        }
        let declaration = self
            .authored(statement.span())
            .ok_or_else(|| reject(ModuleSourceErrorKind::OriginalSite))?;
        Ok(OriginalModuleSource {
            file: self.file,
            row,
            literal,
            raw,
            authored,
            copied,
            quoted,
            content,
            declaration,
            kind,
            namespace: if namespace == ImportOrExportKind::Type {
                Namespace::Type
            } else {
                Namespace::Value
            },
        })
    }

    fn authored(&self, span: oxc_span::Span) -> Option<Span> {
        self.input
            .references
            .authored_span(Span::new(span.start, span.end))
    }
}
