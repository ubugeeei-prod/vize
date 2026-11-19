//! Real branch and loop slot lifecycle under native, retained and official Vapor.
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "integration tests assert full runtime traces"
)]

mod cases;
mod trace;

use serde_json::{Value, json};
use vize_atelier_core::{expr_parse_probe, walk_probe::WalkCounts};
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

#[test]
fn structural_slot_lifetimes_match_official_runtime() {
    for case in cases::cases() {
        let allocator = Allocator::new();
        let options = VaporCompilerOptions {
            prefix_identifiers: true,
            ..Default::default()
        };
        let walks = WalkCounts::snapshot();
        let reparses = expr_parse_probe::expr_parse_count();
        let native = compile_vapor(&allocator, &case.source, options.clone());
        let child = compile_vapor(&allocator, &case.child, options.clone());
        assert!(
            native.error_messages.is_empty(),
            "{}: {:?}",
            case.name,
            native.error_messages
        );
        assert!(
            child.error_messages.is_empty(),
            "{} child: {:?}",
            case.name,
            child.error_messages
        );
        assert_eq!(
            WalkCounts::snapshot().since(walks).total_walks(),
            0,
            "{}",
            case.name
        );
        assert_eq!(
            expr_parse_probe::expr_parse_count() - reparses,
            0,
            "{}",
            case.name
        );
        let retained = compile_vapor(
            &allocator,
            &case.source,
            VaporCompilerOptions {
                davinci_retained_lane: true,
                ..options
            },
        );
        assert!(
            retained.error_messages.is_empty(),
            "{}: {:?}",
            case.name,
            retained.error_messages
        );
        let official = trace::trace(
            "davinci-upstream-vapor-trace.mjs",
            json!({
                "source": case.source, "context": case.context, "steps": case.steps,
                "components": {"Child": {"source": case.child, "props": ["value"]}},
            }),
        );
        assert_semantics(&case.name, &official);
        for (lane, code) in [("native", &native.code), ("retained", &retained.code)] {
            let actual = trace::trace(
                "davinci-mounted-trace.mjs",
                json!({
                    "backend": "vapor", "code": code, "context": case.context, "steps": case.steps,
                    "identities": true, "components": {"Child": {"code": child.code, "props": ["value"]}},
                }),
            );
            assert_eq!(actual, official, "{}: {lane}", case.name);
        }
    }
}

fn text(value: &Value) -> String {
    if let Some(text) = value.as_str() {
        return text.to_owned();
    }
    value
        .get("children")
        .or_else(|| value.get("tree"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(text)
        .collect()
}

fn assert_semantics(name: &str, frames: &[Value]) {
    let (texts, final_events): (&[&str], Value) = match name {
        "conditional" => (
            &[
                "A:P0twotail",
                "B:P1twotail",
                "B:P1twotail",
                "oneB:P1tail",
                "B:P1twotail",
                "B:P1twotail",
                "onetwotail",
                "C:P1twotail",
                "",
            ],
            json!(["B:P1", "P1"]),
        ),
        "loop" => (
            &[
                "A:0:P0B:1:P0tail",
                "A2:1:P1B2:0:P1tail",
                "A2:1:P1B2:0:P1tail",
                "C:1:P1B3:0:P1tail",
                "C:1:P1B3:0:P1tail",
                "B4:0:P1twotail",
                "onetwotail",
                "oneA3:0:P1tail",
                "",
            ],
            json!(["A2:1:P1", "C:1:P1"]),
        ),
        "object" => (
            &[
                "A:a:0:P0B:b:1:P0tail",
                "A2:a:0:P1B2:b:1:P1tail",
                "A2:a:0:P1B2:b:1:P1tail",
                "C:c:1:P1B3:b:0:P1tail",
                "C:c:1:P1B3:b:0:P1tail",
                "onetwotail",
                "",
            ],
            json!(["A2:a:0:P1", "C:c:1:P1"]),
        ),
        _ => panic!("unknown runtime case"),
    };
    assert_eq!(
        frames.iter().map(text).collect::<Vec<_>>(),
        texts,
        "{name}: live text"
    );
    assert_eq!(
        frames.last().unwrap()["events"],
        final_events,
        "{name}: listeners"
    );
    assert_eq!(
        frames.last().unwrap()["identities"],
        json!([]),
        "{name}: unmount"
    );
    assert_eq!(
        identities(frames, 0),
        identities(frames, 1),
        "{name}: body/props reuse"
    );
    if name != "conditional" {
        assert_eq!(
            identities(frames, 3)
                .get(2)
                .unwrap()
                .get(1)
                .unwrap()
                .as_u64(),
            Some(2),
            "{name}: same-name item replacement"
        );
        assert_eq!(
            identities(frames, 3)
                .get(3)
                .unwrap()
                .get(1)
                .unwrap()
                .as_u64(),
            Some(3),
            "{name}: other slot stays mounted"
        );
    }
}

fn identities(frames: &[Value], index: usize) -> &Value {
    let Some(frame) = frames.get(index) else {
        panic!("missing runtime frame {index}")
    };
    frame.get("identities").unwrap()
}
