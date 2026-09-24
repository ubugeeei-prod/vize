//! P2-11 corpus regression: nested children of a `v-for` item keep the shipped
//! props object layout instead of inheriting the item-root compact form.

#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "insta and fixtures use format!; fixtures use std strings"
)]

#[expect(
    dead_code,
    clippy::string_slice,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "shared test support; each binary uses a subset"
)]
mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "audio_dynamic_source_inside_for",
        r#"<audio v-for="(url, index) in recordedUrls" :key="index" controls><source :src="url" type="audio/wav"></audio>"#,
    ),
    (
        "audio_dynamic_source_inside_for_after_static_controls",
        r#"
<div>
  <div>
    <select v-model="constraintId">
      <option value="">
        Select
      </option>
      <option v-for="(item, index) of audioInputs" :key="index" :value="item.deviceId">
        {{ item.label }}
      </option>
    </select>
  </div>
  <div space-x-2>
    <button @click="handleStart">
      Start
    </button>
    <button @click="handleCancel">
      Cancel
    </button>
    <button @click="handleStop">
      Stop
    </button>
  </div>
  <div>
    <audio v-for="(url, index) in recordedUrls" :key="index" controls>
      <source :src="url" type="audio/wav">
    </audio>
  </div>
</div>
"#,
    ),
    (
        "document_global_key_for_item_registers_unused_hoist",
        r#"<div><div v-for="document in docs" :key="document.id" class="pill"><img v-if="document.logoUrl" :src="document.logoUrl" />{{ document.title }}</div></div>"#,
    ),
];

#[test]
fn s2_v_for_child_props_match_the_shipped_dom_lane() {
    support::assert_s2_matches_shipped(BATTERY);
}
