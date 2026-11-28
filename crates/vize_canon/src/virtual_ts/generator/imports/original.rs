//! Original L2 import selections reused by the complete Canon projection.
use oxc_span::SourceType;
use vize_croquis::Croquis;
use vize_l0::SourceBlock;
use vize_l1::embed::authored_program::AuthoredProgram;
use vize_l2::query::script::{DeclarationSyntaxError, ScriptDeclarationKind, ScriptDeclarations};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginalImportError {
    Grammar,
    Syntax(DeclarationSyntaxError),
    Source,
    StatementSelection,
}

#[derive(Debug)]
struct Local<'a> {
    authored: &'a str,
    type_only: bool,
}

#[derive(Debug)]
struct Statement<'a> {
    span: (u32, u32),
    locals: Vec<Local<'a>>,
}

/// Original import declarations with raw local spellings and syntactic type
/// status. Construction borrows the genuine original L1 Program through L2's
/// shared declaration query. These facts grant no resolved File admission.
#[derive(Debug)]
pub struct OriginalImportFacts<'a> {
    block: SourceBlock<'a>,
    statements: Vec<Statement<'a>>,
}

impl<'a> OriginalImportFacts<'a> {
    pub fn new(original: &AuthoredProgram<'a>) -> Result<Self, OriginalImportError> {
        // Every compatibility import slice is parsed with this exact grammar.
        if original.source_type() != SourceType::ts() {
            return Err(OriginalImportError::Grammar);
        }
        let query = ScriptDeclarations::new(original).map_err(OriginalImportError::Syntax)?;
        let block = query.block();
        let mut statements = Vec::new();
        let mut failed = false;
        query.visit_import_statements(|statement| {
            let mut locals = Vec::new();
            statement.visit_names(|local| {
                let ScriptDeclarationKind::Import { type_only } = local.kind else {
                    failed = true;
                    return;
                };
                if let Some(authored) = block
                    .root_source()
                    .get(local.span.start as usize..local.span.end as usize)
                {
                    locals.push(Local {
                        authored,
                        type_only,
                    });
                } else {
                    failed = true;
                }
            });
            statements.push(Statement {
                span: (
                    statement.span.start - block.start(),
                    statement.span.end - block.start(),
                ),
                locals,
            });
        });
        if failed {
            return Err(OriginalImportError::Source);
        }
        Ok(Self { block, statements })
    }

    pub(in crate::virtual_ts::generator) fn validate(
        &self,
        summary: &Croquis,
        script: Option<&str>,
    ) -> Result<(), OriginalImportError> {
        if !script.is_some_and(|script| core::ptr::eq(script, self.block.source())) {
            return Err(OriginalImportError::Source);
        }
        if summary
            .import_statements
            .iter()
            .any(|import| self.statement((import.start, import.end)).is_none())
        {
            return Err(OriginalImportError::StatementSelection);
        }
        Ok(())
    }

    fn statement(&self, span: (u32, u32)) -> Option<&Statement<'a>> {
        self.statements
            .binary_search_by_key(&span, |statement| statement.span)
            .ok()
            .map(|index| &self.statements[index])
    }

    fn names<'f>(
        &'f self,
        summary: &'f Croquis,
        type_only: bool,
    ) -> impl Iterator<Item = &'a str> + 'f {
        summary
            .import_statements
            .iter()
            .filter_map(|import| self.statement((import.start, import.end)))
            .flat_map(|statement| &statement.locals)
            .filter(move |local| local.type_only == type_only)
            .map(|local| local.authored)
    }

    pub(super) fn value_names<'f>(
        &'f self,
        summary: &'f Croquis,
    ) -> impl Iterator<Item = &'a str> + 'f {
        self.names(summary, false)
    }

    pub(super) fn type_names<'f>(
        &'f self,
        summary: &'f Croquis,
    ) -> impl Iterator<Item = &'a str> + 'f {
        self.names(summary, true)
    }
}

#[cfg(test)]
mod tests;
