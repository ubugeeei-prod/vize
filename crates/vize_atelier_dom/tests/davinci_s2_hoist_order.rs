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
    (
        "component_static_class_array_props_stay_inline",
        r#"<section><Menu><Content align="end" side="top" :side-offset="8" :class="['z-50', 'bg-white']"><Item /></Content></Menu><button><div class="i-stop"></div></button><button><div class="i-trash"></div></button></section>"#,
    ),
    (
        "dialog_content_static_props_stay_inline_before_slot_child_hoists",
        r#"<slot v-bind="{ hasPermissions }" /><DialogRoot :open="showDialog"><DialogPortal><DialogOverlay class="fixed inset-0" /><DialogContent flex="~ col items-start gap-4" class="fixed left-1/2 top-1/2"><DialogTitle class="m-0 text-lg font-semibold">{{ title }}</DialogTitle><DialogDescription>{{ body }}<ol mt-4 list-decimal pl-5 text-sm><li>one</li></ol></DialogDescription></DialogContent></DialogPortal></DialogRoot>"#,
    ),
    (
        "for_component_child_hoists_keep_avatar_props_order",
        r#"<div v-for="(author, index) of authors" :key="index"><AvatarRoot class="size-10 inline-flex select-none items-center justify-center overflow-hidden rounded-full bg-neutral-100 align-middle dark:bg-neutral-800"><AvatarImage class="h-full w-full rounded-[inherit] object-cover" :src="author.avatar || author.avatarFallback" :alt="`${author.displayName}'s avatar`" /><AvatarFallback class="h-full w-full flex items-center justify-center bg-white text-sm text-primary font-medium leading-1 dark:bg-neutral-800 dark:text-neutral-300" :delay-ms="600" as-child>{{ [author.displayName.charAt(0).toUpperCase(), author.displayName.charAt(1).toUpperCase()].join('') }}</AvatarFallback></AvatarRoot></div>"#,
    ),
    (
        "scoped_slot_component_child_hoists_keep_parent_props_inline",
        r#"<CursorMomentum v-slot="{ currentValue }"><Volumed :perspective="800" transform="rotateX(45deg) translateY(3px)"><TestDummyMarkerFlat :style="{ transform: `rotate(${currentValue}deg)` }" /></Volumed></CursorMomentum>"#,
    ),
    (
        "slot_if_branch_static_vnodes_keep_shipped_order",
        r#"<a-auto-complete><template #option="item"><template v-if="item.options"><span>{{ item.value }}<a style="float: right" href="https://www.google.com/search?q=antd" target="_blank" rel="noopener noreferrer">more</a></span></template><template v-else-if="item.value === 'all'"><a href="https://www.google.com/search?q=ant-design-vue" target="_blank" rel="noopener noreferrer">View all results</a></template></template></a-auto-complete>"#,
    ),
    (
        "template_for_component_bind_props_keep_shipped_order_before_later_static_props",
        r#"<a-form><a-row :gutter="24"><template v-for="i in 10" :key="i"><a-col v-show="expand || i <= 6" :span="8"><a-form-item><a-input /></a-form-item></a-col></template></a-row><a-row><a-col :span="24" style="text-align: right"><a-button type="primary" html-type="submit">Search</a-button></a-col></a-row></a-form>"#,
    ),
];

#[test]
fn s2_hoist_order_matches_the_shipped_dom_lane() {
    support::assert_s2_matches_shipped(BATTERY);
}
