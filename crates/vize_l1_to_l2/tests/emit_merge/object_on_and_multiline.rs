use super::*;

#[test]
fn an_object_bind_beside_object_on_uses_merge_props() {
    assert_eq!(
        assembled(r#"<div v-bind="obj" v-on="handlers"></div>"#),
        "\
const { mergeProps: _mergeProps, toHandlers: _toHandlers, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _mergeProps(obj, _toHandlers(handlers, true)), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn a_v_if_with_object_on_merges_the_branch_key() {
    assert_eq!(
        assembled(r#"<div v-if="ok" v-on="handlers">x</div>"#),
        "\
const { mergeProps: _mergeProps, toHandlers: _toHandlers, openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(\"div\", _mergeProps({ key: 0 }, _toHandlers(handlers, true)), \"x\", 16 /* FULL_PROPS */))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn object_bind_modifiers_preserve_the_spread_expression() {
    assert_eq!(
        assembled(r#"<div v-bind.prop="obj"></div>"#),
        assembled(r#"<div v-bind="obj"></div>"#)
    );
    assert_eq!(
        assembled(r#"<div id="x" v-bind.attr.camel="obj"></div>"#),
        assembled(r#"<div id="x" v-bind="obj"></div>"#)
    );
}

#[test]
fn a_v_for_spread_key_uses_a_multiline_key_object() {
    assert_eq!(
        assembled(r#"<div v-for="n in list" :key="i" v-bind="obj"></div>"#),
        "\
const { mergeProps: _mergeProps, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(list, (n) => {
    return (_openBlock(), _createElementBlock(\"div\", _mergeProps({
      key: i
    }, obj), null, 16 /* FULL_PROPS */))
  }), 128 /* KEYED_FRAGMENT */))
}"
    );
}

#[test]
fn a_v_for_spread_keeps_later_multi_prop_object_multiline() {
    assert_eq!(
        assembled(r#"<Foo v-for="(domain, index) in domains" :key="domain.key" v-bind="index === 0 ? layout : {}" :label="index === 0 ? 'Domains' : ''" :name="['domains', index, 'value']" :rules="{ required: true }" />"#),
        pin("\
const { resolveComponent: _resolveComponent, mergeProps: _mergeProps, openBlock: _openBlock, createBlock: _createBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(domains, (domain, index) => {
    return (_openBlock(), _createBlock(_component_Foo, _mergeProps({
      key: domain.key
    }, index === 0 ? layout : {}, {
      label: index === 0 ? 'Domains' : '',
      name: ['domains', index, 'value'],
      rules: { required: true }
    }), null, 16 /* FULL_PROPS */, [\"label\", \"name\"]))
  }), 128 /* KEYED_FRAGMENT */))
}")
    );
}

#[test]
fn a_v_for_spread_merges_duplicate_handlers_after_the_spread() {
    assert_eq!(
        assembled(r#"<li v-for="item in items" :key="item.id" v-bind="item.props" @keydown="a" @keydown.enter.prevent="b"></li>"#),
        "\
const { withKeys: _withKeys, withModifiers: _withModifiers, mergeProps: _mergeProps, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
    return (_openBlock(), _createElementBlock(\"li\", _mergeProps({
      key: item.id
    }, item.props, {
      onKeydown: [a, _withKeys(_withModifiers(b, [\"prevent\"]), [\"enter\"])]
    }), null, 48 /* FULL_PROPS, NEED_HYDRATION */, [\"onKeydown\"]))
  }), 128 /* KEYED_FRAGMENT */))
}"
    );
}

#[test]
fn post_spread_single_object_after_branch_object_keeps_shipped_multiline_segment() {
    assert_shipped_parity(
        r#"<button v-if="show" :disabled="isDisabled" v-bind="ptm('button')" data-pc-section="root"></button>"#,
    );
}

#[test]
fn post_spread_single_object_after_unsuppressed_branch_key_keeps_shipped_multiline_segment() {
    assert_shipped_parity(r#"<template v-if="ok"><Foo v-bind="bag" :title="title" /></template>"#);
}

#[test]
fn v_for_pre_spread_single_object_keeps_shipped_multiline_segment() {
    assert_shipped_parity(
        r#"<circle v-for="{ cx, cy, r } in points" data-cy="point" v-bind="{ cx, cy }" :r="r"></circle>"#,
    );
}

#[test]
fn object_on_spread_keeps_authored_multiline_handler_padding() {
    assert_shipped_parity(
        r#"<input
  type="button"
  v-bind="{ ...$attrs }"
  v-on="
    Object.assign({}, {
      input: handleInput,
    }, $listeners)
  "
/>"#,
    );
}

#[test]
fn merge_object_ts_line_comment_value_keeps_authored_padding() {
    assert_shipped_parity(
        r#"<Foo
  v-bind="props"
  :id="subContext.triggerId"
  :ref="
    (vnode: Element | ComponentPublicInstance | null) => {
      if (!vnode) return undefined
      // @ts-ignore
      subContext?.onTriggerChange(vnode?.$el);
      return undefined
    }
  "
  aria-haspopup="menu"
/>"#,
    );
}

#[test]
fn merge_object_multiline_class_keeps_authored_padding() {
    assert_shipped_parity(
        r#"<Foo
  data-slot="trigger"
  v-bind="forwarded"
  :class="
    cn(
      'inline-flex items-center',
      props.class,
    )
  "
></Foo>"#,
    );
}

#[test]
fn trailing_static_attr_after_v_for_object_bind_keeps_shipped_multiline_segment() {
    assert_shipped_parity(
        r#"<rect v-for="(item, index) in items" v-bind="{
  x: item.x,
  y: item.y,
}" :key="index" stroke-width="2"></rect>"#,
    );
}

#[test]
fn trailing_static_class_after_object_bind_keeps_shipped_multiline_segment() {
    assert_shipped_parity(
        r#"<div v-if="ok"></div><div v-else :style="{
  backgroundImage: `url(${src})`,
  backgroundSize: contain ? 'contain' : 'cover',
}" v-bind="{ role: alt ? 'img' : null, 'aria-label': alt }" class="absolute top-0 left-0 h-full w-full bg-center bg-no-repeat"></div>"#,
    );
}

#[test]
fn middle_segment_between_object_bind_and_object_on_keeps_shipped_multiline_segment() {
    assert_shipped_parity(
        r#"<td v-for="header in headers" v-bind="header.rowAttrs" :key="header.value" :align="header.align" v-on="header.rowEvents || {}"></td>"#,
    );
}

#[test]
fn middle_static_attr_between_object_bind_and_object_on_keeps_shipped_inline_segment() {
    assert_shipped_parity(
        r#"<Foo v-bind="calendarProps" data-testid="calendar" v-on="{ 'update:modelValue': emit }"><template #default="{ grid }">{{ grid }}</template></Foo>"#,
    );
}

#[test]
fn trailing_event_after_object_bind_keeps_shipped_multiline_segment() {
    assert_shipped_parity(
        r#"<el-button v-if="current > 0" size="small" :type="mergedType" v-bind="filterButtonProps(prevButtonProps)" @click="onPrev" />"#,
    );
}

#[test]
fn static_and_dynamic_multiline_class_keeps_shipped_array_shape() {
    assert_shipped_parity(
        r#"<div :class="
  data.disableBlock ? 'bg-box-transparent' : block.category.color
" class="mr-4 inline-flex"></div>"#,
    );
}

#[test]
fn guard_reactive_props_keeps_authored_multiline_bind_padding() {
    assert_shipped_parity(
        r#"<component
  :is="component.target ? 'a' : 'router-link'"
  v-bind="
    component.target
      ? { href: component.path, target: component.target }
      : { to: getLocalizedPathname(component.path, isZhCN) }
  "
></component>"#,
    );
}

#[test]
fn dynamic_bind_value_decodes_attribute_entities_like_the_shipped_lane() {
    assert_shipped_parity(
        r#"<UserDialog :subtitle="`&quot;${user.name}&quot; has ${user.likes} likes`" />"#,
    );
}
