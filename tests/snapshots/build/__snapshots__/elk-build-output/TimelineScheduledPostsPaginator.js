import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, createSlots as _createSlots, toDisplayString as _toDisplayString, mergeProps as _mergeProps, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { italic: "true" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:external-link-fill": "true" })
import type { CommonPaginator } from '#components'
import type { mastodon } from 'masto'
import type { ComponentExposed } from 'vue-component-type-helpers'
import { DynamicScrollerItem } from 'vue-virtual-scroller'
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'

type PaginatorRef = ComponentExposed<typeof CommonPaginator>

export default /*@__PURE__*/_defineComponent({
  __name: 'TimelineScheduledPostsPaginator',
  props: {
    paginator: { type: null, required: true },
    stream: { type: null, required: false },
    context: { type: null, required: false },
    account: { type: null, required: false },
    buffer: { type: Number, required: false, default: 10 },
    endMessage: { type: [Boolean, String], required: false, default: true }
  },
  setup(__props: any) {

// @ts-expect-error missing types
const { formatNumber } = useHumanReadableNumber()
const virtualScroller = usePreferences('experimentalVirtualScroller')
const paginatorRef = ref<PaginatorRef>()
const showOriginSite = computed(() =>
  __props.account && __props.account.id !== currentUser.value?.account.id && getServerName(__props.account) !== currentServer.value,
)

return (_ctx: any,_cache: any) => {
  const _component_StatusScheduledCard = _resolveComponent("StatusScheduledCard")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_CommonPaginator = _resolveComponent("CommonPaginator")

  return (_openBlock(), _createBlock(_component_CommonPaginator, _mergeProps({ paginator: __props.paginator, stream: __props.stream, buffer: __props.buffer, endMessage: __props.endMessage }, {
      ref_key: "paginatorRef", ref: paginatorRef,
      "virtual-scroller": _unref(virtualScroller)
    }), _createSlots({ _: 2 /* DYNAMIC */ }, [ {
        name: "updater",
        fn: _withCtx(({ number, update }) => [
          _createElementVNode("button", {
            id: "elk_show_new_items",
            "py-4": "",
            border: "b base",
            flex: "~ col",
            "p-3": "",
            "w-full": "",
            "text-primary": "",
            "font-bold": "",
            onClick: _cache[0] || (_cache[0] = (...args) => (_ctx.update && _ctx.update(...args)))
          }, _toDisplayString(_ctx.$t('timeline.show_new_items', number, { named: { v: _unref(formatNumber)(number) } })), 1 /* TEXT */)
        ])
      }, {
        name: "default",
        fn: _withCtx(({ item, active }) => [
          _createVNode(_resolveDynamicComponent(_unref(virtualScroller) ? _unref(DynamicScrollerItem) : 'article'), {
            item: item,
            active: active
          }, {
            default: _withCtx(() => [
              _createVNode(_component_StatusScheduledCard, {
                item: item,
                onDeleted: _cache[1] || (_cache[1] = ($event: any) => (paginatorRef.value?.removeEntry($event)))
              }, null, 8 /* PROPS */, ["item"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["item", "active"])
        ])
      }, (__props.context === 'account') ? {
          name: "done",
          fn: _withCtx(({ items }) => [
            (showOriginSite.value || items.length === 0)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                p5: "",
                "text-secondary": "",
                "text-center": "",
                flex: "",
                "flex-col": "",
                "items-center": "",
                gap1: ""
              }, [
                (showOriginSite.value)
                  ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                    _createElementVNode("span", _hoisted_1, _toDisplayString(_ctx.$t('timeline.view_older_posts')), 1 /* TEXT */),
                    _createVNode(_component_NuxtLink, {
                      href: __props.account.url,
                      target: "_blank",
                      external: "",
                      flex: "~ gap-1",
                      "items-center": "",
                      "text-primary": "",
                      hover: "underline text-primary-active"
                    }, {
                      default: _withCtx(() => [
                        _hoisted_2,
                        _createTextVNode("\n            "),
                        _createTextVNode(_toDisplayString(_ctx.$t('menu.open_in_original_site')), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["href"])
                  ], 64 /* STABLE_FRAGMENT */))
                  : (items.length === 0)
                    ? (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(_ctx.$t('timeline.no_posts')), 1 /* TEXT */))
                  : _createCommentVNode("v-if", true)
              ]))
              : _createCommentVNode("v-if", true)
          ]),
          key: "0"
        } : undefined ]), 1040 /* FULL_PROPS, DYNAMIC_SLOTS */, ["virtual-scroller"]))
}
}

})
