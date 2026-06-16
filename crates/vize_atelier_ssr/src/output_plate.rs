//! SSR result as a shared `AtelierOutput` finishing plate.
//!
//! Lets the SSR result speak the same finish-plate vocabulary as DOM and Vapor
//! (#1758): SFC assembly builds its output module from these structured sections
//! (via [`into_atelier_output`](SsrCodegenResult::into_atelier_output)) instead
//! of the bespoke `{ code, preamble }` pair.

use crate::codegen::SsrCodegenResult;
use vize_atelier_core::atelier_output::AtelierOutput;
use vize_carton::String;

impl SsrCodegenResult {
    /// Structure this result as an [`AtelierOutput`] finishing plate.
    ///
    /// SSR's `preamble` (helper imports) maps to imports and `code` (the
    /// `ssrRender` function) maps to functions. This borrowed view clones for
    /// inspection; [`into_atelier_output`](Self::into_atelier_output) is the
    /// consuming form SFC assembly uses.
    pub fn atelier_output(&self) -> AtelierOutput {
        AtelierOutput::new(
            self.preamble.clone(),
            String::default(),
            self.code.clone(),
            String::default(),
        )
    }

    /// Consume this result into its [`AtelierOutput`] finishing plate.
    ///
    /// The consuming sibling of [`atelier_output`](Self::atelier_output): the
    /// owned `preamble`/`code` move straight into the imports/functions sections
    /// with no clone, so SFC assembly builds its output module from the shared
    /// finish-plate structure (#1758) at zero allocation cost — byte-for-byte
    /// identical to the previous `{ preamble, code }` positional pairing.
    pub fn into_atelier_output(self) -> AtelierOutput {
        AtelierOutput::new(
            self.preamble,
            String::default(),
            self.code,
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

    #[test]
    fn into_atelier_output_moves_sections_identically() {
        let result = SsrCodegenResult {
            code: String::from("function ssrRender() {}"),
            preamble: String::from("import { x } from \"vue\"\n"),
        };

        let output = result.into_atelier_output();
        assert_eq!(output.imports.as_str(), "import { x } from \"vue\"\n");
        assert_eq!(output.functions.as_str(), "function ssrRender() {}");
        assert!(output.hoists.is_empty());
        assert!(output.exports.is_empty());
    }
}
