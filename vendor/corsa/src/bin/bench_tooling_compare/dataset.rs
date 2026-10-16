use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use corsa::{
    CorsaError, Result,
    api::{ApiClient, ApiMode, ApiSpawnConfig},
    fast::{CompactString, SmallVec},
};

use crate::args::Cli;

#[derive(Clone, Debug)]
pub struct DatasetCase {
    pub label: CompactString,
    pub config_path: PathBuf,
    pub config_wire: CompactString,
    pub primary_file: CompactString,
    pub source_files: SmallVec<[CompactString; 64]>,
    pub file_count: usize,
    pub total_bytes: u64,
    pub total_lines: u64,
}

pub async fn load(cli: &Cli) -> Result<SmallVec<[DatasetCase; 4]>> {
    let client = ApiClient::spawn(
        ApiSpawnConfig::new(&cli.corsa_path)
            .with_cwd(&cli.root_dir)
            .with_mode(ApiMode::SyncMsgpackStdio),
    )
    .await?;
    let mut datasets = SmallVec::<[DatasetCase; 4]>::new();
    for config_path in &cli.dataset_paths {
        let config_wire = CompactString::from(config_path.to_string_lossy().as_ref());
        let config = client.parse_config_file(config_wire.as_str()).await?;
        let mut source_files = SmallVec::<[CompactString; 64]>::new();
        for file_name in &config.file_names {
            source_files.push(CompactString::from(file_name.as_str()));
        }
        let primary_file = pick_primary_file(&source_files).ok_or_else(|| {
            CorsaError::Protocol(CompactString::from(
                "dataset does not contain any source files",
            ))
        })?;
        let mut total_bytes = 0_u64;
        let mut total_lines = 0_u64;
        for file_name in &source_files {
            let resolved = resolve_file_name(file_name.as_str(), config_path);
            let (bytes, lines) = file_metrics(&resolved)?;
            total_bytes += bytes;
            total_lines += lines;
        }
        datasets.push(DatasetCase {
            label: dataset_label(config_path),
            config_path: config_path.clone(),
            config_wire,
            primary_file,
            source_files,
            file_count: config.file_names.len(),
            total_bytes,
            total_lines,
        });
    }
    client.close().await?;
    Ok(datasets)
}

fn pick_primary_file(file_names: &[CompactString]) -> Option<CompactString> {
    for file_name in file_names {
        if !file_name.ends_with(".d.ts") {
            return Some(file_name.clone());
        }
    }
    file_names.first().cloned()
}

fn dataset_label(config_path: &Path) -> CompactString {
    config_path
        .parent()
        .and_then(Path::file_name)
        .map(|name| CompactString::from(name.to_string_lossy().as_ref()))
        .unwrap_or_else(|| CompactString::from("dataset"))
}

fn resolve_file_name(file_name: &str, config_path: &Path) -> PathBuf {
    let path = Path::new(file_name);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    config_path
        .parent()
        .map(|parent| parent.join(path))
        .unwrap_or_else(|| path.to_path_buf())
}

fn file_metrics(path: &Path) -> Result<(u64, u64)> {
    let total_bytes = path.metadata()?.len();
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut total_lines = 0_u64;
    let mut saw_byte = false;
    let mut ended_with_newline = false;
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        saw_byte = true;
        let chunk = &buffer[..read];
        for byte in chunk {
            if *byte == b'\n' {
                total_lines += 1;
                ended_with_newline = true;
            } else {
                ended_with_newline = false;
            }
        }
    }
    if saw_byte && !ended_with_newline {
        total_lines += 1;
    }
    Ok((total_bytes, total_lines))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::{dataset_label, resolve_file_name};

    #[test]
    fn dataset_label_uses_parent_directory_name() {
        let path = PathBuf::from("/tmp/api/tsconfig.json");
        assert_eq!(dataset_label(&path).as_str(), "api");
    }

    #[test]
    fn resolve_file_name_preserves_absolute_paths() {
        let config = PathBuf::from("/tmp/api/tsconfig.json");
        assert_eq!(
            resolve_file_name("/tmp/api/src/index.ts", &config),
            PathBuf::from("/tmp/api/src/index.ts")
        );
        assert_eq!(
            resolve_file_name("src/index.ts", &config),
            PathBuf::from("/tmp/api/src/index.ts")
        );
        let _ = fs::metadata("/tmp");
    }
}
