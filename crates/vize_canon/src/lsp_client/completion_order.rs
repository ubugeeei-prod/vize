//! Deterministic order for native completion packets (#10747).
//!
//! The native server can return the same items in a different order for two
//! requests on an unchanged document. Items are only reordered; every field
//! and every duplicate is kept.

use std::cmp::Ordering;

use serde_json::Value;

/// Sort a `CompletionItem[]` or `CompletionList` response in place.
pub(super) fn stabilize(response: &mut Value) {
    let items = match response {
        Value::Array(items) => items,
        Value::Object(list) => match list.get_mut("items") {
            Some(Value::Array(items)) => items,
            _ => return,
        },
        _ => return,
    };
    items.sort_by(compare_items);
}

fn text<'a>(item: &'a Value, key: &str) -> Option<&'a str> {
    item.get(key).and_then(Value::as_str)
}

fn label(item: &Value) -> &str {
    text(item, "label").unwrap_or_default()
}

fn sort_text(item: &Value) -> &str {
    text(item, "sortText").unwrap_or_else(|| label(item))
}

fn encoded(item: &Value) -> Vec<u8> {
    serde_json::to_vec(item).unwrap_or_default()
}

fn compare_items(left: &Value, right: &Value) -> Ordering {
    sort_text(left)
        .cmp(sort_text(right))
        .then_with(|| label(left).cmp(label(right)))
        // Same-label items (auto-imports) differ elsewhere; their full JSON is
        // a total order, and serializing only these ties keeps it cheap.
        .then_with(|| encoded(left).cmp(&encoded(right)))
}

#[cfg(test)]
mod tests {
    use super::stabilize;
    use serde_json::{Value, json};

    fn items() -> Vec<Value> {
        vec![
            json!({ "label": "greet", "sortText": "11", "kind": 3, "data": { "source": "./b" } }),
            json!({ "label": "greet", "sortText": "11", "kind": 3, "data": { "source": "./a" } }),
            json!({ "label": "été", "kind": 6 }),
            json!({ "label": "Array", "sortText": "15", "kind": 7 }),
            json!({ "label": "dup", "sortText": "11" }),
            json!({ "label": "dup", "sortText": "11" }),
            json!({ "label": "message", "sortText": "11", "kind": 6, "labelDetails": { "description": "string" } }),
        ]
    }

    #[test]
    fn every_input_order_yields_one_complete_order() {
        let mut expected = Value::Array(items());
        stabilize(&mut expected);
        let original = items();
        for rotation in 0..original.len() {
            for reverse in [false, true] {
                let mut permuted = original.clone();
                permuted.rotate_left(rotation);
                if reverse {
                    permuted.reverse();
                }
                let mut array = Value::Array(permuted.clone());
                stabilize(&mut array);
                assert_eq!(array, expected);
                let mut list = json!({ "isIncomplete": false, "itemDefaults": { "commitCharacters": ["."] }, "items": permuted });
                stabilize(&mut list);
                assert_eq!(list["items"], expected);
                assert_eq!(list["isIncomplete"], false);
                assert_eq!(list["itemDefaults"], json!({ "commitCharacters": ["."] }));
            }
        }
        let mut sorted = expected.as_array().cloned().unwrap_or_default();
        let mut source = original;
        let key = |item: &Value| serde_json::to_vec(item).unwrap_or_default();
        sorted.sort_by_key(key);
        source.sort_by_key(key);
        assert_eq!(
            sorted, source,
            "items are reordered, never dropped or merged"
        );
    }

    #[test]
    fn non_completion_payloads_are_untouched() {
        for mut value in [Value::Null, json!({ "isIncomplete": true }), json!("text")] {
            let before = value.clone();
            stabilize(&mut value);
            assert_eq!(value, before);
        }
    }
}
