use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::String as CompactString;

pub(super) struct RegisteredSource {
    pub(super) path: PathBuf,
    pub(super) content: CompactString,
    pub(super) source_type: Option<SourceType>,
}

pub(super) fn is_vue(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("vue")
}

pub(super) fn is_jsx(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".jsx") || name.ends_with(".tsx"))
}
