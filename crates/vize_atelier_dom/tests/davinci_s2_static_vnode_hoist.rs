//! Static child vnode hoists, compared byte-for-byte.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "nested_select_static_option_before_for",
        r#"<div><select v-model="msg"><option value=""> Select </option><option v-for="item in items" :value="item">{{ item }}</option></select></div>"#,
    ),
    (
        "v_if_static_child_and_text",
        r#"<div v-if="ok"><span></span> x</div>"#,
    ),
    (
        "v_if_static_child_with_attrs",
        r#"<div v-if="ok"><span class="x">hello</span></div>"#,
    ),
    (
        "nested_v_show_static_child",
        r#"<div><span v-show="ok"><b>Downloading update</b></span></div>"#,
    ),
    (
        "airi_titlebar_dynamic_native_ancestor_hoists_static_grandchild",
        r#"<div :class="root"><div flex drag-region><button @click="run"><span>{{ title }}</span></button><div w-full drag-region></div></div></div>"#,
    ),
    (
        "airi_screen_capture_component_slot_hoists_static_break",
        r#"<DialogDescription>{{ summary }}<ol><li>{{ step }}<br><span>{{ note }}</span></li></ol></DialogDescription>"#,
    ),
    (
        "v_for_item_root_hoists_static_child_vnodes",
        r#"<div v-for="item in items" :key="item.id"><span class="icon"></span>{{ item.name }}</div>"#,
    ),
    (
        "component_slot_dynamic_text_parent_keeps_icon_inline",
        r#"<Panel><label :class="['flex', 'items-center', 'gap-2']"><div i-solar:magic-stick-bold-duotone></div>{{ t('settings.pages.modules.artistry.autonomous.title') }}</label><Checkbox></Checkbox></Panel>"#,
    ),
    (
        "branch_child_dynamic_text_parent_keeps_icon_inline",
        r#"<div v-if="isCorsError"><div class="flex items-center gap-2"><div i-solar:shield-warning-bold-duotone></div>{{ t('settings.pages.providers.provider.comfyui.settings.cors.title') }}</div></div>"#,
    ),
    (
        "template_if_slot_keeps_upload_icon_hoist_after_parent_props",
        r#"<InputFileCard><template #default="{ isDragging }"><template v-if="!isDragging"><div flex flex-col items-center><div i-solar:upload-square-line-duotone mb-4 text-5xl text="neutral-400 dark:neutral-500"></div><p font-medium text="neutral-600 dark:neutral-300">{{ t('settings.pages.card.upload') }}</p><p text="neutral-500 dark:neutral-400" mt-2 text-sm>{{ t('settings.pages.card.upload_desc') }}</p></div></template><template v-else><div flex flex-col items-center><div i-solar:upload-minimalistic-bold class="mb-2 text-5xl text-primary-500 dark:text-primary-400"></div><p font-medium text="primary-600 dark:primary-300">{{ t('settings.pages.card.drop_here') }}</p></div></template></template></InputFileCard>"#,
    ),
];

#[test]
fn s2_static_child_vnode_hoists_match_the_shipped_dom_lane() {
    support::assert_s2_matches_shipped(BATTERY);
}
