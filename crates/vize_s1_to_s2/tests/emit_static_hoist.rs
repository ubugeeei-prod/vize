//! Focused Davinci parity pins for static-props/static-vnode hoist order.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed;
use vize_s0::Allocator;
use vize_s1_to_s2::emit_dom;

fn shipped(source: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, old) = vize_atelier_dom::compile_template(&allocator, source);
    let blocking: Vec<_> = errors
        .iter()
        .filter(|error| !error.is_compatibility_notice())
        .collect();
    assert!(blocking.is_empty(), "{source:?}: {blocking:?}");
    format!("{}\n{}", old.preamble, old.code)
}

fn emitted(source: &str) -> String {
    with_transformed(source, |lowered, _folio, facts, _budget| {
        emit_dom(lowered, facts)
            .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
            .assembled()
            .to_string()
    })
}

fn assert_shipped_parity(source: &str) {
    assert_eq!(emitted(source), shipped(source), "{source}");
}

#[test]
fn nested_not_static_props_hoist_in_legacy_order() {
    assert_shipped_parity(r#"<div><section class="panel"><span>{{ msg }}</span></section></div>"#);
}

#[test]
fn dynamic_parent_hoists_static_children_instead_of_render_cache() {
    assert_shipped_parity(
        r#"<div><section class="panel" @click="ok"><span></span></section></div>"#,
    );
}

#[test]
fn cached_static_child_formats_multikey_props_like_the_shipped_snapshot() {
    assert_shipped_parity(
        r#"<div class="root">{{ msg }}<span id="cta" class="pill"></span></div>"#,
    );
}

#[test]
fn static_ref_child_stays_inline_when_static_cache_is_enabled() {
    assert_shipped_parity(
        r#"<aside class="seed"></aside><main><div ref="canvasContainerRef"></div></main>"#,
    );
}

#[test]
fn cached_static_children_array_formats_multikey_props_like_the_shipped_snapshot() {
    assert_shipped_parity(
        r#"<div class="root"><span id="hero" class="title"></span><span data-panel="intro" aria-hidden="true"></span></div>"#,
    );
}

#[test]
fn static_ref_child_stays_inline_under_dynamic_parent_hoist() {
    assert_shipped_parity(
        r#"<section @mousemove="track"><div ref="silhouette" class="silhouette"></div></section>"#,
    );
}

#[test]
fn static_bind_props_hoist_with_nested_dynamic_descendants() {
    assert_shipped_parity(
        r#"<div><section class="panel" :id="'fixed'"><span>{{ msg }}</span></section></div>"#,
    );
}

#[test]
fn nested_event_and_model_children_match_legacy_block_shape() {
    assert_shipped_parity(r#"<div><button @click="run">Run</button></div>"#);
    assert_shipped_parity(
        r#"<div class="password-input"><input v-model="password" :key="`password-${showPassword}`" /></div>"#,
    );
}

#[test]
fn html_parent_with_svg_child_keeps_legacy_parent_vnode_shape() {
    assert_shipped_parity(
        r#"<div><span class="menu-button" aria-label="Menu" @click="toggleMenu"><svg viewBox="0 0 24 24"><path d="M0 0h1" /></svg></span></div>"#,
    );
    assert_shipped_parity(
        r#"<label><span class="mark"><svg v-if="checked" viewBox="0 0 24 24"><path d="M0 0h1" /></svg></span></label>"#,
    );
    assert_shipped_parity(
        r#"<section><article class="chart-card"><h2>{{ title }}</h2><svg viewBox="0 0 100 40"><polyline :points="points" /></svg></article></section>"#,
    );
}

#[test]
fn cached_static_child_with_dynamic_text_uses_legacy_parent_block() {
    assert_shipped_parity(
        r#"<section><div class="cta"><svg viewBox="0 0 24 24"><path d="M0 0h1" /></svg>{{ label }}</div></section>"#,
    );
    assert_shipped_parity(
        r#"<section><a href="/docs" class="cta"><svg viewBox="0 0 24 24"><path d="M0 0h1" /></svg>{{ label }}</a></section>"#,
    );
}

#[test]
fn v_for_item_props_hoist_is_registered_but_not_used() {
    assert_shipped_parity(r#"<div v-for="item in list" class="row">{{ item }}</div>"#);
    assert_shipped_parity(
        r#"<template v-for="item in list"><section class="row"><span>{{ item }}</span></section></template>"#,
    );
}

#[test]
fn v_for_component_item_props_hoist_is_registered_but_not_used() {
    assert_shipped_parity(r#"<Foo v-for="item in list" class="row"><span>{{ item }}</span></Foo>"#);
    assert_shipped_parity(
        r#"<NodeListInline v-for="document of filteredNodes" :key="document.id" :document="document" class="line-item" />"#,
    );
    assert_shipped_parity(
        r#"<NodeCard v-for="document in filteredNodes" :key="document.id" :node="document" />"#,
    );
}

#[test]
fn component_slot_dynamic_static_name_props_use_legacy_hoist() {
    assert_shipped_parity(
        r#"<NuxtLink :to="`/dashboard/docs/${document.id}`" class="name">{{ document.name }}</NuxtLink>"#,
    );
    assert_shipped_parity(
        r#"<NuxtLink :to="`/dashboard/docs/${node.id}`" class="name">{{ node.name }}</NuxtLink>"#,
    );
    assert_shipped_parity(
        r#"<footer v-if="document"><NuxtLink :to="`/dashboard/docs/edit/${document.id}`" class="edit-link">{{ document.name }}</NuxtLink></footer>"#,
    );
    assert_shipped_parity(
        r#"<TooltipRoot v-for="{ name } of contributors" :key="name"><span>{{ name }}</span></TooltipRoot>"#,
    );
    assert_shipped_parity(
        r#"<NodeResourceInline v-for="diagram in diagrams" :key="diagram.id" :node="diagram" class="line-item" />"#,
    );
}

#[test]
fn component_patchless_bind_props_use_legacy_hoist() {
    assert_shipped_parity(r#"<a-radio-button :value="'month'">Month</a-radio-button>"#);
    assert_shipped_parity(r#"<a-col :xs="24" :sm="12" :lg="8" :xl="6">{{ title }}</a-col>"#);
    assert_shipped_parity(
        r#"<a-calendar><template #headerRender="{ type }"><a-radio-group :value="type"><a-radio-button value="month">Month</a-radio-button><a-radio-button value="year">Year</a-radio-button></a-radio-group></template></a-calendar>"#,
    );
    assert_shipped_parity(
        r#"<template v-for="component in group.children" :key="component.title"><a-col :xs="24" :sm="12" :lg="8" :xl="6"><component :is="component.target ? 'a' : 'router-link'">{{ component.title }}</component></a-col></template>"#,
    );
}

#[test]
fn foreign_namespace_component_props_hoist_without_static_children() {
    assert_shipped_parity(r#"<svg><Foo id="x" /></svg>"#);
    assert_shipped_parity(r#"<svg><motion.path fill="transparent" stroke="red" /></svg>"#);
}

#[test]
fn foreign_namespace_builtin_component_props_hoist_without_static_children() {
    assert_shipped_parity(
        r#"<svg><TransitionGroup name="fade"><path v-for="arrow in arrows" :key="arrow.id" :class="{ [arrow.type]: true }" :d="arrow.d" stroke-linecap="round" /></TransitionGroup></svg>"#,
    );
}

#[test]
fn foreign_namespace_fragment_root_props_use_legacy_hoist() {
    assert_shipped_parity(
        r#"<Foo /><svg absolute op0 width="0" height="0"><defs><clipPath id="avatar-mask" clipPathUnits="objectBoundingBox"><path d="M 0,0.5 C 0,0 0,0 0.5,0 S 1,0 1,0.5 1,1 0.5,1 0,1 0,0.5" /></clipPath></defs></svg>"#,
    );
}

#[test]
fn forwarded_slot_component_global_constant_props_use_legacy_hoist() {
    assert_shipped_parity(
        r#"<MkCondensedLine :minScale="2 / 3"><slot name="label"></slot></MkCondensedLine>"#,
    );
}

#[test]
fn static_option_with_undefined_value_hoists_like_legacy_global_constant() {
    assert_shipped_parity(
        r#"<select v-model="locale"><option :value="undefined"></option><option :value="'de-DE'">de-DE</option></select>"#,
    );
}

#[test]
fn template_for_component_root_skips_non_hoistable_static_props() {
    assert_shipped_parity(
        r#"<template v-for="group in groups" :key="group.id"><ComboboxGroup :class="['overflow-x-hidden']"><span></span></ComboboxGroup></template>"#,
    );
    assert_shipped_parity(
        r#"<template v-for="i in 10" :key="i"><a-col v-show="open" :span="8"><a-input /></a-col></template>"#,
    );
}

#[test]
fn component_static_key_with_dynamic_event_keeps_legacy_hoist() {
    assert_shipped_parity(r#"<a-menu-item key="1" @click="open">Open</a-menu-item>"#);
    assert_shipped_parity(
        r#"<a-tree><template #title="{ key: treeKey, title }"><a-dropdown><template #overlay><a-menu @click="({ key: menuKey }) => onContextMenuClick(treeKey, menuKey)"><a-menu-item key="1">1st menu item</a-menu-item><a-menu-item key="2">2nd menu item</a-menu-item></a-menu></template></a-dropdown></template></a-tree>"#,
    );
}

#[test]
fn static_cache_uses_component_prop_hoists_for_plain_breaks() {
    assert_shipped_parity(
        r#"<div><a-radio-button :value="'month'">Month</a-radio-button><a-switch v-model:checked="showLine" /><br><br></div>"#,
    );
    assert_shipped_parity(
        r#"<div><div style="margin-bottom: 16px">showLine:<a-switch v-model:checked="showLine" /><br><br>showIcon:<a-switch v-model:checked="showIcon" /></div></div>"#,
    );
    assert_shipped_parity(
        r#"<div>
    <div style="margin-bottom: 16px">
      showLine:
      <a-switch v-model:checked="showLine" />
      <br />
      <br />
      showIcon:
      <a-switch v-model:checked="showIcon" />
    </div>
  </div>"#,
    );
    assert_shipped_parity(
        r#"
  <div>
    <div style="margin-bottom: 16px">
      showLine:
      <a-switch v-model:checked="showLine" />
      <br />
      <br />
      showIcon:
      <a-switch v-model:checked="showIcon" />
    </div>
  </div>
"#,
    );
    assert_shipped_parity(
        r#"
  <div>
    <div style="margin-bottom: 16px">
      showLine:
      <a-switch v-model:checked="showLine" />
      <br />
      <br />
      showIcon:
      <a-switch v-model:checked="showIcon" />
    </div>
    <a-tree>
      <template #title="{ dataRef }">
        <template v-if="dataRef.key === '0-0-0-1'">
          <div>multiple line title</div>
          <div>multiple line title</div>
        </template>
        <template v-else>{{ dataRef.title }}</template>
      </template>
    </a-tree>
  </div>
"#,
    );
}

#[test]
fn pure_static_vnode_hoist_drops_descendant_patchless_binds() {
    assert_shipped_parity(
        r#"<Foo><button :style="{ transform: 'none' }"><span class="x" :style="{ color: 'red' }"></span></button></Foo>"#,
    );
    assert_shipped_parity(
        r#"<Foo><button :style="{ transform: 'none' }"><span class="x" :id="'y'"></span></button></Foo>"#,
    );
}

#[test]
fn branch_roots_with_mixed_text_still_hoist_static_children() {
    assert_shipped_parity(
        r#"<li :data-status="status"><div><span v-if="pending"><span class="animate-spin"></span>{{ label }}</span></div></li>"#,
    );
    assert_shipped_parity(
        r#"<div><div v-if="current">{{ current }}</div><div v-else-if="listening"><div class="animate-pulse"></div>{{ label }}</div></div>"#,
    );
}

#[test]
fn template_if_static_branch_hoist_enables_static_sibling_cache() {
    assert_shipped_parity(r#"<div><template v-if="ok"><span></span></template><p>x</p></div>"#);
    assert_shipped_parity(
        r#"<section><h2>Nasa Picture of the day</h2><template v-if="ok"><div class="spinner"></div></template></section>"#,
    );
    assert_shipped_parity(r#"<div><span v-if="ok"></span><p>x</p></div>"#);
}

#[test]
fn nested_element_if_inside_template_if_still_hoists_static_branch_children() {
    assert_shipped_parity(
        r#"<div><template v-if="hide"><div><b v-if="s"><i class="a"></i> {{ one }}</b><b v-else><i class="b"></i> {{ two }}</b></div></template></div>"#,
    );
}

#[test]
fn template_if_branch_roots_with_mixed_text_keep_static_children_inline() {
    assert_shipped_parity(
        r#"<template v-if="item.options"><span>{{ item.value }}<a href="/">more</a></span></template><template v-else><a href="/all">all</a></template>"#,
    );
}

#[test]
fn component_slot_mixed_text_parent_keeps_nested_static_child_inline() {
    assert_shipped_parity(
        r#"<Main><Section><h1 :id="id" class="bv-no-focus-ring"><span class="bd-content-title">{{ groupTitle }} <span class="small text-muted">- table of contents</span></span></h1></Section></Main>"#,
    );
    assert_shipped_parity(
        r#"<Main><Section><h1 :id="id" class="bv-no-focus-ring"><span class="bd-content-title">{{ groupTitle }} <span class="small text-muted">- table of contents</span></span></h1></Section><Section><b-list-group-item v-for="page in pages" :key="page.slug" active-class=""><strong class="text-primary">{{ page.title }}</strong></b-list-group-item></Section></Main>"#,
    );
    assert_shipped_parity(
        r#"<Main><Section tag="header"><h1 :id="id" class="bv-no-focus-ring" tabindex="-1"><span class="bd-content-title">{{ groupTitle }} <span class="small text-muted">- table of contents</span></span></h1><p v-if="groupDescription" class="bd-lead">{{ groupDescription }}</p></Section><Section><b-list-group tag="nav" :aria-label="`${groupTitle} section navigation`" class="mb-5"><b-list-group-item v-for="page in pages" :key="page.slug" :to="`/docs/${slug}/${page.slug}`" active-class=""><strong class="text-primary">{{ page.title }}</strong> - <b-badge v-if="page.new" variant="success">NEW</b-badge><span class="text-muted">{{ page.description }}</span><b-badge v-if="page.version" variant="secondary">v{{ page.version }}</b-badge></b-list-group-item></b-list-group></Section></Main>"#,
    );
}

#[test]
fn v_for_item_v_once_stays_in_the_legacy_plain_item_path() {
    assert_shipped_parity(
        r#"<Foo v-for="item in list" v-once :key="item.id" :name="item.name" />"#,
    );
    assert_shipped_parity(
        r#"<span v-for="item in list" v-once :key="item.id">{{ item.name }}</span>"#,
    );
}

#[test]
fn v_for_descendant_dynamic_text_props_stay_inline() {
    assert_shipped_parity(
        r#"<div v-for="group in itemsGroups" v-if="visible" :key="group.key" :class="group.key"><h2 class="d-flex align-items-center mb-3">{{ group.label }}<span class="badge badge-pill badge-default ml-2">{{ items[group.key].length }}</span></h2></div>"#,
    );
}
