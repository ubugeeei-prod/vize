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
    let hints = InlayHintService::get_hints(&super::fresh_state(), content, &uri, range);
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
    assert_eq!(
        labels,
        vec![": Ref<number>", ": ComputedRef<number>"],
        "the retained arithmetic control has a known numeric result"
    );
}

#[test]
fn unknown_imported_computed_result_is_not_guessed() {
    let content = "<script setup lang=\"ts\">\nimport { computed } from 'vue';\nimport { readProfile } from './profile';\nconst profile = computed(readProfile);\n</script>\n";
    let uri = Url::parse("file:///unknown-computed.vue").unwrap();
    let range = Range::new(Position::new(0, 0), Position::new(6, 0));
    assert_eq!(
        InlayHintService::get_hints(&super::fresh_state(), content, &uri, range),
        Vec::new()
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
    let hints = InlayHintService::get_hints(&super::fresh_state(), content, &uri, range);
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
