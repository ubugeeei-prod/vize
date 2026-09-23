//! Ordinary phrasing elements admitted by S3 keep dynamic child addresses.

use super::native_control::element;
use crate::mounted_trace;
use serde_json::json;

#[test]
fn semantic_inline_elements_update_in_place() {
    let source = r#"<p><code :title="tip">{{ label }}</code><mark>new</mark><time :datetime="date">{{ date }}</time></p>"#;
    let view = |tip: &str, label: &str, date: &str| {
        json!({
            "tree": [element("p", json!({}), json!([
                element("code", json!({"title": tip}), json!([label])),
                element("mark", json!({}), json!(["new"])),
                element("time", json!({"datetime": date}), json!([date]))
            ]))],
            "events": []
        })
    };
    let expected = json!([
        view("first", "A", "2026-09-23"),
        view("second", "B", "2026-09-24"),
        {"tree": [], "events": []}
    ]);
    for backend in ["vdom", "vapor", "vapor-legacy"] {
        let actual = mounted_trace(
            backend,
            source,
            json!({"tip": "first", "label": "A", "date": "2026-09-23"}),
            json!([{"patch": {"tip": "second", "label": "B", "date": "2026-09-24"}}]),
        );
        assert_eq!(actual, expected, "{backend}");
    }
}
