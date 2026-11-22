//! Capture the declaration membership and exact open/disk bytes of one editor request.
#![expect(
    clippy::disallowed_types,
    reason = "keep the editor's shared native overlay buffers"
)]
use super::super::{is_declaration_file, is_excluded_directory};
use super::ServerState;
use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};
use vize_canon::virtual_ts::VirtualTsOptions;
use vize_l0::FxHashMap;

impl ServerState {
    pub(crate) async fn ambient_declaration_inputs(
        &self,
        root: &Path,
        overlays: &[(PathBuf, Arc<String>)],
    ) -> io::Result<(VirtualTsOptions, Vec<(PathBuf, Arc<String>)>)> {
        let mut paths = self.global_component_reference_paths().await;
        let mut open = FxHashMap::default();
        for (path, text) in overlays {
            if !is_declaration_file(path) {
                continue;
            }
            let path = vize_l0::path::physical_path(path);
            if path
                .strip_prefix(root)
                .is_ok_and(|relative| relative.iter().all(|part| !is_excluded_directory(part)))
            {
                // Existing disk members always use their open buffer. Apply
                // discovery's size limit to new unsaved members only.
                if text.len() <= 4 * 1024 * 1024 {
                    paths.push(path.clone());
                }
                open.insert(path, Arc::clone(text));
            }
        }
        if !open.is_empty() {
            paths.sort_unstable();
            paths.dedup();
        }
        let mut options = self.virtual_ts_options();
        let mut selected = Vec::new();
        let mut closed = Vec::new();
        for path in paths {
            options.reference_paths.push(
                path.to_str()
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "declaration path has no UTF-8 spelling",
                        )
                    })?
                    .into(),
            );
            if path.starts_with(root) {
                match open.remove(&path) {
                    Some(text) => selected.push((path, text)),
                    None => closed.push(path),
                }
            }
        }
        if !closed.is_empty() {
            selected.extend(self.read_closed_declarations(closed).await?);
        }
        Ok((options, selected))
    }
}
