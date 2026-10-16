//! Exact current typed descriptor membership in the owning native snapshot.
use corsa::api::ProjectResponse;
use std::path::Path;

pub(super) fn matching_project<'snapshot>(
    projects: &'snapshot [ProjectResponse],
    config: &Path,
    expected: &ProjectResponse,
) -> Option<&'snapshot ProjectResponse> {
    let config = config
        .to_str()
        .filter(|config| !config.contains('\0') && Path::new(config).is_absolute())?;
    let mut configured = projects
        .iter()
        .filter(|project| project.config_file_name == config);
    let selected = configured.next()?;
    if configured.next().is_some()
        || selected != expected
        || selected.id.as_str().is_empty()
        || projects
            .iter()
            .filter(|project| project.id == selected.id)
            .count()
            != 1
    {
        return None;
    }
    Some(selected)
}

#[cfg(test)]
mod tests;
