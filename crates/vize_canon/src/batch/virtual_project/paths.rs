use std::path::{Path, PathBuf};

use vize_carton::cstr;

use crate::batch::AUTHORED_VUE_TS_ALIAS_SENTINEL;
use crate::batch::error::{CorsaError, CorsaResult};

pub(super) fn script_virtual_path(
    project_root: &Path,
    virtual_root: &Path,
    path: &Path,
) -> CorsaResult<PathBuf> {
    let relative = path.strip_prefix(project_root)?;
    let mut virtual_path = virtual_root.join(relative);
    let Some(file_name) = virtual_path.file_name().and_then(|name| name.to_str()) else {
        return Err(CorsaError::PathError {
            path: path.to_path_buf(),
        });
    };
    let source_file_name = path.file_name().and_then(|name| name.to_str());
    let authored_vue_ts_extension = source_file_name.and_then(|name| {
        name.strip_suffix(".vue.ts")
            .map(|stem| (stem, "ts"))
            .or_else(|| name.strip_suffix(".vue.tsx").map(|stem| (stem, "tsx")))
    });
    if let Some((stem, extension)) = authored_vue_ts_extension
        && path.with_file_name(cstr!("{stem}.vue").as_str()).is_file()
    {
        virtual_path.set_file_name(
            cstr!("{file_name}{AUTHORED_VUE_TS_ALIAS_SENTINEL}.{extension}").as_str(),
        );
        return Ok(virtual_path);
    }
    if let Some(stem) = file_name.strip_suffix(".d.ts") {
        virtual_path.set_file_name(cstr!("{stem}.d.cts").as_str());
    }
    Ok(virtual_path)
}
