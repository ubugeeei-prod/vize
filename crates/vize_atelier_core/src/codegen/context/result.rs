//! The result types of a generated module, which both copies of this crate
//! share: the `composed` copy names this module from the plain source.

use vize_l0::String;

/// Byte offsets of the structural sections of a generated render module,
/// recorded at emission time.
///
/// SFC inline assembly needs the generated module split back into imports /
/// hoisted consts / asset preamble / render body. Recording the boundaries
/// while the code is written lets the caller slice the buffer directly
/// instead of re-scanning the output line by line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodegenSections {
    /// Byte length of the import statement section at the start of
    /// `preamble`. Hoisted declarations (when present) follow after a single
    /// `'\n'` separator.
    pub imports_len: usize,
    /// Byte range in `code` covering the component/directive resolution
    /// statements inside the render function (raw, including indentation).
    pub assets_start: usize,
    pub assets_end: usize,
    /// Byte range in `code` covering the root `return` expression (the bytes
    /// after `"return "` up to the closing brace line).
    pub return_expr_start: usize,
    pub return_expr_end: usize,
}

/// Code generation result
pub struct CodegenResult {
    /// Generated code
    pub code: String,
    /// Preamble (imports)
    pub preamble: String,
    /// Source map (JSON)
    pub map: Option<String>,
}

/// Code generation result with emission-recorded section boundaries.
///
/// This is a separate wrapper so the longstanding public [`CodegenResult`]
/// remains constructible with the same public fields.
pub struct CodegenResultWithSections {
    /// Generated code, preamble, and source map.
    pub result: CodegenResult,
    /// Section boundaries recorded during emission (`None` when codegen
    /// bailed out before producing a render function).
    pub sections: Option<CodegenSections>,
}

impl CodegenResultWithSections {
    /// Drop section metadata and keep the public codegen result.
    pub fn into_result(self) -> CodegenResult {
        self.result
    }
}
