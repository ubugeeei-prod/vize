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
pub(crate) fn dependencies_to_napi(dependencies: Vec<vize_l0::String>) -> Option<Vec<String>> {
    (!dependencies.is_empty()).then(|| {
        let mut paths: Vec<String> = dependencies
            .into_iter()
            .map(|path| {
                // The compile result no longer needs these paths. Moving a
                // heap-backed CompactString into String keeps its allocation.
                vize::type_projects::plain_windows_path(path.as_str())
                    .map_or_else(|| path.into(), Into::into)
            })
            .collect();
        paths.sort_unstable();
        paths.dedup();
        paths
    })
}

#[cfg(test)]
mod tests {
    use super::dependencies_to_napi;

    #[test]
    fn dependencies_keep_their_packet_and_reuse_owned_path_storage() {
        assert_eq!(dependencies_to_napi(vec![]), None);
        let path = vize_l0::String::from("/project/node_modules/domain-types/src/models/Item.ts");
        let allocation = path.as_ptr();
        let converted = dependencies_to_napi(vec![path]).expect("one dependency");
        assert_eq!(
            converted,
            ["/project/node_modules/domain-types/src/models/Item.ts"]
        );
        assert_eq!(converted.first().unwrap().as_ptr(), allocation);
        assert_eq!(
            dependencies_to_napi(vec!["/z.ts".into(), "/a.ts".into(), "/z.ts".into()]),
            Some(vec!["/a.ts".into(), "/z.ts".into()]),
        );
    }

    #[test]
    #[cfg(windows)]
    fn normalized_windows_spellings_are_still_deduplicated() {
        assert_eq!(
            dependencies_to_napi(vec![
                r"\\?\C:\repo\types.ts".into(),
                r"C:\repo\types.ts".into(),
            ]),
            Some(vec![r"C:\repo\types.ts".into()]),
        );
    }
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
