mod backend;
mod reference;
mod refusals;
mod support;

use std::path::{Path, PathBuf};
use vize_canon::CorsaBridgeConfig;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const OPTIONS: &str = r#"{"compilerOptions":{"noEmit":true,"strict":true,"allowJs":true,"checkJs":true,"types":[],"target":"ESNext","module":"ESNext","moduleResolution":"Bundler","moduleDetection":"force","skipLibCheck":true},"include":["**/*"]}"#;

fn configured(root: &Path, backend: PathBuf) -> Result<CorsaBridgeConfig, std::io::Error> {
    std::fs::write(root.join("tsconfig.json"), OPTIONS)?;
    Ok(CorsaBridgeConfig {
        working_dir: Some(root.to_path_buf()),
        corsa_path: Some(backend),
        ..Default::default()
    })
}

fn require(observation: bool, failure: &'static str) -> TestResult {
    if observation {
        Ok(())
    } else {
        Err(failure.into())
    }
}

fn require_equal(
    actual: &serde_json::Value,
    expected: &serde_json::Value,
    context: &str,
) -> TestResult {
    if actual != expected {
        eprintln!("{context}: expected {expected:#}, actual {actual:#}");
        return Err("complete native Program diagnostic mismatch".into());
    }
    Ok(())
}
