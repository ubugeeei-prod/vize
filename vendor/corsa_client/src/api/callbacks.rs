use crate::jsonrpc::{RpcHandler, RpcHandlerMap, RpcResponseError};
use corsa_core::fast::{CompactString, SmallVec, compact_format};
use phf::phf_map;
use serde_json::{Value, json};
use std::sync::Arc;

const CALLBACK_PREFIX: &str = "--callbacks=";

static CALLBACKS: phf::Map<&'static str, CallbackKind> = phf_map! {
    "readFile" => CallbackKind::ReadFile,
    "fileExists" => CallbackKind::FileExists,
    "directoryExists" => CallbackKind::DirectoryExists,
    "getAccessibleEntries" => CallbackKind::GetAccessibleEntries,
    "realpath" => CallbackKind::Realpath,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CallbackKind {
    ReadFile,
    FileExists,
    DirectoryExists,
    GetAccessibleEntries,
    Realpath,
}

/// Declares which filesystem callbacks are implemented by an [`ApiFileSystem`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FileSystemCapabilities {
    /// Enables the `readFile` callback.
    pub read_file: bool,
    /// Enables the `fileExists` callback.
    pub file_exists: bool,
    /// Enables the `directoryExists` callback.
    pub directory_exists: bool,
    /// Enables the `getAccessibleEntries` callback.
    pub get_accessible_entries: bool,
    /// Enables the `realpath` callback.
    pub realpath: bool,
}

/// Result of a `readFile` callback.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadFileResult {
    /// Defer to the server's default filesystem behavior.
    Fallback,
    /// Report that the file does not exist.
    NotFound,
    /// Return virtualized file contents.
    Content(CompactString),
}

/// Directory listing returned by `getAccessibleEntries`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DirectoryEntries {
    /// Visible file entries.
    pub files: SmallVec<[CompactString; 8]>,
    /// Visible subdirectory entries.
    pub directories: SmallVec<[CompactString; 8]>,
}

/// Filesystem interface exposed to Corsa.
///
/// Implementations can opt into individual callbacks via
/// [`capabilities`](Self::capabilities).
pub trait ApiFileSystem: Send + Sync + 'static {
    /// Returns the set of callbacks this implementation supports.
    fn capabilities(&self) -> FileSystemCapabilities;

    /// Returns file contents for `path`, or [`ReadFileResult::Fallback`] to let
    /// Corsa read from disk directly.
    fn read_file(&self, _path: &str) -> ReadFileResult {
        ReadFileResult::Fallback
    }

    /// Returns whether `path` exists as a file, or `None` to fall back to the
    /// server's native filesystem lookup.
    fn file_exists(&self, _path: &str) -> Option<bool> {
        None
    }

    /// Returns whether `path` exists as a directory, or `None` to fall back to
    /// the server's native filesystem lookup.
    fn directory_exists(&self, _path: &str) -> Option<bool> {
        None
    }

    /// Returns directory entries visible from `path`, or `None` to fall back
    /// to the server's native directory scan.
    fn get_accessible_entries(&self, _path: &str) -> Option<DirectoryEntries> {
        None
    }

    /// Returns a canonicalized path, or `None` to defer to the server.
    fn realpath(&self, _path: &str) -> Option<CompactString> {
        None
    }
}

/// Returns the enabled callback names in the order expected by Corsa.
///
/// # Examples
///
/// ```
/// use corsa_client::{ApiFileSystem, FileSystemCapabilities, callback_names};
///
/// struct Fs;
///
/// impl ApiFileSystem for Fs {
///     fn capabilities(&self) -> FileSystemCapabilities {
///         FileSystemCapabilities { read_file: true, realpath: true, ..Default::default() }
///     }
/// }
///
/// let names = callback_names(&Fs);
/// assert_eq!(names.as_slice(), &["readFile", "realpath"]);
/// ```
pub fn callback_names(fs: &dyn ApiFileSystem) -> SmallVec<[&'static str; 5]> {
    let caps = fs.capabilities();
    let mut names = SmallVec::new();
    if caps.read_file {
        names.push("readFile");
    }
    if caps.file_exists {
        names.push("fileExists");
    }
    if caps.directory_exists {
        names.push("directoryExists");
    }
    if caps.get_accessible_entries {
        names.push("getAccessibleEntries");
    }
    if caps.realpath {
        names.push("realpath");
    }
    names
}

/// Renders the `--callbacks=...` argument for a filesystem implementation.
///
/// # Examples
///
/// ```
/// use corsa_client::{ApiFileSystem, FileSystemCapabilities, callback_flag};
///
/// struct Fs;
///
/// impl ApiFileSystem for Fs {
///     fn capabilities(&self) -> FileSystemCapabilities {
///         FileSystemCapabilities { file_exists: true, directory_exists: true, ..Default::default() }
///     }
/// }
///
/// assert_eq!(
///     callback_flag(&Fs).as_deref(),
///     Some("--callbacks=fileExists,directoryExists"),
/// );
/// ```
pub fn callback_flag(fs: &dyn ApiFileSystem) -> Option<CompactString> {
    let names = callback_names(fs);
    (!names.is_empty()).then(|| render_callback_flag(&names))
}

/// Builds JSON-RPC handler functions for the enabled callbacks.
pub fn jsonrpc_handlers(fs: Arc<dyn ApiFileSystem>) -> RpcHandlerMap {
    callback_names(fs.as_ref())
        .into_iter()
        .map(|name| (CompactString::from(name), build_handler(fs.clone(), name)))
        .collect()
}

pub(crate) fn invoke_callback(
    fs: &dyn ApiFileSystem,
    method: &str,
    payload: &Value,
) -> std::result::Result<Value, RpcResponseError> {
    let Some(kind) = CALLBACKS.get(method).copied() else {
        return Err(unsupported_callback(method));
    };
    let path = callback_path(method, payload)?;
    Ok(match kind {
        CallbackKind::ReadFile => match fs.read_file(path) {
            ReadFileResult::Fallback => Value::Null,
            ReadFileResult::NotFound => json!({ "content": Value::Null }),
            ReadFileResult::Content(content) => json!({ "content": content }),
        },
        CallbackKind::FileExists => fs
            .file_exists(path)
            .map(Value::Bool)
            .unwrap_or(Value::Null),
        CallbackKind::DirectoryExists => fs
            .directory_exists(path)
            .map(Value::Bool)
            .unwrap_or(Value::Null),
        CallbackKind::GetAccessibleEntries => fs
            .get_accessible_entries(path)
            .map(|entries| {
                json!({
                    "files": Value::Array(
                        entries.files.into_iter().map(|path| Value::String(path.into())).collect()
                    ),
                    "directories": Value::Array(
                        entries.directories.into_iter().map(|path| Value::String(path.into())).collect()
                    ),
                })
            })
            .unwrap_or(Value::Null),
        CallbackKind::Realpath => fs
            .realpath(path)
            .map(|path| Value::String(path.into()))
            .unwrap_or(Value::Null),
    })
}

fn build_handler(fs: Arc<dyn ApiFileSystem>, method: &'static str) -> RpcHandler {
    Arc::new(move |payload| invoke_callback(fs.as_ref(), method, &payload))
}

fn render_callback_flag(names: &[&'static str]) -> CompactString {
    let capacity = CALLBACK_PREFIX.len()
        + names.iter().map(|name| name.len()).sum::<usize>()
        + names.len().saturating_sub(1);
    let mut flag = CompactString::with_capacity(capacity);
    flag.push_str(CALLBACK_PREFIX);
    for (index, name) in names.iter().enumerate() {
        if index > 0 {
            flag.push(',');
        }
        flag.push_str(name);
    }
    flag
}

fn unsupported_callback(method: &str) -> RpcResponseError {
    RpcResponseError {
        code: -32601,
        message: compact_format(format_args!("unsupported callback: {method}")),
        data: None,
    }
}

fn callback_path<'a>(
    method: &str,
    payload: &'a Value,
) -> std::result::Result<&'a str, RpcResponseError> {
    payload.as_str().ok_or_else(|| RpcResponseError {
        code: -32602,
        message: compact_format(format_args!(
            "invalid callback params for {method}: expected a string path"
        )),
        data: None,
    })
}

#[cfg(test)]
#[path = "callbacks_tests.rs"]
mod tests;
