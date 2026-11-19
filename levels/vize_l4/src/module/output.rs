use vize_l0::String;

/// Structural module sections before they are flattened into emitted code.
///
/// This legacy-compatible boundary keeps imports, hoists, functions and
/// exports distinct until the final byte layout and source offsets are known.
#[derive(Debug, Default)]
pub struct OutputModule {
    pub imports: String,
    pub hoists: String,
    pub functions: String,
    pub exports: String,
}

impl OutputModule {
    /// Build the existing template-module shape from its emitted chunks.
    #[must_use]
    pub fn from_render_chunks(imports: String, functions: String) -> Self {
        Self {
            imports,
            functions,
            ..Self::default()
        }
    }

    /// Byte offset at which the function section starts after flattening.
    #[must_use]
    pub fn function_base_offset(&self) -> usize {
        self.imports.len() + self.hoists.len() + 1
    }

    /// Flatten sections in the legacy module order without rewriting bytes.
    #[must_use]
    pub fn into_code(self) -> String {
        let mut code = String::default();
        code.push_str(&self.imports);
        code.push_str(&self.hoists);
        code.push('\n');
        code.push_str(&self.functions);
        code.push('\n');
        code.push_str(&self.exports);
        code
    }
}

#[cfg(test)]
mod tests {
    use vize_l0::ToCompactString;

    use super::OutputModule;

    #[test]
    fn flattening_preserves_section_bytes_and_function_offset() {
        let module = OutputModule {
            imports: "import { h } from \"vue\"".to_compact_string(),
            hoists: "\nconst café = h".to_compact_string(),
            functions: "function render() { return café }".to_compact_string(),
            exports: "export { render }\n".to_compact_string(),
        };
        let function_base = module.function_base_offset();
        let code = module.into_code();

        assert_eq!(
            code.as_str(),
            concat!(
                "import { h } from \"vue\"\n",
                "const café = h\n",
                "function render() { return café }\n",
                "export { render }\n",
            )
        );
        assert_eq!(code.find("function render"), Some(function_base));
    }

    #[test]
    fn render_chunks_keep_legacy_empty_hoist_and_export_newlines() {
        let module = OutputModule::from_render_chunks(
            "import { h } from \"vue\"".to_compact_string(),
            "function render() {}".to_compact_string(),
        );
        let function_base = module.function_base_offset();
        let code = module.into_code();

        assert_eq!(
            code.as_str(),
            "import { h } from \"vue\"\nfunction render() {}\n"
        );
        assert_eq!(code.find("function render"), Some(function_base));
    }
}
