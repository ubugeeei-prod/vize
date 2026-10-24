//! One-shot measurement of the exact authored SSR history workload.

use serde_json::json;

use davinci_test_support::ssr_fix_history as fixtures;

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
