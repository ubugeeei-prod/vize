use std::path::Path;

use vize_carton::{String, cstr};

use crate::file_uri::path_to_file_uri;

pub(super) const VUE_DEPENDENCY_FALLBACK: &str =
    "const component: any = undefined;\nexport default component;\n";

pub(super) fn fallback_vue_virtual_uri(path: &Path) -> String {
    let virtual_path = path.with_file_name(cstr!(
        "{}.ts",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
    ));
    path_to_file_uri(&virtual_path)
}
