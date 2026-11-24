//! Collect compatible top-level declarations and static imports in one walk.
use super::super::{BlockLines, SymbolKind, block_ranges, collect};
use super::{
    AuthoredProgram, DocumentSymbol, ScriptQueryProducts, SfcScriptBlock, Statement, tag_ranges,
};
use vize_l0::line_index::LineIndex;

impl ScriptQueryProducts {
    pub(crate) fn read_tool_editor_products(
        &mut self,
        name: &str,
        block: &SfcScriptBlock<'_>,
        prefix: &str,
        syntax: &AuthoredProgram<'_>,
    ) {
        let parsed = syntax.parsed();
        let mut lines = None;
        let mut children = Vec::new();
        let static_only = Self::static_specifiers_only(syntax);
        if static_only {
            // Requested modules count distinct static sources. Repeated imports
            // can still grow the vector through the ordinary push path. Reserve
            // the capacity that path's doubling would reach, so a later block
            // grows from it as the separate readers' vector does: an exact
            // reservation of 5 made the next push double it to 10.
            let sources = parsed.module_record.requested_modules.len();
            if sources > 4 {
                let len = self.module_specifiers.len();
                let target = len.saturating_add(sources).next_power_of_two();
                self.module_specifiers
                    .reserve_exact(target.saturating_sub(len));
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
                    let lines = lines.get_or_insert_with(|| {
                        let lines = BlockLines::new(prefix, &block.loc);
                        let loc = lines.relative(&block.loc);
                        (lines, loc)
                    });
                    collect(declaration, lines.0.index(), lines.1.start, &mut children);
                }
            }
        }
        let (range, selection_range) = match &lines {
            Some((lines, loc)) => block_ranges(lines.index(), loc, "script"),
            None => tag_ranges(block, prefix)
                .unwrap_or_else(|| block_ranges(&LineIndex::new(prefix), &block.loc, "script")),
        };
        let mut symbol = DocumentSymbol {
            name: name.into(),
            kind: SymbolKind::MODULE,
            tags: None,
            deprecated: None,
            range,
            selection_range,
            detail: block.lang.as_deref().map(Into::into),
            children: (!children.is_empty()).then_some(children),
        };
        if let Some((lines, _)) = &lines {
            lines.place(&mut symbol);
        }
        self.push_script_symbol(symbol);
    }
}
