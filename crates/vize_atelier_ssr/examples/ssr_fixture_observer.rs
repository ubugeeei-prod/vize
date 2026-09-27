//! One-shot measurement of the exact authored SSR history workload.

use serde_json::json;

#[path = "../tests/support/fix_history.rs"]
mod fixtures;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cases = fixtures::CASES
        .iter()
        .map(|(id, source)| Ok(json!({ "id": id, "output": fixtures::observe(source)? })))
        .collect::<Result<Vec<_>, serde_json::Error>>()?;
    println!(
        "{}",
        serde_json::to_string(&json!({
            "schema": "vize.ssr.fix-history-observation", "version": 1,
            "options": fixtures::options()?, "cases": cases,
        }))?
    );
    Ok(())
}
