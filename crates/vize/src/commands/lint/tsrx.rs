//! `.tsrx` modules lint through [`vize_patina::Linter::lint_tsrx`], the one
//! TSRX lint entry the Node API and the language server share.

use std::path::Path;

use vize_l0::dialect::FileKind;

pub(super) fn is_tsrx_path(path: &Path) -> bool {
    FileKind::of_path(path) == Some(FileKind::Tsrx)
}
