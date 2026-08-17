use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::cstr;

use crate::batch::AUTHORED_VUE_TS_ALIAS_SENTINEL;
use crate::batch::error::{CorsaError, CorsaResult};

mod commonjs_declaration;
use commonjs_declaration::declaration_requires_commonjs_spelling;

/// Mirror `path` into the virtual project, given `roots` as
/// `(project_root, virtual_root)`.
///
/// `content` is the file's authored text, read to decide whether a `.d.ts` has
/// to be mirrored as `.d.cts`; see [`commonjs_declaration`].
pub(super) fn script_virtual_path(
    roots: (&Path, &Path),
    path: &Path,
    content: &str,
    preserve_declaration_spelling: bool,
) -> CorsaResult<PathBuf> {
    let (project_root, virtual_root) = roots;
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
    if !preserve_declaration_spelling
        && let Some(stem) = file_name.strip_suffix(".d.ts")
        && declaration_requires_commonjs_spelling(content)
    {
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

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::script_virtual_path;

    fn mirrored(file_name: &str, content: &str) -> PathBuf {
        script_virtual_path(
            (Path::new("/project"), Path::new("/project/.virtual")),
            &Path::new("/project/types").join(file_name),
            content,
            false,
        )
        .expect("mirroring a project-local declaration file should succeed")
    }

    #[test]
    fn module_augmentation_declaration_keeps_its_declaration_extension() {
        assert_eq!(
            mirrored(
                "augment.d.ts",
                "declare module \"vue\" {\n  interface ComponentCustomProperties {\n    $local: (label: string) => string;\n  }\n}\n\nexport {};\n",
            ),
            Path::new("/project/.virtual/types/augment.d.ts"),
        );
    }

    #[test]
    fn commonjs_export_assignment_declaration_is_mirrored_as_cts() {
        assert_eq!(
            mirrored(
                "legacy.d.ts",
                "declare module \"legacy-shout\" {\n  function legacyShout(label: string): string;\n  export = legacyShout;\n}\n",
            ),
            Path::new("/project/.virtual/types/legacy.d.cts"),
        );
    }

    #[test]
    fn session_scripts_keep_every_declaration_extension() {
        let mirrored = script_virtual_path(
            (Path::new("/project"), Path::new("/project/.virtual")),
            Path::new("/project/types/legacy.d.ts"),
            "declare function f(): void;\nexport = f;\n",
            true,
        )
        .expect("mirroring a session script should succeed");
        assert_eq!(mirrored, Path::new("/project/.virtual/types/legacy.d.ts"));
    }

    #[test]
    fn plain_script_keeps_its_extension() {
        assert_eq!(
            mirrored("helper.ts", "export const value = 1;\n"),
            Path::new("/project/.virtual/types/helper.ts"),
        );
    }
}
