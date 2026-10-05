use std::path::Path;

use super::patterns::is_unimplemented_document_extension;

pub(super) fn validate(path: &Path) -> Result<(), &'static str> {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(is_unimplemented_document_extension)
    {
        return Err(
            "YAML and Markdown formatting is not implemented; these formats are excluded from default discovery",
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate;
    use std::path::Path;

    #[test]
    fn complete_document_support_boundary_is_explicit() {
        for file in ["a.yaml", "a.yml", "a.md", "a.markdown"] {
            assert_eq!(
                validate(Path::new(file)),
                Err(
                    "YAML and Markdown formatting is not implemented; these formats are excluded from default discovery"
                )
            );
        }
        for file in ["a.vue", "a.js", "a.ts", "a.json", "a.jsonc"] {
            assert_eq!(validate(Path::new(file)), Ok(()));
        }
    }
}
