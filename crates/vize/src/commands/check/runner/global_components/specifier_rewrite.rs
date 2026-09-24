use std::path::Path;

use vize_s0::{String, cstr};

pub(super) fn rewrite_global_component_imports_for_virtual_project(
    type_annotation: &str,
    project_root: &Path,
) -> String {
    crate::commands::check::dts_rewrite::rewrite_import_type_specifiers(
        type_annotation,
        |specifier| virtual_project_global_component_specifier(specifier, project_root),
    )
}

fn virtual_project_global_component_specifier(specifier: &str, project_root: &Path) -> String {
    if !specifier.ends_with(".vue") {
        return specifier.into();
    }

    let specifier_path = Path::new(specifier);
    if let Some(relative) = specifier_path
        .is_absolute()
        .then(|| specifier_path.strip_prefix(project_root).ok())
        .flatten()
    {
        let mut rendered = cstr!("./{}", relative.display());
        rendered.push_str(".ts");
        return rendered;
    }

    cstr!("{specifier}.ts")
}
