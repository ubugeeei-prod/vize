//! Raw public SSR Result bytes from five further original fix witnesses.
//!
//! Each authored source and the clean source-bound capture are immutable. The
//! test reads complete outputs, including preamble, nullable map and errors.

use serde_json::Value;

mod support;
use support::fix_history_next as fixtures;

const FIRST: &str = include_str!("fixtures/fix-history-next/capture-c2451/first.stdout.json");
const REPEAT: &str = include_str!("fixtures/fix-history-next/capture-c2451/repeat.stdout.json");
const EXPECTED: &[&str] = &[
    include_str!("fixtures/fix-history-next/textarea-model.expected.json"),
    include_str!("fixtures/fix-history-next/slot-fallback-vnode.expected.json"),
    include_str!("fixtures/fix-history-next/component-slot-props.expected.json"),
    include_str!("fixtures/fix-history-next/named-scoped-slot.expected.json"),
    include_str!("fixtures/fix-history-next/component-lone-spread.expected.json"),
];

#[test]
fn five_history_fixes_preserve_complete_ssr_outputs() {
    assert_eq!(
        FIRST.as_bytes(),
        REPEAT.as_bytes(),
        "two raw captures differ"
    );
    let captured = serde_json::from_str::<Value>(FIRST).expect("valid raw capture");
    let options = captured.get("options").expect("captured options");
    assert_eq!(
        fixtures::options().expect("original defaults must be admitted"),
        *options,
        "default SSR options differ from the captured historical workload"
    );
    let cases = captured
        .get("cases")
        .and_then(Value::as_array)
        .expect("captured cases");
    assert_eq!(cases.len(), fixtures::CASES.len());
    assert_eq!(EXPECTED.len(), cases.len());
    for (((id, source), captured_case), expected) in fixtures::CASES.iter().zip(cases).zip(EXPECTED)
    {
        assert_eq!(captured_case.get("id").and_then(Value::as_str), Some(*id));
        let expected = serde_json::from_str::<Value>(expected).expect("valid case expectation");
        assert_eq!(expected.get("options"), Some(options));
        assert_eq!(expected.get("output"), captured_case.get("output"));
        assert_eq!(
            fixtures::observe(source).expect("serialize complete output"),
            expected["output"],
            "complete SSR output changed for {id}"
        );
    }
}
