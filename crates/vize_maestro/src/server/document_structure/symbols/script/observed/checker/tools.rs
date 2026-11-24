//! Collect compatible top-level declarations and static imports in one walk.
use super::super::{SymbolKind, block_ranges, collect};
use super::{
    AuthoredProgram, DocumentSymbol, LineIndex, ScriptQueryProducts, SfcScriptBlock, Statement,
    tag_ranges,
};

impl ScriptQueryProducts {
    pub(crate) fn read_tool_editor_products(
        &mut self,
        name: &str,
        block: &SfcScriptBlock<'_>,
        prefix: &str,
        syntax: &AuthoredProgram<'_>,
    ) {
        let parsed = syntax.parsed();
        let mut index = None;
        let mut children = Vec::new();
        let static_only = Self::static_specifiers_only(syntax);
        if static_only {
            // Requested modules count distinct static sources. Repeated imports
            // can still grow the vector through the ordinary push path.
            let sources = parsed.module_record.requested_modules.len();
            if sources > 4 {
                self.module_specifiers.reserve(sources);
            }
        } else {
            self.read_specifiers(syntax);
        }
        if !parsed.panicked && parsed.diagnostics.is_empty() {
            let start = syntax.block().start();
            for statement in &parsed.program.body {
                if static_only {
                    self.read_static_specifier(start, statement);
                }
                let declaration = match statement {
                    Statement::ExportNamedDeclaration(export) => export.declaration.as_ref(),
                    other => other.as_declaration(),
                };
                if let Some(declaration) = declaration {
                    let index = index.get_or_insert_with(|| LineIndex::new(prefix));
                    collect(declaration, index, block.loc.start, &mut children);
                }
            }
        }
        let (range, selection_range) = match index {
            Some(index) => block_ranges(&index, &block.loc, "script"),
            None => tag_ranges(block, prefix)
                .unwrap_or_else(|| block_ranges(&LineIndex::new(prefix), &block.loc, "script")),
        };
        self.push_script_symbol(DocumentSymbol {
            name: name.into(),
            kind: SymbolKind::MODULE,
            tags: None,
            deprecated: None,
            range,
            selection_range,
            detail: block.lang.as_deref().map(Into::into),
            children: (!children.is_empty()).then_some(children),
        });
    }
}
