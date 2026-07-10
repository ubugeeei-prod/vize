/// Byte range in a flattened Atelier artifact.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct AtelierRange {
    pub start: usize,
    pub end: usize,
}

impl AtelierRange {
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

/// Coarse chunks in a flattened Atelier module.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct AtelierModuleSections {
    pub imports: AtelierRange,
    pub hoists: AtelierRange,
    pub functions: AtelierRange,
    pub exports: AtelierRange,
}

impl AtelierModuleSections {
    pub const fn from_chunk_lengths(
        imports_len: usize,
        hoists_len: usize,
        functions_len: usize,
        exports_len: usize,
    ) -> Self {
        let imports = AtelierRange::new(0, imports_len);
        let hoists = AtelierRange::new(imports.end, imports.end + hoists_len);
        let functions_start = hoists.end + 1;
        let functions = AtelierRange::new(functions_start, functions_start + functions_len);
        let exports_start = functions.end + 1;
        let exports = AtelierRange::new(exports_start, exports_start + exports_len);
        Self {
            imports,
            hoists,
            functions,
            exports,
        }
    }
}

/// Fine sections within a target render function.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct AtelierRenderSections {
    pub imports: AtelierRange,
    pub hoisted: AtelierRange,
    pub assets: AtelierRange,
    pub return_expr: AtelierRange,
}

impl AtelierRenderSections {
    pub const fn from_dom_codegen(
        imports_len: usize,
        preamble_len: usize,
        function_base_offset: usize,
        assets: (usize, usize),
        return_expr: (usize, usize),
    ) -> Self {
        Self {
            imports: AtelierRange::new(0, imports_len),
            hoisted: if preamble_len > imports_len {
                AtelierRange::new(imports_len + 1, preamble_len)
            } else {
                AtelierRange::empty(preamble_len)
            },
            assets: AtelierRange::new(
                function_base_offset + assets.0,
                function_base_offset + assets.1,
            ),
            return_expr: AtelierRange::new(
                function_base_offset + return_expr.0,
                function_base_offset + return_expr.1,
            ),
        }
    }
}
