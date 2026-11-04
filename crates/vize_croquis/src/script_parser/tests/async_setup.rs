//! Async setup inference keeps top-level and nested awaits distinct.

use super::super::parse_script_setup;
use crate::scope::{ScopeData, ScopeKind};

fn script_setup_is_async(source: &str) -> bool {
    let result = parse_script_setup(source);
    result
        .scopes
        .iter()
        .find(|scope| matches!(scope.kind, ScopeKind::ScriptSetup))
        .and_then(|scope| match scope.data() {
            ScopeData::ScriptSetup(data) => Some(data.is_async),
            _ => None,
        })
        .unwrap_or(false)
}

#[test]
fn test_parse_script_setup_marks_top_level_await_async() {
    assert!(script_setup_is_async("const data = await fetchData()"));
}

#[test]
fn test_parse_script_setup_marks_top_level_for_await_async() {
    assert!(script_setup_is_async(
        r#"
for await (const item of items) {
    console.log(item)
}
"#,
    ));
}

#[test]
fn test_parse_script_setup_ignores_non_top_level_awaits() {
    assert!(!script_setup_is_async(
        r#"
const message = "await should not force async setup"
// await should not force async setup
async function load() {
    await fetchData()
}
const run = async () => {
    await fetchData()
}
"#,
    ));
}
