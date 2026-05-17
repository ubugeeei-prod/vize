use napi_derive::napi;
use vize_atelier_sfc::vite_plugin::{
    PrecompileChunkOptions, PrecompileFileMetadata, PrecompileFileMetadataEntry,
};

#[napi(object)]
pub struct VitePrecompileFileMetadataNapi {
    pub mtime_ms: f64,
    pub size: f64,
}

#[napi(object)]
pub struct VitePrecompileFileMetadataEntryNapi {
    pub path: String,
    pub mtime_ms: f64,
    pub size: f64,
}

#[napi(object)]
pub struct VitePrecompileDiffNapi {
    pub changed_files: Vec<String>,
    pub deleted_files: Vec<String>,
}

#[napi(object)]
pub struct VitePrecompileChunkOptionsNapi {
    pub max_bytes: Option<f64>,
    pub metadata: Option<Vec<VitePrecompileFileMetadataEntryNapi>>,
}

#[napi(js_name = "hasVitePrecompileFileMetadataChanged")]
pub fn has_vite_precompile_file_metadata_changed(
    previous: Option<VitePrecompileFileMetadataNapi>,
    next: VitePrecompileFileMetadataNapi,
) -> bool {
    let previous = previous.map(Into::into);
    let next = next.into();
    vize_atelier_sfc::vite_plugin::has_file_metadata_changed(previous.as_ref(), &next)
}

#[napi(js_name = "diffVitePrecompileFiles")]
pub fn diff_vite_precompile_files(
    files: Vec<String>,
    current_metadata: Vec<VitePrecompileFileMetadataEntryNapi>,
    previous_metadata: Vec<VitePrecompileFileMetadataEntryNapi>,
) -> VitePrecompileDiffNapi {
    let files = into_native_files(files);
    let current_metadata = into_native_entries(current_metadata);
    let previous_metadata = into_native_entries(previous_metadata);
    let diff = vize_atelier_sfc::vite_plugin::diff_precompile_files(
        &files,
        &current_metadata,
        &previous_metadata,
    );
    VitePrecompileDiffNapi {
        changed_files: diff.changed_files.into_iter().map(Into::into).collect(),
        deleted_files: diff.deleted_files.into_iter().map(Into::into).collect(),
    }
}

#[napi(js_name = "normalizeVitePrecompileBatchSize")]
pub fn normalize_vite_precompile_batch_size(value: Option<f64>) -> u32 {
    vize_atelier_sfc::vite_plugin::normalize_precompile_batch_size(value) as u32
}

#[napi(js_name = "chunkVitePrecompileFiles")]
pub fn chunk_vite_precompile_files(
    files: Vec<String>,
    batch_size: Option<f64>,
    options: Option<VitePrecompileChunkOptionsNapi>,
) -> Vec<Vec<String>> {
    let files = into_native_files(files);
    let metadata = options
        .as_ref()
        .and_then(|options| options.metadata.as_ref())
        .map(|metadata| into_native_entries_ref(metadata));
    let chunks = vize_atelier_sfc::vite_plugin::chunk_precompile_files(
        &files,
        batch_size,
        PrecompileChunkOptions {
            max_bytes: options.as_ref().and_then(|options| options.max_bytes),
            metadata: metadata.as_deref(),
        },
    );
    chunks
        .into_iter()
        .map(|chunk| chunk.into_iter().map(Into::into).collect())
        .collect()
}

impl From<VitePrecompileFileMetadataNapi> for PrecompileFileMetadata {
    fn from(metadata: VitePrecompileFileMetadataNapi) -> Self {
        Self {
            mtime_ms: metadata.mtime_ms,
            size: metadata.size,
        }
    }
}

fn into_native_entries(
    entries: Vec<VitePrecompileFileMetadataEntryNapi>,
) -> Vec<PrecompileFileMetadataEntry> {
    entries.into_iter().map(Into::into).collect()
}

fn into_native_entries_ref(
    entries: &[VitePrecompileFileMetadataEntryNapi],
) -> Vec<PrecompileFileMetadataEntry> {
    entries.iter().map(Into::into).collect()
}

fn into_native_files(files: Vec<String>) -> Vec<vize_carton::String> {
    files.into_iter().map(Into::into).collect()
}

impl From<VitePrecompileFileMetadataEntryNapi> for PrecompileFileMetadataEntry {
    fn from(entry: VitePrecompileFileMetadataEntryNapi) -> Self {
        Self {
            path: entry.path.into(),
            metadata: PrecompileFileMetadata {
                mtime_ms: entry.mtime_ms,
                size: entry.size,
            },
        }
    }
}

impl From<&VitePrecompileFileMetadataEntryNapi> for PrecompileFileMetadataEntry {
    fn from(entry: &VitePrecompileFileMetadataEntryNapi) -> Self {
        Self {
            path: entry.path.as_str().into(),
            metadata: PrecompileFileMetadata {
                mtime_ms: entry.mtime_ms,
                size: entry.size,
            },
        }
    }
}
