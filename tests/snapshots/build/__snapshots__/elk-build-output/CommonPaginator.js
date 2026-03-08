import { useSlots as _useSlots } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, renderSlot as _renderSlot, toDisplayString as _toDisplayString, mergeProps as _mergeProps, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { p5: "true", "text-secondary": "true", italic: "true", "text-center": "true" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:arrow-down-line": "true" })
import type { mastodon } from 'masto'
import { DynamicScroller } from 'vue-virtual-scroller'
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'

export default /*@__PURE__*/_defineComponent({
  __name: 'CommonPaginator',
  props: {
    paginator: { type: null, required: true },
    keyProp: { type: null, required: false, default: 'id' },
    virtualScroller: { type: Boolean, required: false, default: false },
    stream: { type: null, required: false },
    eventType: { type: String, required: false },
    preprocess: { type: Function, required: false },
    endMessage: { type: [Boolean, String], required: false, default: true }
  },
  setup(__props: any, { expose: __expose }) {

// @ts-expect-error missing types
const { t } = useI18n()
const nuxtApp = useNuxtApp()
const {
  items,
  prevItems,
  update,
  state,
  endAnchor,
  error,
  canLoadMore,
} = usePaginator(__props.paginator, toRef(() => __props.stream), __props.eventType, __props.preprocess)
nuxtApp.hook('elk-logo:click', () => {
  update()
  nuxtApp.$scrollToTop()
})
function createEntry(item: any) {
  items.value = [...items.value, __props.preprocess?.([item]) ?? item]
}
function updateEntry(item: any) {
  const id = item[keyProp]
  const index = items.value.findIndex(i => (i as any)[keyProp] === id)
  if (index > -1)
    items.value = [...items.value.slice(0, index), __props.preprocess?.([item]) ?? item, ...items.value.slice(index + 1)]
}
function removeEntry(entryId: any) {
  items.value = items.value.filter(i => (i as any)[keyProp] !== entryId)
}
__expose({ createEntry, removeEntry, updateEntry })

return (_ctx: any,_cache: any) => {
  const _component_TimelineSkeleton = _resolveComponent("TimelineSkeleton")

  return (_openBlock(), _createElementBlock("div", null, [ (_unref(prevItems).length) ? (_openBlock(), _createElementBlock("slot", _mergeProps({ number: _unref(prevItems).length, update: _unref(update) }, {
          key: 0,
          name: "updater"
        }))) : _createCommentVNode("v-if", true), _renderSlot(_ctx.$slots, "items", {
        name: "items",
        items: _unref(items)
      }, () => [ (__props.virtualScroller) ? (_openBlock(), _createBlock(DynamicScroller, {
            key: 0,
            items: _unref(items),
            "min-item-size": 200,
            "key-field": __props.keyProp,
            "page-mode": ""
          }, {
            default: _withCtx(({ item, active, index }) => [
              _renderSlot(_ctx.$slots, "default", _mergeProps({ key: item[__props.keyProp] }, {
                item: item,
                active: active,
                older: _unref(items)[index + 1],
                newer: _unref(items)[index - 1],
                index: index,
                items: _unref(items)
              }))
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["items", "min-item-size", "key-field"])) : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(items), (item, index) => {
              return (_openBlock(), _createElementBlock("slot", _mergeProps({ key: item[__props.keyProp] }, {
                item: item,
                older: _unref(items)[index + 1],
                newer: _unref(items)[index - 1],
                index: index,
                items: _unref(items)
              }), 16 /* FULL_PROPS */, ["item", "older", "newer", "index", "items"]))
            }), 256 /* UNKEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)) ]), _createElementVNode("div", { ref_key: "endAnchor", ref: endAnchor }, null, 512 /* NEED_PATCH */), (_unref(state) === 'loading') ? (_openBlock(), _createElementBlock("slot", {
          key: 0,
          name: "loading"
        }, [ _createVNode(_component_TimelineSkeleton) ])) : (_unref(state) === 'done' && __props.endMessage !== false) ? (_openBlock(), _createElementBlock("slot", {
            key: 1,
            name: "done",
            items: _unref(items)
          }, [ _createElementVNode("div", _hoisted_1, _toDisplayString(_unref(t)(typeof __props.endMessage === 'string' && _unref(items).length <= 0 ? __props.endMessage : 'common.end_of_list')), 1 /* TEXT */) ])) : (_unref(state) === 'error') ? (_openBlock(), _createElementBlock("div", {
            key: 2,
            p5: "",
            "text-secondary": ""
          }, _toDisplayString(_unref(t)('common.error')) + ": " + _toDisplayString(_unref(error)), 1 /* TEXT */)) : _createCommentVNode("v-if", true), (_unref(state) !== 'loading' && _unref(state) !== 'done' && !_unref(canLoadMore)) ? (_openBlock(), _createElementBlock("button", {
          key: 0,
          flex: "~ gap-1 center",
          "w-full": "",
          "my-6": "",
          "py-6": "",
          "btn-text": "",
          "rounded-lg": "",
          bg: "base",
          "filter-saturate-0": "",
          "hover:filter-saturate-100": "",
          onClick: _cache[0] || (_cache[0] = ($event: any) => (canLoadMore.value = true))
        }, [ _hoisted_2, _createTextVNode("\n      "), _toDisplayString(_ctx.$t('timeline.load_more')) ])) : _createCommentVNode("v-if", true) ]))
}
}

})
