use crate::{BackendFamily, RuntimeFamily};

#[test]
fn javascript_families_match_the_language_neutral_schema_value() {
    let runtime = serde_json::to_value(RuntimeFamily::JavaScript).unwrap();
    let backend = serde_json::to_value(BackendFamily::JavaScript).unwrap();

    assert_eq!(runtime, "javascript");
    assert_eq!(backend, "javascript");
    assert_eq!(
        serde_json::from_value::<RuntimeFamily>(runtime).unwrap(),
        RuntimeFamily::JavaScript
    );
    assert_eq!(
        serde_json::from_value::<BackendFamily>(backend).unwrap(),
        BackendFamily::JavaScript
    );
}
