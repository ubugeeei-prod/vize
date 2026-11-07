//! What one `VirtualProject` keeps on the heap, by family, so editor memory
//! can be attributed to projection text, mapping rows and per-file indexes
//! rather than guessed from RSS.

use std::fmt;
use std::path::Path;

use super::VirtualProject;

/// Retained heap bytes of one project, by family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RetainedBytes {
    pub files: usize,
    /// Materialized projection text (`VirtualFile::content`).
    pub content: usize,
    /// Pre-rewrite projection text kept for editor module resolution.
    pub pre_rewrite: usize,
    /// Authored text kept beside rewritten scripts.
    pub original_contents: usize,
    /// Mapping rows of every SFC projection.
    pub mapping_rows: usize,
    pub mapping_sub_spans: usize,
    pub mapping_meta: usize,
    pub semantic_links: usize,
    /// Import rewrite offset rows.
    pub import_maps: usize,
    /// Path keys and values of the per-file indexes.
    pub file_paths: usize,
    /// Entries across the package route and shadow indexes.
    pub package_entries: usize,
    pub dependency_edges: usize,
}

impl RetainedBytes {
    /// Every family that is a byte count.
    pub fn total(&self) -> usize {
        self.content
            + self.pre_rewrite
            + self.original_contents
            + self.mapping_rows
            + self.mapping_sub_spans
            + self.mapping_meta
            + self.semantic_links
            + self.import_maps
            + self.file_paths
    }
}

impl fmt::Display for RetainedBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mb = |bytes: usize| bytes as f64 / 1_048_576.0;
        write!(
            formatter,
            "files={} content={:.1}MB pre_rewrite={:.1}MB original={:.1}MB rows={:.1}MB sub_spans={:.1}MB meta={:.1}MB links={:.1}MB import_maps={:.1}MB paths={:.1}MB total={:.1}MB package_entries={} dependency_edges={}",
            self.files,
            mb(self.content),
            mb(self.pre_rewrite),
            mb(self.original_contents),
            mb(self.mapping_rows),
            mb(self.mapping_sub_spans),
            mb(self.mapping_meta),
            mb(self.semantic_links),
            mb(self.import_maps),
            mb(self.file_paths),
            mb(self.total()),
            self.package_entries,
            self.dependency_edges,
        )
    }
}

fn path_bytes(path: &Path) -> usize {
    path.as_os_str().len()
}

impl VirtualProject {
    /// The heap this project retains, by family.
    pub fn retained_bytes(&self) -> RetainedBytes {
        let mut census = RetainedBytes {
            files: self.virtual_files.len(),
            ..RetainedBytes::default()
        };
        for (path, file) in &self.virtual_files {
            census.content += file.content.len();
            census.file_paths +=
                path_bytes(path) + path_bytes(&file.original_path) + path_bytes(&file.virtual_path);
            census.import_maps += file.source_map.import_map.retained_bytes();
            if let Some(sfc) = &file.source_map.sfc_map {
                let (rows, sub_spans, meta, links) = sfc.projection().retained_bytes();
                census.mapping_rows += rows;
                census.mapping_sub_spans += sub_spans;
                census.mapping_meta += meta;
                census.semantic_links += links;
            }
        }
        census.pre_rewrite = self
            .pre_rewrite_sources
            .iter()
            .filter_map(|path| self.virtual_files.get(path))
            .map(|file| file.source_map.import_map.retained_bytes())
            .sum();
        census.original_contents = self
            .original_contents
            .values()
            .map(|content| content.len())
            .sum();
        census.file_paths += self
            .original_index
            .iter()
            .map(|(key, value)| path_bytes(key) + path_bytes(value))
            .sum::<usize>();
        census.file_paths += self
            .passthrough_files
            .iter()
            .map(|(key, value)| path_bytes(key) + path_bytes(value))
            .sum::<usize>();
        census.file_paths += self
            .source_artifacts
            .keys()
            .map(|path| path_bytes(path))
            .sum::<usize>();
        census.package_entries = self.package_routes.len()
            + self.package_route_importers.len()
            + self.package_route_source_owners.len()
            + self.package_route_source_paths.len()
            + self.package_route_manifests.len()
            + self.package_route_roots.len()
            + self.package_source_index.len()
            + self.package_shadow_files.len()
            + self.package_shadow_manifests.len()
            + self.materialized_package_links.len();
        census.dependency_edges = self
            .dependency_edges
            .values()
            .map(|edges| edges.len())
            .sum();
        census
    }
}
