//! Reactive binding inlay hints (`Ref<...>` / `ComputedRef<...>` type
//! previews) and the string-detection helper they rely on.

use super::InlayHintService;
use tower_lsp::lsp_types::{InlayHintLabel, Position, Range, Url};

#[test]
fn test_is_in_string() {
    assert!(!InlayHintService::is_in_string("foo bar", 4));
    assert!(InlayHintService::is_in_string("'foo bar'", 4));
    assert!(InlayHintService::is_in_string("\"foo bar\"", 4));
    assert!(!InlayHintService::is_in_string("\"foo\" bar", 6));
    assert!(InlayHintService::is_in_string("`foo bar`", 4));
}

#[test]
fn test_reactive_binding_inlay_hint() {
    let content = r#"<script setup lang="ts">
import { ref, computed } from 'vue'
const count = ref(0)
const doubled = computed(() => count.value * 2)
</script>
"#;
    let uri = Url::parse("file:///reactive.vue").unwrap();
    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 100,
            character: 0,
        },
    };
    let hints = InlayHintService::get_hints(content, &uri, range);
    let labels: Vec<String> = hints
        .iter()
        .filter_map(|h| match &h.label {
            InlayHintLabel::String(s) => Some(s.clone()),
            _ => None,
        })
        .collect();
    assert!(
        labels.iter().any(|s| s.contains("Ref")),
        "expected a Ref<...> inlay hint, got {labels:?}",
    );
    assert!(
        labels.iter().any(|s| s.contains("ComputedRef")),
        "expected a ComputedRef<...> inlay hint, got {labels:?}",
    );
}

#[test]
fn test_reactive_binding_inlay_hint_resolves_inner_type() {
    // Follow-up to #696: the inlay hint must surface the inferred
    // value type rather than a placeholder. `ref(0)` is number;
    // `ref<string>()` carries an explicit type parameter.
    let content = r#"<script setup lang="ts">
import { ref } from 'vue'
const counter = ref(0)
const label = ref<string>()
</script>
"#;
    let uri = Url::parse("file:///inferred.vue").unwrap();
    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 100,
            character: 0,
        },
    };
    let hints = InlayHintService::get_hints(content, &uri, range);
    let labels: Vec<String> = hints
        .iter()
        .filter_map(|h| match &h.label {
            InlayHintLabel::String(s) => Some(s.clone()),
            _ => None,
        })
        .collect();
    assert!(
        labels.iter().any(|s| s.contains("Ref<number>")),
        "expected Ref<number> inlay hint for ref(0), got {labels:?}",
    );
    assert!(
        labels.iter().any(|s| s.contains("Ref<string>")),
        "expected Ref<string> inlay hint for ref<string>(), got {labels:?}",
    );
}
