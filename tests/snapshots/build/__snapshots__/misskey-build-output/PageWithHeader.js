import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderSlot as _renderSlot, mergeProps as _mergeProps, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { computed, useTemplateRef } from 'vue'
import { scrollInContainer } from '@@/js/scroll.js'
import type { PageHeaderProps } from './MkPageHeader.vue'
import { useScrollPositionKeeper } from '@/composables/use-scroll-position-keeper.js'
import MkSwiper from '@/components/MkSwiper.vue'
import { useRouter } from '@/router.js'
import { prefer } from '@/preferences.js'
import MkTabs from '@/components/MkTabs.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'PageWithHeader',
  props: {
    reversed: { type: Boolean, required: false, default: false },
    swipable: { type: Boolean, required: false, default: true },
    "tab": {}
  },
  emits: ["update:tab"],
  setup(__props: any, { expose: __expose }) {

const props = __props
const tab = _useModel(__props, "tab")
const pageHeaderProps = computed(() => {
	const { reversed, tab, ...rest } = props;
	return rest;
});
const pageHeaderPropsWithoutTabs = computed(() => {
	const { reversed, tabs, ...rest } = props;
	return rest;
});
const rootEl = useTemplateRef('rootEl');
useScrollPositionKeeper(rootEl);
const router = useRouter();
router.useListener('same', () => {
	scrollToTop();
});
function scrollToTop() {
	if (rootEl.value) scrollInContainer(rootEl.value, { top: 0, behavior: 'smooth' });
}
__expose({
	scrollToTop,
})

return (_ctx: any,_cache: any) => {
  const _component_MkPageHeader = _resolveComponent("MkPageHeader")
  const _component_MkStickyContainer = _resolveComponent("MkStickyContainer")

  return (_openBlock(), _createElementBlock("div", {
      ref_key: "rootEl", ref: rootEl,
      class: _normalizeClass(_ctx.reversed ? '_pageScrollableReversed' : '_pageScrollable')
    }, [ _createVNode(_component_MkStickyContainer, null, {
        header: _withCtx(() => [
          (_unref(prefer).s.showPageTabBarBottom && (props.tabs?.length ?? 0) > 0)
            ? (_openBlock(), _createBlock(_component_MkPageHeader, _mergeProps(pageHeaderPropsWithoutTabs.value, { key: 0 }), null, 16 /* FULL_PROPS */))
            : (_openBlock(), _createBlock(_component_MkPageHeader, _mergeProps(pageHeaderProps.value, {
              key: 1,
              tab: tab.value,
              "onUpdate:tab": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
            }), null, 16 /* FULL_PROPS */, ["tab"]))
        ]),
        footer: _withCtx(() => [
          _renderSlot(_ctx.$slots, "footer"),
          (_unref(prefer).s.showPageTabBarBottom && (props.tabs?.length ?? 0) > 0)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(_ctx.$style.footerTabs)
            }, [
              _createVNode(MkTabs, {
                tabs: props.tabs,
                centered: true,
                tabHighlightUpper: true,
                tab: tab.value,
                "onUpdate:tab": _cache[1] || (_cache[1] = ($event: any) => ((tab).value = $event))
              }, null, 8 /* PROPS */, ["tabs", "centered", "tabHighlightUpper", "tab"])
            ]))
            : _createCommentVNode("v-if", true)
        ]),
        default: _withCtx(() => [
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.body)
          }, [
            (_unref(prefer).s.enableHorizontalSwipe && _ctx.swipable && (props.tabs?.length ?? 1) > 1)
              ? (_openBlock(), _createBlock(MkSwiper, {
                key: 0,
                class: _normalizeClass(_ctx.$style.swiper),
                tabs: props.tabs ?? [],
                tab: tab.value,
                "onUpdate:tab": _cache[2] || (_cache[2] = ($event: any) => ((tab).value = $event))
              }, {
                default: _withCtx(() => [
                  _renderSlot(_ctx.$slots, "default")
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["tabs", "tab"]))
              : (_openBlock(), _createElementBlock("slot", { key: 1 }))
          ])
        ]),
        _: 1 /* STABLE */
      }) ], 512 /* NEED_PATCH */))
}
}

})
