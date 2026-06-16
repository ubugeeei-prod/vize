//! SSR result as a shared `AtelierOutput` finishing plate.
//!
//! Lets the SSR result speak the same finish-plate vocabulary as DOM and Vapor
//! (#1758), so SFC assembly can eventually slice SSR output through structured
//! sections instead of the bespoke `{ code, preamble }` pair.

use crate::codegen::SsrCodegenResult;
use vize_atelier_core::atelier_output::AtelierOutput;
use vize_carton::String;

impl SsrCodegenResult {
    /// Structure this result as an [`AtelierOutput`] finishing plate.
    ///
    /// SSR's `preamble` (helper imports) maps to imports and `code` (the
    /// `ssrRender` function) maps to functions. The owned `code`/`preamble`
    /// fields are unchanged — this is an additive structured view, not yet the
    /// assembly contract, so SSR output stays byte-equivalent.
    pub fn atelier_output(&self) -> AtelierOutput {
        AtelierOutput::new(
            self.preamble.clone(),
            String::default(),
            self.code.clone(),
            String::default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atelier_output_maps_preamble_to_imports_and_code_to_functions() {
        let result = SsrCodegenResult {
            code: String::from("function ssrRender() {}"),
            preamble: String::from("import { x } from \"vue\"\n"),
        };

        let output = result.atelier_output();
        assert_eq!(output.imports.as_str(), "import { x } from \"vue\"\n");
        assert_eq!(output.functions.as_str(), "function ssrRender() {}");
        assert!(output.hoists.is_empty());
        assert!(output.exports.is_empty());
    }
}
