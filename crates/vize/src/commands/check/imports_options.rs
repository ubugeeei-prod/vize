#[derive(Clone, Copy, Debug, Default)]
pub(in crate::commands::check) struct ImportFileOptions {
    pub(in crate::commands::check) include_js: bool,
    pub(in crate::commands::check) include_jsx: bool,
}

impl From<bool> for ImportFileOptions {
    fn from(include_jsx: bool) -> Self {
        Self {
            include_js: false,
            include_jsx,
        }
    }
}
