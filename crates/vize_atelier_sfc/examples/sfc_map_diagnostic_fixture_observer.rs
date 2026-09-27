//! Observe whole current results for the original map/diagnostic fix inputs.

#[path = "../tests/support/fix_history_map_diagnostics.rs"]
mod fixtures;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string(&fixtures::observe()?)?);
    Ok(())
}
