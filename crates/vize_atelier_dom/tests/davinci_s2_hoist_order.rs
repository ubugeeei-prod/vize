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
    (
        "scoped_slot_component_static_bind_props_wait_for_keyed_children",
        r#"<a-tree><template #title="{ key }"><a-dropdown :trigger="['contextmenu']"><template #overlay><a-menu><a-menu-item key="1">one</a-menu-item><a-menu-item key="2">two</a-menu-item></a-menu></template><span>{{ key }}</span></a-dropdown></template></a-tree>"#,
    ),
    (
        "transition_forwarded_slot_static_props_wait_for_fallback_hoists",
        r#"<div class="container"><svg><path d="M0 0z" /></svg><Transition name="fade"><div v-if="open" class="content"><slot /></div></Transition></div>"#,
    ),
    (
        "nested_component_responsive_props_wait_before_parent_static_bind",
        r#"<section class="markdown"><template v-for="group in menuItems" :key="group.title"><div class="components-overview"><h2 class="ant-typography components-overview-group-title"><a-space align="center">{{ isZhCN ? group.title : group.enTitle }}<a-tag style="display: block">{{ group.children.length }}</a-tag></a-space></h2><a-row :gutter="[24, 24]"><template v-for="component in group.children" :key="component.title"><a-col :xs="24" :sm="12" :lg="8" :xl="6"><component :is="component.target ? 'a' : 'router-link'" v-bind="component.target ? { href: component.path, target: component.target } : { to: getLocalizedPathname(component.path, isZhCN) }"><a-card size="small" class="components-overview-card"><template #title><div class="components-overview-title">{{ component.title }}{{ isZhCN ? component.subtitle : '' }}</div></template><div class="components-overview-img"><img :src="isDark && component.coverDark ? component.coverDark : component.cover" :alt="component.title" /></div></a-card></component></a-col></template></a-row></div></template></section>"#,
    ),
    (
        "slot_component_static_attrs_stay_inline_after_prior_hoist",
        r#"<a-menu><a-menu-item-group v-if="isZhCN" key="advanced" title="advanced"><a-menu-item key="surely-table"><a href="https://www.surelyvue.com" target="_blank" rel="noopener noreferrer" style="position: relative">Surely Table</a></a-menu-item></a-menu-item-group><template v-for="m in menus"><template v-if="m.children"><a-menu-item-group :key="m.order" :title="m.title"><template v-for="n in m.children"><a-menu-item v-if="n.path" :key="n.path"><router-link :to="n.path"><span>{{ n.title }}</span><span v-if="isZhCN" class="chinese">{{ n.subtitle }}</span></router-link><a-tag v-if="n.tag" color="green" style="margin-left: auto">{{ n.tag }}</a-tag></a-menu-item></template></a-menu-item-group></template></template></a-menu>"#,
    ),
    (
        "slot_component_static_bind_props_stay_inline_after_prior_hoist",
        r#"<a-row class="list-row" :gutter="24"><a-col v-for="item in renderData" :key="item.id" class="list-col" :xs="12" :sm="12" :md="12" :lg="6" :xl="6" :xxl="6"><CardWrap :loading="loading" :title="item.title" :description="item.description"><template #skeleton><a-skeleton :animation="true"><a-skeleton-line :widths="['50%', '100%']" :rows="4" /></a-skeleton></template></CardWrap></a-col></a-row>"#,
    ),
    (
        "table_column_static_props_stay_inline_after_static_vnode",
        r#"<b-table :data="data" :loading="loading" paginated backend-pagination :total="total" :per-page="perPage" @page-change="onPageChange" backend-sorting :default-sort="[sortField, sortOrder]" @sort="onSort"><b-table-column field="original_title" label="Title" sortable v-slot="props">{{ props.row.original_title }}</b-table-column><b-table-column field="vote_average" label="Vote Average" numeric sortable v-slot="props"><span class="tag" :class="type(props.row.vote_average)">{{ props.row.vote_average }}</span></b-table-column><b-table-column field="vote_count" label="Vote Count" numeric sortable v-slot="props">{{ props.row.vote_count }}</b-table-column></b-table>"#,
    ),
    (
        "component_static_attr_props_precede_direct_static_slot_vnodes",
        r#"<Head title="Test Head Component"><meta name="viewport" content="width=device-width, initial-scale=1" /><meta name="undefined" :content="undefined" /><meta name="number" :content="0" /></Head><h1 :style="{ fontSize: '40px' }">Head Component</h1>"#,
    ),
    (
        "component_static_bind_props_precede_direct_static_slot_vnode",
        r#"<div><AspectRatio :ratio="16 / 9"><img class="Image" src="x" alt="y"></AspectRatio></div>"#,
    ),
    (
        "component_mixed_props_precede_direct_static_slot_vnode",
        r#"<DocSectionText v-bind="$attrs"><p>Intro</p><p>More <a href="x">link</a></p></DocSectionText><div class="card"><Panel header="Header" toggleable unstyled :pt="{ root: 'x', header: (options) => ({ id: 'myPanelHeader' }) }"><p class="m-0">Lorem ipsum</p></Panel></div><DocSectionCode :code="code" />"#,
    ),
    (
        "branch_component_static_props_stay_inline_before_static_slot_vnode",
        r#"<Form v-if="show" action="/dump/post" method="post"><input type="text" name="name" id="name" value="John" /></Form>"#,
    ),
    (
        "branch_child_component_static_props_follow_shipped_hoist_order",
        r#"<div v-if="show"><BranchRatio :ratio="16 / 9"><img class="Image" src="x" alt="y"></BranchRatio></div>"#,
    ),
    (
        "for_component_root_slot_static_props_stay_inline",
        r#"<ListItem v-for="entry in entries" label="fixed" v-slot="{ value }">{{ value }}</ListItem>"#,
    ),
    (
        "if_branch_component_root_slot_static_props_stay_inline",
        r#"<Panel v-if="open" label="fixed" v-slot="{ value }"><span class="value">{{ value }}</span></Panel>"#,
    ),
];

#[test]
fn s2_hoist_order_matches_the_shipped_dom_lane() {
    support::assert_s2_matches_shipped(BATTERY);
}
