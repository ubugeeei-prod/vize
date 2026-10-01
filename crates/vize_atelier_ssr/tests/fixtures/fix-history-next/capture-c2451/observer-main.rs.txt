//! One-shot measurement of five further authored SSR fix workloads.

use serde_json::json;

mod fixtures;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cases = fixtures::CASES
        .iter()
        .map(|(id, source)| Ok(json!({ "id": id, "output": fixtures::observe(source)? })))
        .collect::<Result<Vec<_>, serde_json::Error>>()?;
    println!(
        "{}",
        serde_json::to_string(&json!({
            "schema": "vize.ssr.fix-history-next-observation", "version": 1,
            "options": fixtures::options()?, "cases": cases,
        }))?
    );
    Ok(())
}
