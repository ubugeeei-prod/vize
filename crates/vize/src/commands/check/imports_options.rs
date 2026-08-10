use std::path::{Path, PathBuf};

const RESOLVE_EXTENSIONS: &[&str] = &[".ts", ".tsx", ".vue", ".mts", ".cts"];
const JSX_RESOLVE_EXTENSIONS: &[&str] = &[".ts", ".tsx", ".jsx", ".vue", ".mts", ".cts"];
const JS_RESOLVE_EXTENSIONS: &[&str] = &[
    ".ts", ".tsx", ".vue", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs",
];

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(in crate::commands::check) struct ImportFileOptions {
    pub(in crate::commands::check) include_js: bool,
    pub(in crate::commands::check) include_jsx: bool,
}

impl ImportFileOptions {
    pub(super) fn path_has_typescript_source_extension(path: &Path) -> bool {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return false;
        };
        RESOLVE_EXTENSIONS
            .iter()
            .any(|ext| name.ends_with(ext) && name.len() > ext.len())
    }

    pub(super) fn javascript_extension_is_enabled(self, path: &Path) -> bool {
        let extension = path.extension().and_then(|extension| extension.to_str());
        self.include_js && matches!(extension, Some("js" | "jsx" | "mjs" | "cjs"))
            || self.include_jsx && extension == Some("jsx")
    }

    pub(super) fn resolve_extensions(self) -> &'static [&'static str] {
        if self.include_js {
            JS_RESOLVE_EXTENSIONS
        } else if self.include_jsx {
            JSX_RESOLVE_EXTENSIONS
        } else {
            RESOLVE_EXTENSIONS
        }
    }
}

#[derive(Debug)]
pub(in crate::commands::check) struct TransitiveLocalImports {
    pub(in crate::commands::check) registrations: Vec<PathBuf>,
    pub(in crate::commands::check) authored: Vec<PathBuf>,
    /// Bare workspace-package specifiers whose source target must resolve to
    /// Vize's virtual mirror instead of the package manifest's real `.vue`.
    /// Keeping the original specifier in authored code lets declaration emit
    /// preserve package identity.
    pub(in crate::commands::check) virtual_module_aliases: Vec<(vize_carton::String, PathBuf)>,
}

impl From<bool> for ImportFileOptions {
    fn from(include_jsx: bool) -> Self {
        Self {
            include_js: false,
            include_jsx,
        }
    }
}
