//! Davinci S2 hoist ordering residuals, compared byte-for-byte.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "component_slot_static_props_keep_source_order",
        r#"<div class="not-prose">
  <TooltipRoot v-for="{ name, avatar } of contributors" :key="name">
    <TooltipTrigger as-child>
      <AvatarRoot as-child>
        <a :href="`https://github.com/${name}`">
          <div class="h-12 w-12">
            <AvatarImage :src="avatar" />
            <AvatarFallback class="text-center text-sm font-semibold uppercase" :delay-ms="1000">
              {{ name }}
            </AvatarFallback>
          </div>
        </a>
      </AvatarRoot>
    </TooltipTrigger>
    <TooltipContent class="border border-muted rounded bg-card px-2 py-1 text-xs font-semibold" side="bottom">
      {{ name }}
    </TooltipContent>
  </TooltipRoot>
</div>"#,
    ),
    (
        "conditional_table_static_vnodes_keep_transform_order",
        r#"<div>
  <div v-if="loading"><span text-sm>Loading model...</span></div>
  <template v-else>
    <label><span text-sm>Auto process on upload</span></label>
    <table>
      <thead bg="neutral-100 dark:neutral-800">
        <tr><th px-4 py-3 font-medium>Original</th></tr>
      </thead>
      <tbody>
        <tr v-if="imageItems.length === 0">
          <td colspan="5" px-4 py-8 text-center text-neutral-400>No images uploaded yet</td>
        </tr>
        <tr v-for="item in imageItems" :key="item.file.name">
          <td>{{ item.file.name }}</td>
        </tr>
      </tbody>
    </table>
  </template>
</div>"#,
    ),
];

#[test]
fn s2_hoist_order_matches_the_shipped_dom_lane() {
    support::assert_s2_matches_shipped(BATTERY);
}
