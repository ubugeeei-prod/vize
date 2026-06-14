//! Internal Rendu plate vocabulary.
//!
//! Rendu is the rendered plate of an atelier: render-semantic operations first,
//! then output registration marks and optional source-map material. All views
//! borrow existing buffers so a tool can observe output shape without forcing
//! another render or flattening pass.

mod semantic;
mod walk;

pub use semantic::{
    RenduBlock, RenduCapabilities, RenduElementKind, RenduExprRef, RenduOp, RenduRoot, RenduSource,
    RenduSpan,
};
pub use walk::{RenduWalkSummary, summarize_rendu_ops, walk_rendu_ops};

use crate::source_atlas::SourceAtlasTarget;

/// Byte range in a flattened Rendu plate.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct RenduRange {
    pub start: usize,
    pub end: usize,
}

impl RenduRange {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub const fn empty(offset: usize) -> Self {
        Self {
            start: offset,
            end: offset,
        }
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// Coarse chunks owned by a rendered module.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RenduModuleSections {
    pub imports: RenduRange,
    pub hoists: RenduRange,
    pub functions: RenduRange,
    pub exports: RenduRange,
}

impl RenduModuleSections {
    pub const fn from_chunk_lengths(
        imports_len: usize,
        hoists_len: usize,
        functions_len: usize,
        exports_len: usize,
    ) -> Self {
        let imports = RenduRange::new(0, imports_len);
        let hoists = RenduRange::new(imports.end, imports.end + hoists_len);
        let functions_start = hoists.end + 1;
        let functions = RenduRange::new(functions_start, functions_start + functions_len);
        let exports_start = functions.end + 1;
        let exports = RenduRange::new(exports_start, exports_start + exports_len);

        Self {
            imports,
            hoists,
            functions,
            exports,
        }
    }
}

/// Fine sections within a target render function.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RenduRenderSections {
    pub imports: RenduRange,
    pub hoisted: RenduRange,
    pub assets: RenduRange,
    pub return_expr: RenduRange,
}

impl RenduRenderSections {
    pub const fn from_dom_codegen(
        imports_len: usize,
        preamble_len: usize,
        function_base_offset: usize,
        assets: (usize, usize),
        return_expr: (usize, usize),
    ) -> Self {
        Self {
            imports: RenduRange::new(0, imports_len),
            hoisted: if preamble_len > imports_len {
                // DOM codegen inserts one blank-line separator between helper
                // imports and hoists. Keep the hoisted section focused on
                // declarations so inline assembly receives only movable
                // template artifacts.
                RenduRange::new(imports_len + 1, preamble_len)
            } else {
                RenduRange::empty(preamble_len)
            },
            assets: RenduRange::new(
                function_base_offset + assets.0,
                function_base_offset + assets.1,
            ),
            return_expr: RenduRange::new(
                function_base_offset + return_expr.0,
                function_base_offset + return_expr.1,
            ),
        }
    }
}

/// Named chunks in a borrowed Rendu plate.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenduChunk {
    Imports,
    Hoists,
    Functions,
    Exports,
}

/// Borrowed view of a rendered output plate.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RenduPlate<'a> {
    pub target: SourceAtlasTarget,
    pub imports: &'a str,
    pub hoists: &'a str,
    pub functions: &'a str,
    pub exports: &'a str,
    pub module_sections: RenduModuleSections,
    pub render_sections: Option<RenduRenderSections>,
    pub source_map: Option<&'a str>,
}

impl<'a> RenduPlate<'a> {
    pub const fn new(
        target: SourceAtlasTarget,
        imports: &'a str,
        hoists: &'a str,
        functions: &'a str,
        exports: &'a str,
    ) -> Self {
        Self {
            target,
            imports,
            hoists,
            functions,
            exports,
            module_sections: RenduModuleSections::from_chunk_lengths(
                imports.len(),
                hoists.len(),
                functions.len(),
                exports.len(),
            ),
            render_sections: None,
            source_map: None,
        }
    }

    pub const fn with_render_sections(mut self, sections: Option<RenduRenderSections>) -> Self {
        self.render_sections = sections;
        self
    }

    pub const fn with_source_map(mut self, source_map: Option<&'a str>) -> Self {
        self.source_map = source_map;
        self
    }

    pub const fn has_source_map(self) -> bool {
        self.source_map.is_some()
    }

    pub const fn chunk(self, chunk: RenduChunk) -> &'a str {
        match chunk {
            RenduChunk::Imports => self.imports,
            RenduChunk::Hoists => self.hoists,
            RenduChunk::Functions => self.functions,
            RenduChunk::Exports => self.exports,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_sections_follow_output_module_flattening_marks() {
        let sections = RenduModuleSections::from_chunk_lengths(8, 5, 13, 7);

        assert_eq!(sections.imports, RenduRange::new(0, 8));
        assert_eq!(sections.hoists, RenduRange::new(8, 13));
        assert_eq!(sections.functions, RenduRange::new(14, 27));
        assert_eq!(sections.exports, RenduRange::new(28, 35));
    }

    #[test]
    fn plate_borrows_render_chunks_and_source_map() {
        let plate = RenduPlate::new(
            SourceAtlasTarget::Dom,
            "import { h } from \"vue\"\n",
            "const _hoisted_1 = null\n",
            "function render() {}",
            "export default _sfc_main\n",
        )
        .with_render_sections(Some(RenduRenderSections::from_dom_codegen(
            24,
            48,
            49,
            (2, 8),
            (12, 18),
        )))
        .with_source_map(Some("{\"version\":3}"));

        assert_eq!(plate.chunk(RenduChunk::Functions), "function render() {}");
        assert!(plate.has_source_map());
        assert_eq!(
            plate.render_sections.expect("render sections").assets,
            RenduRange::new(51, 57)
        );
    }
}
