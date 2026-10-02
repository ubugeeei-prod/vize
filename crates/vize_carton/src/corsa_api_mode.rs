//! Runtime transport classification shared by Corsa-backed integrations.

use std::path::Path;

/// Whether `path` names a TypeScript/Corsa runtime that speaks the async
/// JSON-RPC stdio API instead of Corsa's older sync msgpack transport.
pub fn uses_async_json_rpc_api(path: &Path) -> bool {
    if path.extension().and_then(|extension| extension.to_str()) == Some("js") {
        return true;
    }

    if path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        == Some(".bin")
    {
        return true;
    }

    is_typescript_package_runtime(path)
}

fn is_typescript_package_runtime(path: &Path) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };
    let Some(parent_name) = parent.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if parent_name != "bin" && parent_name != "lib" {
        return false;
    }

    let Some(package_dir) = parent.parent() else {
        return false;
    };
    let Some(package_name) = package_dir.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if package_name == "typescript" {
        return true;
    }
    let Some(scope_name) = package_dir
        .parent()
        .and_then(|scope| scope.file_name())
        .and_then(|name| name.to_str())
    else {
        return false;
    };
    if scope_name != "@typescript" {
        return false;
    }

    package_name.starts_with("typescript-")
        || package_name == "native-preview"
        || package_name.starts_with("native-preview-")
}

#[cfg(test)]
mod tests {
    use super::uses_async_json_rpc_api;
    use std::path::{Path, PathBuf};

    #[test]
    fn classifies_typescript_package_runtimes_as_json_rpc() {
        for path in [
            PathBuf::from("/workspace/node_modules/.bin/tsgo"),
            PathBuf::from("/workspace/node_modules/typescript/bin/tsc"),
            PathBuf::from("/workspace/node_modules/typescript/lib/tsc.js"),
            PathBuf::from("/workspace/node_modules/@typescript/typescript-darwin-arm64/lib/tsc"),
            PathBuf::from("/workspace/node_modules/@typescript/typescript-win32-x64/lib/tsc.exe"),
            PathBuf::from(
                "/workspace/node_modules/@typescript/native-preview-darwin-arm64/lib/tsgo",
            ),
            PathBuf::from("/workspace/node_modules/@typescript/native-preview/bin/tsgo.js"),
        ] {
            assert!(
                uses_async_json_rpc_api(&path),
                "{} should use async JSON-RPC",
                path.display()
            );
        }

        assert!(!uses_async_json_rpc_api(Path::new("/workspace/bin/corsa")));
    }
}
