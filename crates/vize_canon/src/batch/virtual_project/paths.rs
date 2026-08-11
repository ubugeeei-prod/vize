use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::cstr;

use crate::batch::AUTHORED_VUE_TS_ALIAS_SENTINEL;
use crate::batch::error::{CorsaError, CorsaResult};

pub(super) fn script_virtual_path(
    project_root: &Path,
    virtual_root: &Path,
    path: &Path,
    preserve_declaration_spelling: bool,
) -> CorsaResult<PathBuf> {
    // Out-of-root scripts (a workspace barrel reached by the reachability
    // pass, #3887) mirror into the external escape subtree.
    let mut virtual_path = match path.strip_prefix(project_root) {
        Ok(relative) => virtual_root.join(relative),
        Err(_) => super::external_mirror::external_mirror_path(virtual_root, path)?,
    };
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
    if !preserve_declaration_spelling && let Some(stem) = file_name.strip_suffix(".d.ts") {
        virtual_path.set_file_name(cstr!("{stem}.d.cts").as_str());
    }
    Ok(virtual_path)
}

pub(super) fn source_type_for_path(path: &Path) -> Option<SourceType> {
    let file_name = path.file_name()?.to_str()?;
    if file_name.ends_with(".jsx") {
        return Some(SourceType::unambiguous().with_jsx(true));
    }
    if file_name.ends_with(".tsx") {
        return Some(SourceType::tsx());
    }
    if file_name.ends_with(".cjs") {
        return Some(SourceType::cjs());
    }
    if file_name.ends_with(".mjs") {
        return Some(SourceType::mjs());
    }
    if file_name.ends_with(".js") {
        return Some(SourceType::unambiguous());
    }
    if file_name.ends_with(".ts")
        || file_name.ends_with(".d.ts")
        || file_name.ends_with(".mts")
        || file_name.ends_with(".cts")
    {
        return Some(SourceType::ts());
    }
    if file_name.ends_with(".js") || file_name.ends_with(".mjs") || file_name.ends_with(".cjs") {
        return SourceType::from_path(path).ok();
    }
    None
}
