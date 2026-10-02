//! Observe original public VAPOR compiler entrypoints for the shared corpus.

mod fixtures;

use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [command] if command == "--contract" => json!({
            "schema": "vize.compiler.public-target-observation", "version": 1,
            "cases": ["compiler/vapor/once-directive","compiler/vapor/insertion-placeholder","compiler/vapor/root-document-order","compiler/vapor/hydration-sibling-element","compiler/vapor/mounted-control-slot"],
            "fields": ["code","templates","map","errorMessages"], "native": "unsupported",
        }),
        [command] if command == "--observe" => {
            let cases = fixtures::CASES
                .iter()
                .map(|&(id, source, prefix)| {
                    json!({
                        "id": format!("compiler/vapor/{id}"), "source": source,
                        "entrypoint": "compile_vapor", "options": fixtures::option_payload(prefix),
                        "result": fixtures::observe(source, prefix),
                    })
                })
                .collect::<Vec<_>>();
            json!({ "schema": "vize.compiler.public-target-observation", "version": 1, "cases": cases })
        }
        _ => return Err("usage: vapor_fix_history_observer --contract|--observe".into()),
    };
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}
