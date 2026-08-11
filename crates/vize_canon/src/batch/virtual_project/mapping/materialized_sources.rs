//! Exact authored identities for files emitted into a Canon materialization.

use std::path::PathBuf;

use super::super::VirtualProject;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MaterializedSourceMappingKind {
    Generated,
    AuthoredIdentity,
    Synthetic,
}

pub(crate) struct MaterializedSourceDocument {
    pub(crate) materialized_path: PathBuf,
    pub(crate) source_path: PathBuf,
    pub(crate) source: vize_carton::String,
    pub(crate) code: vize_carton::String,
    pub(crate) mappings: Vec<crate::virtual_ts::VizeMapping>,
    pub(crate) import_source_map: crate::batch::ImportSourceMap,
    pub(crate) mapping_kind: MaterializedSourceMappingKind,
}

impl VirtualProject {
    pub(crate) fn materialized_source_documents(&self) -> Vec<MaterializedSourceDocument> {
        let mut documents = self
            .virtual_files_sorted()
            .into_iter()
            .map(|file| {
                let source = self
                    .original_contents
                    .get(&file.virtual_path)
                    .cloned()
                    .unwrap_or_default();
                let mappings = file
                    .source_map
                    .sfc_map
                    .as_ref()
                    .map(|map| map.mappings().to_vec())
                    .unwrap_or_default();
                MaterializedSourceDocument {
                    materialized_path: file.virtual_path.clone(),
                    source_path: file.original_path.clone(),
                    source,
                    code: file.content.clone(),
                    mappings,
                    import_source_map: file.source_map.import_map.clone(),
                    mapping_kind: MaterializedSourceMappingKind::Generated,
                }
            })
            .collect::<Vec<_>>();

        for (materialized_path, canonical_path) in &self.package_shadow_files {
            let Some(file) = self.virtual_files.get(canonical_path) else {
                continue;
            };
            let Ok(code) = self.package_shadow_content(materialized_path, canonical_path) else {
                continue;
            };
            let source = self
                .original_contents
                .get(&file.virtual_path)
                .cloned()
                .unwrap_or_default();
            let mapping_kind = if code == file.content {
                MaterializedSourceMappingKind::Generated
            } else if code == source {
                MaterializedSourceMappingKind::AuthoredIdentity
            } else {
                MaterializedSourceMappingKind::Synthetic
            };
            let mappings = if mapping_kind == MaterializedSourceMappingKind::Generated {
                file.source_map
                    .sfc_map
                    .as_ref()
                    .map(|map| map.mappings().to_vec())
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            let import_source_map = if mapping_kind == MaterializedSourceMappingKind::Generated {
                file.source_map.import_map.clone()
            } else {
                Default::default()
            };
            documents.push(MaterializedSourceDocument {
                materialized_path: materialized_path.clone(),
                source_path: file.original_path.clone(),
                source,
                code,
                mappings,
                import_source_map,
                mapping_kind,
            });
        }

        for (materialized_path, source_path) in &self.passthrough_files {
            let Ok(source) = std::fs::read_to_string(source_path) else {
                continue;
            };
            documents.push(MaterializedSourceDocument {
                materialized_path: materialized_path.clone(),
                source_path: source_path.clone(),
                source: source.clone().into(),
                code: source.into(),
                mappings: Vec::new(),
                import_source_map: Default::default(),
                mapping_kind: MaterializedSourceMappingKind::AuthoredIdentity,
            });
        }

        documents.sort_by(|left, right| left.materialized_path.cmp(&right.materialized_path));
        documents.dedup_by(|left, right| left.materialized_path == right.materialized_path);
        documents
    }
}

#[cfg(test)]
mod tests {
    use super::{MaterializedSourceMappingKind, VirtualProject};

    #[test]
    fn passthrough_modules_are_exposed_as_authored_identity_mappings() {
        let root = tempfile::tempdir().unwrap();
        let host = root.path().join("src/main.ts");
        let data = root.path().join("src/data.json");
        std::fs::create_dir_all(host.parent().unwrap()).unwrap();
        std::fs::write(&host, "import data from './data.json';\nvoid data;\n").unwrap();
        std::fs::write(&data, "{\"answer\":42}\n").unwrap();
        let mut project = VirtualProject::new(root.path()).unwrap();
        project.register_path(&host).unwrap();

        let canonical_data = vize_carton::path::canonicalize_non_verbatim(&data);
        let document = project
            .materialized_source_documents()
            .into_iter()
            .find(|document| document.source_path == canonical_data)
            .expect("passthrough identity");
        assert_eq!(
            document.mapping_kind,
            MaterializedSourceMappingKind::AuthoredIdentity
        );
        assert_eq!(document.source, "{\"answer\":42}\n");
        assert_eq!(document.code, document.source);
        assert!(document.mappings.is_empty());
    }
}
