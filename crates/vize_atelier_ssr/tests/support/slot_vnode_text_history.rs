//! Complete source-bound successors for scalar text children inside slot VNodes.
use serde_json::Value;

pub fn read(source: &str, ids: &[&str], options: &Value) -> Value {
    let current: Value = serde_json::from_str(source).expect("valid source-bound text successor");
    assert_eq!(
        current["sourceCommit"],
        "3d69ee66328cb981e02c6eed97f4d73d5f917f0b"
    );
    assert_eq!(current["repeatEquality"], true);
    assert_eq!(&current["options"], options);
    let cases = current["cases"]
        .as_array()
        .expect("complete successor cases");
    assert_eq!(
        cases
            .iter()
            .map(|case| case["id"].as_str().expect("case id"))
            .collect::<Vec<_>>(),
        ids,
        "only the reviewed scalar text cases may select a successor"
    );
    current
}

pub fn output<'a>(current: &'a Value, id: &str, previous: &'a Value) -> &'a Value {
    let cases = current["cases"].as_array().expect("successor cases");
    match cases.iter().find(|case| case["id"].as_str() == Some(id)) {
        Some(case) => {
            assert_eq!(
                &case["previousOutput"], previous,
                "previous complete output changed for {id}"
            );
            &case["output"]
        }
        None => previous,
    }
}
