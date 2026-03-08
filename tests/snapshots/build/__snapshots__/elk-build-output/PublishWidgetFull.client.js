import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "aria-hidden": "true", "i-ri:arrow-down-s-line": "true" })
const _hoisted_2 = { "text-secondary": "true" }
import type { DraftItem } from '#shared/types'
import { formatTimeAgo } from '@vueuse/core'

export default /*@__PURE__*/_defineComponent({
  __name: 'PublishWidgetFull.client',
  setup(__props) {

const route = useRoute()
const { formatNumber } = useHumanReadableNumber()
const timeAgoOptions = useTimeAgoOptions()
const draftKey = ref<DraftKey>('home')
const draftKeys = computed<DraftKey[]>(() => Object.keys(currentUserDrafts.value) as DraftKey[])
const nonEmptyDrafts = computed(() => draftKeys.value
  .filter(i => i !== draftKey.value && !isEmptyDraft(currentUserDrafts.value[i]))
  .map(i => [i, currentUserDrafts.value[i]] as const),
)
watchEffect(() => {
  const quotedStatusId = route.query.quote?.toString()
  if (quotedStatusId) {
    draftKey.value = 'quote'
    currentUserDrafts.value[draftKey.value] = [getDefaultDraftItem({ quotedStatusId })]
  }
  else {
    const key = route.query.draft?.toString() || 'home'
    if (isDraftKey(key))
      draftKey.value = key
  }
})
onDeactivated(() => {
  clearEmptyDrafts()
})
function firstDraftItemOf(drafts: DraftItem | Array<DraftItem>): DraftItem {
  if (Array.isArray(drafts))
    return drafts[0]
  return drafts
}

return (_ctx: any,_cache: any) => {
  const _component_i18n_t = _resolveComponent("i18n-t")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_VDropdown = _resolveComponent("VDropdown")
  const _component_PublishWidgetList = _resolveComponent("PublishWidgetList")

  return (_openBlock(), _createElementBlock("div", {
      flex: "~ col",
      "pb-6": ""
    }, [ _createElementVNode("div", {
        "inline-flex": "",
        "justify-end": "",
        "h-8": ""
      }, [ (nonEmptyDrafts.value.length) ? (_openBlock(), _createBlock(_component_VDropdown, {
            key: 0,
            placement: "bottom-end"
          }, {
            popper: _withCtx(({ hide }) => [
              _createElementVNode("div", { flex: "~ col" }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(nonEmptyDrafts.value, ([key, drafts]) => {
                  return (_openBlock(), _createBlock(_component_NuxtLink, {
                    key: key,
                    border: "b base",
                    "text-left": "",
                    py2: "",
                    px4: "",
                    "hover:bg-active": "",
                    replace: true,
                    to: `/compose?draft=${encodeURIComponent(key)}`,
                    onClick: _cache[0] || (_cache[0] = ($event: any) => (_ctx.hide()))
                  }, {
                    default: _withCtx(() => [
                      _createElementVNode("div", null, [
                        _createElementVNode("div", {
                          flex: "~ gap-1",
                          "items-center": ""
                        }, [
                          _createVNode(_component_i18n_t, { keypath: "compose.draft_title" }, {
                            default: _withCtx(() => [
                              _createElementVNode("code", null, _toDisplayString(key), 1 /* TEXT */)
                            ]),
                            _: 2 /* DYNAMIC */
                          }),
                          (firstDraftItemOf(drafts).lastUpdated)
                            ? (_openBlock(), _createElementBlock("span", {
                              key: 0,
                              "text-secondary": "",
                              "text-sm": ""
                            }, "\n                    &middot; " + _toDisplayString(_unref(formatTimeAgo)(new Date(firstDraftItemOf(drafts).lastUpdated), _unref(timeAgoOptions))), 1 /* TEXT */))
                            : _createCommentVNode("v-if", true)
                        ]),
                        _createElementVNode("div", _hoisted_2, _toDisplayString(_ctx.htmlToText(firstDraftItemOf(drafts).params.status).slice(0, 50)), 1 /* TEXT */)
                      ])
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["replace", "to"]))
                }), 128 /* KEYED_FRAGMENT */))
              ])
            ]),
            default: _withCtx(() => [
              _createElementVNode("button", {
                "btn-text": "",
                flex: "inline center"
              }, [
                _createTextVNode(_toDisplayString(_ctx.$t('compose.drafts', nonEmptyDrafts.value.length, { named: { v: _unref(formatNumber)(nonEmptyDrafts.value.length) } })) + " \n          ", 1 /* TEXT */),
                _hoisted_1
              ])
            ]),
            _: 1 /* STABLE */
          })) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", null, [ _createVNode(_component_PublishWidgetList, {
          expanded: "",
          class: "min-h-100!",
          "draft-key": draftKey.value
        }, null, 8 /* PROPS */, ["draft-key"]) ]) ]))
}
}

})
