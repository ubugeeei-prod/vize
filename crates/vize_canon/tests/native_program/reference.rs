//! Original configured disk-file diagnostics without a second overlay snapshot.

use super::{TestResult, require};
use corsa::api::{ApiMode, ApiSpawnConfig, DocumentIdentifier, ProjectSession};
use std::{fmt::Write, path::Path};
use vize_canon::LspDiagnostic;
use vize_l0::String;

pub(super) struct DiskReference {
    session: ProjectSession,
    root: std::path::PathBuf,
}

impl DiskReference {
    pub(super) async fn spawn(
        root: &Path,
        executable: &Path,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let root = root.canonicalize()?;
        let config = root.join("tsconfig.json").canonicalize()?;
        let mode = if vize_carton::corsa_api_mode::uses_async_json_rpc_api(executable) {
            ApiMode::AsyncJsonRpcStdio
        } else {
            ApiMode::SyncMsgpackStdio
        };
        let session = ProjectSession::spawn(
            ApiSpawnConfig::new(executable.to_str().ok_or("backend path")?)
                .with_mode(mode)
                .with_cwd(&root),
            config.to_str().ok_or("reference config path")?,
            None,
        )
        .await?;
        eprintln!(
            "actual original configured ProjectSession: {}",
            serde_json::to_string(session.project())?,
        );
        let returned = Path::new(&session.project().config_file_name);
        let returned = if returned.is_absolute() {
            returned.to_path_buf()
        } else {
            root.join(returned)
        };
        require(
            returned.canonicalize()? == config,
            "actual backend resolves the configured original project",
        )?;
        let options = &session.project().compiler_options;
        require(
            options.get("moduleDetection") == Some(&serde_json::json!("force"))
                || options.get("moduleDetection") == Some(&serde_json::json!(3)),
            "actual original backend retains Module Force",
        )?;
        for option in ["strict", "allowJs", "checkJs"] {
            require(
                options.get(option).and_then(serde_json::Value::as_bool) == Some(true),
                "actual original backend retains authored checker options",
            )?;
        }
        Ok(Self { session, root })
    }

    pub(super) async fn diagnostics(
        &self,
        path: &Path,
    ) -> Result<Vec<LspDiagnostic>, Box<dyn std::error::Error>> {
        let path = path.canonicalize()?;
        require(
            path.starts_with(&self.root)
                && self.session.project().root_files.iter().any(|file| {
                    let file = Path::new(file);
                    let file = if file.is_absolute() {
                        file.to_path_buf()
                    } else {
                        self.root.join(file)
                    };
                    file.canonicalize().is_ok_and(|file| file == path)
                }),
            "original file belongs to the actual configured backend project",
        )?;
        let uri = file_uri(&path)?;
        let response = self
            .session
            .get_diagnostics_for_file(DocumentIdentifier::Uri {
                uri: uri.as_str().into(),
            })
            .await?;
        let returned = response.file.as_wire_value();
        require(
            returned.as_str() == uri.as_str()
                || returned.as_str() == path.to_str().ok_or("original file path")?,
            "actual compiler response belongs to the requested original file",
        )?;
        // Preserve the existing bridge's syntactic/semantic/suggestion order
        // and its complete public diagnostic representation, without filtering.
        response
            .syntactic
            .iter()
            .chain(&response.semantic)
            .chain(&response.suggestion)
            .map(|diagnostic| Ok(serde_json::from_value(serde_json::to_value(diagnostic)?)?))
            .collect()
    }

    pub(super) async fn close(&self) -> TestResult {
        self.session.close().await?;
        Ok(())
    }
}

fn file_uri(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let path = path.to_str().ok_or("original URI path")?;
    let mut uri: String = "file://".into();
    for byte in path.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' => {
                uri.push(char::from(byte));
            }
            _ => write!(uri, "%{byte:02X}")?,
        }
    }
    Ok(uri)
}
