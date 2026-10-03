use super::{NativeVueError, OriginalProgramError};
use crate::CorsaBridgeConfig;
use std::path::{Path, PathBuf};
use vize_l4::targets::ts::SourceKind;

#[derive(Clone)]
pub(super) struct Bounds {
    pub source: PathBuf,
    pub projected: PathBuf,
    pub config: PathBuf,
    configuration: Vec<u8>,
}
impl Bounds {
    pub fn checked(
        config: &CorsaBridgeConfig,
        authored: &Path,
        kind: SourceKind,
    ) -> Result<Self, NativeVueError> {
        let fail = NativeVueError::Identity;
        let root = config
            .working_dir
            .as_deref()
            .and_then(|path| path.canonicalize().ok())
            .ok_or_else(|| fail(OriginalProgramError::MissingConfiguredProject))?;
        let selected = [root.join("tsconfig.json"), root.join("jsconfig.json")]
            .into_iter()
            .find(|path| path.is_file())
            .ok_or_else(|| fail(OriginalProgramError::MissingConfiguredProject))?;
        if config
            .tsconfig_path
            .as_ref()
            .is_some_and(|path| path.canonicalize().ok().as_ref() != Some(&selected))
        {
            return Err(fail(OriginalProgramError::MissingConfiguredProject));
        }
        if !authored.is_absolute() {
            return Err(fail(OriginalProgramError::InvalidSourcePath));
        }
        let source = authored
            .canonicalize()
            .map_err(|_| fail(OriginalProgramError::InvalidSourcePath))?;
        if source.to_str().is_none()
            || selected.to_str().is_none()
            || source.extension().and_then(|extension| extension.to_str()) != Some("vue")
        {
            return Err(fail(OriginalProgramError::InvalidSourcePath));
        }
        if !source.starts_with(&root) {
            return Err(fail(OriginalProgramError::OutsideProject));
        }
        if source
            .parent()
            .into_iter()
            .flat_map(Path::ancestors)
            .take_while(|parent| *parent != root)
            .any(|parent| {
                parent.join("tsconfig.json").is_file() || parent.join("jsconfig.json").is_file()
            })
        {
            return Err(fail(OriginalProgramError::MissingConfiguredProject));
        }
        let mut filename = source.as_os_str().to_os_string();
        filename.push(match kind {
            SourceKind::JavaScript => ".mjs",
            SourceKind::TypeScript => ".ts",
            SourceKind::Jsx | SourceKind::Tsx => {
                return Err(fail(OriginalProgramError::SourceKindMismatch));
            }
        });
        let projected = PathBuf::from(filename);
        if std::fs::symlink_metadata(&projected).is_ok() {
            return Err(NativeVueError::ProjectionCollision);
        }
        let configuration = std::fs::read(&selected)
            .map_err(|_| fail(OriginalProgramError::MissingConfiguredProject))?;
        Ok(Self {
            source,
            projected,
            config: selected,
            configuration,
        })
    }
    pub fn verify(&self, original: &str) -> Result<(), NativeVueError> {
        if std::fs::read(&self.source).ok().as_deref() != Some(original.as_bytes()) {
            return Err(NativeVueError::Identity(
                OriginalProgramError::SourceChanged,
            ));
        }
        if std::fs::read(&self.config).ok().as_deref() != Some(self.configuration.as_slice()) {
            return Err(NativeVueError::Identity(
                OriginalProgramError::ConfigurationChanged,
            ));
        }
        if std::fs::symlink_metadata(&self.projected).is_ok() {
            return Err(NativeVueError::ProjectionCollision);
        }
        Ok(())
    }
}
