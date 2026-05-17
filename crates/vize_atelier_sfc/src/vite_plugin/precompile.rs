use vize_carton::{FxHashMap, FxHashSet, String};

pub const DEFAULT_PRECOMPILE_BATCH_SIZE: usize = 128;
pub const DEFAULT_PRECOMPILE_BATCH_MAX_BYTES: f64 = 32.0 * 1024.0 * 1024.0;

#[derive(Clone, Debug, PartialEq)]
pub struct PrecompileFileMetadata {
    pub mtime_ms: f64,
    pub size: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrecompileFileMetadataEntry {
    pub path: String,
    pub metadata: PrecompileFileMetadata,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrecompileDiff {
    pub changed_files: Vec<String>,
    pub deleted_files: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PrecompileChunkOptions<'a> {
    pub max_bytes: Option<f64>,
    pub metadata: Option<&'a [PrecompileFileMetadataEntry]>,
}

pub fn has_file_metadata_changed(
    previous: Option<&PrecompileFileMetadata>,
    next: &PrecompileFileMetadata,
) -> bool {
    previous.is_none_or(|previous| previous.mtime_ms != next.mtime_ms || previous.size != next.size)
}

pub fn diff_precompile_files(
    files: &[String],
    current_metadata: &[PrecompileFileMetadataEntry],
    previous_metadata: &[PrecompileFileMetadataEntry],
) -> PrecompileDiff {
    let current = metadata_map(current_metadata);
    let previous = metadata_map(previous_metadata);
    let mut seen = FxHashSet::default();
    seen.reserve(files.len());

    let mut changed_files = Vec::new();
    for file in files {
        seen.insert(file.as_str());
        match current.get(file.as_str()) {
            Some(metadata)
                if !has_file_metadata_changed(previous.get(file.as_str()).copied(), metadata) => {}
            _ => changed_files.push(file.clone()),
        }
    }

    let mut deleted_files = Vec::new();
    for entry in previous_metadata {
        if !seen.contains(entry.path.as_str()) {
            deleted_files.push(entry.path.clone());
        }
    }

    PrecompileDiff {
        changed_files,
        deleted_files,
    }
}

pub fn normalize_precompile_batch_size(value: Option<f64>) -> usize {
    let Some(value) = value else {
        return DEFAULT_PRECOMPILE_BATCH_SIZE;
    };
    if !value.is_finite() || value <= 0.0 {
        return DEFAULT_PRECOMPILE_BATCH_SIZE;
    }

    usize::max(1, value.floor() as usize)
}

pub fn chunk_precompile_files(
    files: &[String],
    batch_size: Option<f64>,
    options: PrecompileChunkOptions<'_>,
) -> Vec<Vec<String>> {
    let normalized_batch_size = normalize_precompile_batch_size(batch_size);
    let max_bytes = normalize_max_bytes(options.max_bytes);
    let metadata = options.metadata.map(metadata_map);
    let mut chunks = Vec::new();
    let mut current = Vec::new();
    let mut current_bytes = 0.0;

    for file in files {
        let file_bytes = metadata
            .as_ref()
            .and_then(|metadata| metadata.get(file.as_str()))
            .map_or(0.0, |metadata| metadata.size.max(0.0));

        if !current.is_empty()
            && (current.len() >= normalized_batch_size || current_bytes + file_bytes > max_bytes)
        {
            chunks.push(current);
            current = Vec::new();
            current_bytes = 0.0;
        }

        current.push(file.clone());
        current_bytes += file_bytes;
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

fn metadata_map(
    entries: &[PrecompileFileMetadataEntry],
) -> FxHashMap<&str, &PrecompileFileMetadata> {
    let mut map = FxHashMap::default();
    map.reserve(entries.len());
    for entry in entries {
        map.insert(entry.path.as_str(), &entry.metadata);
    }
    map
}

fn normalize_max_bytes(value: Option<f64>) -> f64 {
    let value = value.unwrap_or(DEFAULT_PRECOMPILE_BATCH_MAX_BYTES).floor();
    if !value.is_finite() || value <= 0.0 {
        1.0
    } else {
        value
    }
}
