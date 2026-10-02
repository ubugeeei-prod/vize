//! Observe original public SSR compiler entrypoints for the shared corpus.

mod fixtures;

use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [command] if command == "--contract" => json!({
            "schema": "vize.compiler.public-target-observation", "version": 1,
            "cases": ["compiler/ssr/textarea-model","compiler/ssr/slot-fallback-vnode","compiler/ssr/component-slot-props","compiler/ssr/named-scoped-slot","compiler/ssr/component-lone-spread"],
            "fields": ["code","preamble","map","diagnostics"], "native": "unsupported",
        }),
        [command] if command == "--observe" => {
            let cases = {
                let options = fixtures::options()?;
                let mut cases = Vec::new();
                for &(id, source) in fixtures::CASES {
                    cases.push(json!({
                        "id": format!("compiler/ssr/{id}"), "source": source,
                        "entrypoint": "compile_ssr", "options": options,
                        "result": fixtures::observe(source)?,
                    }));
                }
                cases
            };
            json!({ "schema": "vize.compiler.public-target-observation", "version": 1, "cases": cases })
        }
        _ => return Err("usage: ssr_fix_history_observer --contract|--observe".into()),
    };
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}
