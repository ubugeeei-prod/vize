#![allow(dead_code)]

use std::{path::PathBuf, sync::Arc};

use corsa::{
    api::{
        ApiFileSystem, ApiMode, ApiProfile, ApiSpawnConfig, DirectoryEntries,
        FileSystemCapabilities, ReadFileResult,
    },
    lsp::LspSpawnConfig,
};
use corsa_core::fast::{CompactString, FastMap, SmallVec};

pub fn mock_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mock_corsa"))
}

pub fn workspace_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join("pnpm-workspace.yaml").exists() {
            return dir;
        }
        assert!(
            dir.pop(),
            "failed to locate workspace root from {}",
            env!("CARGO_MANIFEST_DIR")
        );
    }
}

pub fn test_cwd() -> PathBuf {
    let cwd = std::env::temp_dir().join("corsa-tests");
    std::fs::create_dir_all(&cwd).unwrap();
    cwd
}

pub fn api_config(mode: ApiMode) -> ApiSpawnConfig {
    ApiSpawnConfig::new(mock_binary())
        .with_mode(mode)
        .with_cwd(test_cwd())
}

pub fn api_profile(id: &str, mode: ApiMode) -> ApiProfile {
    ApiProfile::new(id, api_config(mode))
}

pub fn lsp_config() -> LspSpawnConfig {
    LspSpawnConfig::new(mock_binary()).with_cwd(test_cwd())
}

pub fn resolved_real_corsa_binary() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("CORSA_EXECUTABLE") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }
    [
        workspace_root().join(".cache/corsa"),
        workspace_root().join(".cache/corsa.exe"),
        workspace_root().join("ref/corsa-upstream/.cache/corsa"),
        workspace_root().join("ref/corsa-upstream/.cache/corsa.exe"),
        workspace_root().join("ref/corsa-upstream/built/local/corsa"),
        workspace_root().join("ref/corsa-upstream/built/local/corsa.exe"),
    ]
    .into_iter()
    .find(|path| path.exists())
}

pub fn real_corsa_binary() -> PathBuf {
    resolved_real_corsa_binary().unwrap_or_else(|| {
        workspace_root().join(if cfg!(windows) {
            ".cache/corsa.exe"
        } else {
            ".cache/corsa"
        })
    })
}

pub fn real_dataset() -> PathBuf {
    [workspace_root().join("ref/corsa-upstream/packages/typescript/tsconfig.json")]
        .into_iter()
        .find(|path| path.exists())
        .unwrap_or_else(|| {
            workspace_root().join("ref/corsa-upstream/packages/typescript/tsconfig.json")
        })
}

pub fn real_api_config(mode: ApiMode) -> Option<ApiSpawnConfig> {
    let binary = resolved_real_corsa_binary()?;
    let dataset = real_dataset();
    if !dataset.exists() {
        return None;
    }
    Some(
        ApiSpawnConfig::new(binary)
            .with_mode(mode)
            .with_cwd(workspace_root()),
    )
}

pub fn virtual_fs(entries: &[(&str, &str)]) -> Arc<VirtualFs> {
    let files = entries
        .iter()
        .map(|(path, content)| (CompactString::from(*path), CompactString::from(*content)))
        .collect();
    Arc::new(VirtualFs { files })
}

pub struct VirtualFs {
    files: FastMap<CompactString, CompactString>,
}

impl ApiFileSystem for VirtualFs {
    fn capabilities(&self) -> FileSystemCapabilities {
        FileSystemCapabilities {
            read_file: true,
            file_exists: true,
            directory_exists: true,
            get_accessible_entries: true,
            realpath: true,
        }
    }

    fn read_file(&self, path: &str) -> ReadFileResult {
        self.files
            .get(path)
            .cloned()
            .map(ReadFileResult::Content)
            .unwrap_or(ReadFileResult::Fallback)
    }

    fn file_exists(&self, path: &str) -> Option<bool> {
        Some(self.files.contains_key(path))
    }

    fn directory_exists(&self, path: &str) -> Option<bool> {
        Some(path == "/virtual")
    }

    fn get_accessible_entries(&self, path: &str) -> Option<DirectoryEntries> {
        (path == "/virtual").then(|| DirectoryEntries {
            files: self
                .files
                .keys()
                .filter_map(|path| path.strip_prefix("/virtual/").map(CompactString::from))
                .collect(),
            directories: SmallVec::new(),
        })
    }

    fn realpath(&self, path: &str) -> Option<CompactString> {
        Some(path.into())
    }
}
