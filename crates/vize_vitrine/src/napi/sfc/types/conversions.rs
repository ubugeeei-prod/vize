//! Additive include metadata without changing existing no-include packets.
use super::MacroArtifactNapi;
pub(crate) fn type_project_to_napi(
    project: &vize::type_projects::SelectedTypeProject,
) -> Option<super::TypeProjectContextNapi> {
    project
        .info()
        .map(|(root, revision)| super::TypeProjectContextNapi {
            root: root.to_string_lossy().into_owned(),
            revision: revision.to_string(),
        })
}
/// Watch inputs for the host: one file can be reached both canonically
/// (`\\?\C:\…` on Windows) and as authored, so list its plain spelling once.
pub(crate) fn dependencies_to_napi(dependencies: &[impl AsRef<str>]) -> Option<Vec<String>> {
    (!dependencies.is_empty()).then(|| {
        let mut paths: Vec<String> = dependencies
            .iter()
            .map(|path| vize_l0::path::normalize_windows_verbatim_str(path.as_ref()).into_owned())
            .collect();
        paths.sort_unstable();
        paths.dedup();
        paths
    })
}

pub(crate) fn macro_artifacts_to_napi(
    artifacts: Vec<vize_atelier_sfc::SfcMacroArtifact>,
) -> Vec<MacroArtifactNapi> {
    artifacts
        .into_iter()
        .map(|artifact| MacroArtifactNapi {
            kind: artifact.kind.into(),
            name: artifact.name.into(),
            source: artifact.source.into(),
            content: artifact.content.into(),
            module_code: artifact.module_code.map(Into::into),
            start: artifact.start as u32,
            end: artifact.end as u32,
        })
        .collect()
}
