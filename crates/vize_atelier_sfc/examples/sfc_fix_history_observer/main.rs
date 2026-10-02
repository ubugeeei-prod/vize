//! Observe original public SFC compiler entrypoints for the shared corpus.

mod options;
mod profiles;

use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [command] if command == "--contract" => json!({
            "schema": "vize.compiler.public-sfc-observation",
            "version": 1,
            "cases": profiles::IDS,
            "fields": ["code", "css", "map", "errors", "warnings", "bindings", "macroArtifacts"],
            "native": "unsupported",
        }),
        [command] if command == "--observe" => json!({
            "schema": "vize.compiler.public-sfc-observation",
            "version": 1,
            "cases": profiles::observe()?,
        }),
        _ => return Err("usage: sfc_fix_history_observer --contract|--observe".into()),
    };
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}
