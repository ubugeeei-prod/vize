//! One-shot measurement of exact authored Vapor history workloads.

use serde_json::json;

mod fixtures;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cases = fixtures::CASES
        .iter()
        .map(|(id, source, prefix)| {
            json!({
                "id": id, "options": fixtures::option_payload(*prefix),
                "output": fixtures::observe(source, *prefix),
            })
        })
        .collect::<Vec<_>>();
    println!(
        "{}",
        serde_json::to_string(&json!({
            "schema": "vize.vapor.fix-history-observation", "version": 1,
            "cases": cases,
        }))?
    );
    Ok(())
}
