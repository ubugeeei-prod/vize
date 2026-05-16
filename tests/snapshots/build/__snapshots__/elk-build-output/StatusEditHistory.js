import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span")
import type { mastodon } from 'masto'
import { formatTimeAgo } from '@vueuse/core'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusEditHistory',
  props: {
    status: { type: null, required: true }
  },
  setup(__props: any) {

const paginator = useMastoClient().v1.statuses.$select(__props.status.id).history.list()
function showHistory(edit: mastodon.v1.StatusEdit) {
  openEditHistoryDialog(edit)
}
const timeAgoOptions = useTimeAgoOptions()
// TODO: rework, this is only reversing the first page of edits
function reverseHistory(items: mastodon.v1.StatusEdit[]) {
  return [...items].reverse()
}

return (_ctx: any,_cache: any) => {
  const _component_i18n_t = _resolveComponent("i18n-t")
  const _component_CommonDropdownItem = _resolveComponent("CommonDropdownItem")
  const _component_StatusEditHistorySkeleton = _resolveComponent("StatusEditHistorySkeleton")
  const _component_CommonPaginator = _resolveComponent("CommonPaginator")

  return (_openBlock(), _createBlock(_component_CommonPaginator, {
      paginator: _unref(paginator),
      "key-prop": "createdAt",
      preprocess: reverseHistory
    }, {
      default: _withCtx(({ items, item, index }) => [
        _createVNode(_component_CommonDropdownItem, {
          px: "0.5",
          onClick: _cache[0] || (_cache[0] = ($event: any) => (showHistory(_ctx.item)))
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_ctx.getDisplayName(item.account)), 1 /* TEXT */),
            _createTextVNode("\n\n        "),
            (index === items.length - 1)
              ? (_openBlock(), _createBlock(_component_i18n_t, {
                key: 0,
                keypath: "status_history.created"
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(formatTimeAgo)(new Date(item.createdAt), _unref(timeAgoOptions))), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }))
              : (_openBlock(), _createBlock(_component_i18n_t, {
                key: 1,
                keypath: "status_history.edited"
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(formatTimeAgo)(new Date(item.createdAt), _unref(timeAgoOptions))), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }))
          ]),
          _: 1 /* STABLE */
        })
      ]),
      loading: _withCtx(() => [
        _createVNode(_component_StatusEditHistorySkeleton),
        _createVNode(_component_StatusEditHistorySkeleton, { op50: "" }),
        _createVNode(_component_StatusEditHistorySkeleton, { op25: "" })
      ]),
      done: _withCtx(() => [
        _hoisted_1
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["paginator", "preprocess"]))
}
}

})
