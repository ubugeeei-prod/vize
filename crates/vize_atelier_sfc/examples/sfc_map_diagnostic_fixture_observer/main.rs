//! Observe whole current results for the original map/diagnostic fix inputs.

mod fixture_options;
mod fixtures;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let observed = fixtures::observe()?;
    let cases = observed
        .get("cases")
        .and_then(serde_json::Value::as_array)
        .ok_or("observer lost its case array")?;
    if cases.len() != fixtures::CASE_IDS.len()
        || cases
            .iter()
            .zip(fixtures::CASE_IDS)
            .any(|(case, id)| case.get("id").and_then(serde_json::Value::as_str) != Some(*id))
    {
        return Err("observer cases are missing or reordered".into());
    }
    println!("{}", serde_json::to_string(&observed)?);
    Ok(())
}
