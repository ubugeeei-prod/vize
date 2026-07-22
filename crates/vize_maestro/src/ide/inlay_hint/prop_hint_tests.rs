//! Prop inlay hint behavior: destructured and aliased props, template usage
//! without destructuring, and the rule that hints only attach to real code
//! references (never event-name patterns or literal template text).

use super::InlayHintService;
use tower_lsp::lsp_types::{InlayHintLabel, Position, Range, Url};

#[test]
fn test_props_destructure_analysis() {
    let content = r#"<script setup lang="ts">
const { title, disabled } = defineProps<{
  title: string
  disabled?: boolean
}>()

console.log(title)
</script>

<template>
  <div>{{ title }}</div>
</template>"#;

    let uri = Url::parse("file:///test.vue").unwrap();
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

    // Should have hints for title in script (line 6) and template (line 10)
    assert!(!hints.is_empty(), "Should have inlay hints");

    // Verify all hints are #props.
    for hint in &hints {
        if let InlayHintLabel::String(label) = &hint.label {
            assert_eq!(label, "#props.");
        }
    }
}

#[test]
fn template_literal_text_never_hints_prop_names() {
    let content = r#"<script setup lang="ts">
const { size, state, tone, variant } = defineProps<{
  size: string
  state: string
  tone: string
  variant: string
}>()
</script>

<template>
  <span
    :aria-disabled="state === 'disabled'"
    :class="[
      'tag',
      `tag--size-${size}`,
      `tag--state-${state}`,
      `tag--tone-${tone}`,
      `tag--variant-${variant}`,
    ]"
  >
    <slot />
  </span>
</template>"#;

    let uri = Url::parse("file:///test.vue").unwrap();
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

    // Exactly one hint per real reference: the aria-disabled comparison
    // plus the four `${...}` interpolations. The static `tag--size-`
    // template text must not hint.
    assert_eq!(hints.len(), 5, "{hints:#?}");

    let lines: Vec<&str> = content.lines().collect();
    for hint in &hints {
        let line = lines[hint.position.line as usize];
        let before = &line[..hint.position.character as usize];
        assert!(
            before.ends_with("${") || before.ends_with("=\"") || before.ends_with("\""),
            "hint must sit on a code reference, not literal text: line {:?} col {} ({before:?})",
            hint.position.line,
            hint.position.character
        );
    }
}

#[test]
fn test_props_destructure_with_alias() {
    let content = r#"<script setup lang="ts">
const { title: localTitle } = defineProps<{
  title: string
}>()

console.log(localTitle)
</script>

<template>
  <div>{{ localTitle }}</div>
</template>"#;

    let uri = Url::parse("file:///test.vue").unwrap();
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

    // Should have hints for localTitle (the alias), not title
    assert!(
        !hints.is_empty(),
        "Should have inlay hints for aliased prop"
    );
}

#[test]
fn test_no_hints_in_define_props_type() {
    let content = r#"<script setup lang="ts">
const { title } = defineProps<{
  title: string
}>()
</script>

<template>
  <div>{{ title }}</div>
</template>"#;

    let uri = Url::parse("file:///test.vue").unwrap();
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

    // Check that no hints are in the defineProps type definition
    // (lines 1-3 in script, which is around line 1-4 in the file)
    for hint in &hints {
        assert!(
            hint.position.line > 3,
            "Hint should not be in defineProps type definition, found at line {}",
            hint.position.line
        );
    }
}

#[test]
fn test_no_hints_in_event_name_pattern() {
    // Test that "title" in "update:title" event name does not get a hint
    let content = r#"<script setup lang="ts">
const { title } = defineProps<{
  title: string
}>()

const emit = defineEmits<{
  (e: 'update:title', value: string): void
}>()
</script>

<template>
  <input :value="title" @update:title="emit('update:title', $event)" />
</template>"#;

    let uri = Url::parse("file:///test.vue").unwrap();
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

    // Should have hints for title in :value="title" and possibly template
    // But NOT for title in 'update:title' event names
    for hint in &hints {
        // Get the position in the content
        let line = hint.position.line as usize;
        let lines: Vec<&str> = content.lines().collect();
        if line < lines.len() {
            let line_content = lines[line];
            // Verify the hint is not on a line containing 'update:title' pattern
            // where title immediately follows a colon
            let char_pos = hint.position.character as usize;
            if char_pos > 0 && char_pos <= line_content.len() {
                let before_char = line_content.as_bytes().get(char_pos - 1);
                assert_ne!(
                    before_char,
                    Some(&b':'),
                    "Hint should not be placed after colon (event name pattern)"
                );
            }
        }
    }
}

#[test]
fn test_props_without_destructure_in_template() {
    // Test that props defined without destructuring also get hints in template
    let content = r#"<script setup lang="ts">
const props = defineProps<{
  title: string
  count: number
}>()

// In script, we access via props.title (no hint needed for 'title' alone)
console.log(props.title)
</script>

<template>
  <div>{{ title }}</div>
  <span>{{ count }}</span>
</template>"#;

    let uri = Url::parse("file:///test.vue").unwrap();
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

    // Should have hints for title and count in template (lines 11 and 12)
    // Even though props are not destructured
    let template_hints: Vec<_> = hints.iter().filter(|h| h.position.line >= 11).collect();

    assert!(
        !template_hints.is_empty(),
        "Should have hints for props in template even without destructuring"
    );
}
