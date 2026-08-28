use std::path::Path;

use vize_s0::corsa_resolver::discover_corsa_in_ancestors;

pub(crate) fn resolve(workspace_root: &Path) -> Option<String> {
    if let Some(path) = std::env::var_os("CORSA_PATH").filter(|path| Path::new(path).is_file()) {
        return Some(path.to_string_lossy().into_owned());
    }

    for start in [
        workspace_root.to_path_buf(),
        workspace_root.join("examples").join("vite-musea"),
    ] {
        if let Some(path) = discover_corsa_in_ancestors(&start).filter(|path| path.is_file()) {
            return Some(path.display().to_string());
        }
    }

    None
}
