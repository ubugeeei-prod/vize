//! A `v-model` value reaches TypeScript through exactly one reporting copy.
//!
//! Its bare `void (…); // VModel` statement only carries navigation, as the
//! bare statement of a `:prop` does. Without that, `v-model="item.name"` on a
//! component reported TS2339 twice through the content mapper (#11655, misskey
//! `settings/profile.vue`, n8n `WorkflowSettings.vue`).

use std::path::Path;

use crate::batch::{ContentMapperTransform, generate_vue_content_mapper_transform};

/// `protocol::DIRECTIVE_POLICY_IGNORE`.
const IGNORE: usize = 0;

fn ignored(transform: &ContentMapperTransform, at: usize) -> bool {
    transform
        .diagnostic_directives
        .iter()
        .flat_map(|table| &table.directives)
        .any(|directive| {
            let [_, _, start, end, policy, _] = directive.0;
            policy == IGNORE && start <= at && at < end
        })
}

/// Generated copies of the `occurrence`th authored `target` that map back to
/// it verbatim and are not ignored, i.e. the copies TypeScript reports from.
fn reporting_copies(source: &str, target: &str, occurrence: usize) -> usize {
    let transform = generate_vue_content_mapper_transform(Path::new("src/Page.vue"), source)
        .expect("transform");
    let authored = source
        .match_indices(target)
        .nth(occurrence)
        .expect("authored target")
        .0;
    transform
        .mappings
        .iter()
        .filter(|span| {
            let [generated, length, original, original_length, kind, _] = span.0;
            kind == 0
                && original == authored
                && original_length == target.len()
                && length == target.len()
                && !ignored(&transform, generated)
        })
        .count()
}

const COMPONENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/component-v-model-once-11655/Component.vue.txt"
));

#[test]
fn component_model_value_reports_from_one_copy() {
    assert_eq!(reporting_copies(COMPONENT, "state.missing", 0), 1);
    assert_eq!(reporting_copies(COMPONENT, "state.other", 0), 1);
}

const SLOT_CONTENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/component-v-model-once-11655/SlotContent.vue.txt"
));

#[test]
fn slot_scoped_component_model_value_reports_from_one_copy() {
    assert_eq!(reporting_copies(SLOT_CONTENT, "item.name", 0), 1);
}

const NATIVE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/component-v-model-once-11655/Native.vue.txt"
));

/// A native `v-model` has no component prop check, so its statement stays.
#[test]
fn native_model_value_keeps_its_statement() {
    assert_eq!(reporting_copies(NATIVE, "state.missing", 0), 1);
}
