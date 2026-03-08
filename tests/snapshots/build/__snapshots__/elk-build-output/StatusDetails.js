import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { ms1: "true", "font-bold": "true", "cursor-pointer": "true" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { "aria-hidden": "true" }, "\n        &middot;\n      ")
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusDetails',
  props: {
    status: { type: null, required: true },
    newer: { type: null, required: false },
    command: { type: Boolean, required: false },
    actions: { type: Boolean, required: false, default: true },
    isNested: { type: Boolean, required: false, default: false }
  },
  emits: ["refetchStatus"],
  setup(__props: any, { emit: $emit }) {

const status = computed(() => {
  if (__props.status.reblog && __props.status.reblog)
    return __props.status.reblog
  return __props.status
})
const createdAt = useFormattedDateTime(status.value.createdAt)
const { t } = useI18n()
useHydratedHead({
  title: () => `${getDisplayName(status.value.account)} ${t('common.in')} ${t('app_name')}: "${removeHTMLTags(status.value.content) || ''}"`,
})

return (_ctx: any,_cache: any) => {
  const _component_StatusActionsMore = _resolveComponent("StatusActionsMore")
  const _component_AccountInfo = _resolveComponent("AccountInfo")
  const _component_AccountHoverWrapper = _resolveComponent("AccountHoverWrapper")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_StatusContent = _resolveComponent("StatusContent")
  const _component_StatusEditIndicator = _resolveComponent("StatusEditIndicator")
  const _component_StatusVisibilityIndicator = _resolveComponent("StatusVisibilityIndicator")
  const _component_StatusEmojiReaction = _resolveComponent("StatusEmojiReaction")
  const _component_StatusActions = _resolveComponent("StatusActions")

  return (_openBlock(), _createElementBlock("div", {
      id: `status-${status.value.id}`,
      flex: "",
      "flex-col": "",
      "gap-2": "",
      pt2: "",
      pb1: "",
      "ps-3": "",
      "pe-4": "",
      relative: "",
      lang: status.value.language ?? undefined,
      "aria-roledescription": "status-details"
    }, [ _createVNode(_component_StatusActionsMore, {
        status: status.value,
        details: true,
        absolute: "",
        "inset-ie-2": "",
        "top-2": "",
        onAfterEdit: _cache[0] || (_cache[0] = ($event: any) => ($emit('refetchStatus')))
      }, null, 8 /* PROPS */, ["status", "details"]), _createVNode(_component_NuxtLink, {
        to: _ctx.getAccountRoute(status.value.account),
        "rounded-full": "",
        "hover:bg-active": "",
        "transition-100": "",
        pe5: "",
        "me-a": ""
      }, {
        default: _withCtx(() => [
          _createVNode(_component_AccountHoverWrapper, { account: status.value.account }, {
            default: _withCtx(() => [
              _createVNode(_component_AccountInfo, { account: status.value.account }, null, 8 /* PROPS */, ["account"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["account"])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["to"]), _createVNode(_component_StatusContent, {
        status: status.value,
        newer: __props.newer,
        context: "details",
        "is-nested": __props.isNested
      }, null, 8 /* PROPS */, ["status", "newer", "is-nested"]), _createElementVNode("div", {
        flex: "~ gap-1",
        "items-center": "",
        "text-secondary": "",
        "text-sm": ""
      }, [ _createElementVNode("div", { flex: "" }, [ _createElementVNode("div", null, _toDisplayString(_unref(createdAt)), 1 /* TEXT */), _createVNode(_component_StatusEditIndicator, {
            status: status.value,
            inline: false
          }, {
            default: _withCtx(() => [
              _createElementVNode("span", _hoisted_1, _toDisplayString(_ctx.$t('state.edited')), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["status", "inline"]) ]), _hoisted_2, _createVNode(_component_StatusVisibilityIndicator, { status: status.value }, null, 8 /* PROPS */, ["status"]), (status.value.application?.name) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            "aria-hidden": "true"
          }, "\n        &middot;\n      ")) : _createCommentVNode("v-if", true), (status.value.application?.website && status.value.application.name) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createVNode(_component_NuxtLink, { to: status.value.application.website }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(status.value.application.name), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to"]) ])) : (status.value.application?.name) ? (_openBlock(), _createElementBlock("div", { key: 1 }, _toDisplayString(status.value.application?.name), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", {
        border: "t base",
        "py-2": ""
      }, [ (__props.actions) ? (_openBlock(), _createBlock(_component_StatusEmojiReaction, {
            key: 0,
            status: status.value,
            details: "",
            command: __props.command
          }, null, 8 /* PROPS */, ["status", "command"])) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", {
        border: "t base",
        "py-2": ""
      }, [ (__props.actions) ? (_openBlock(), _createBlock(_component_StatusActions, {
            key: 0,
            status: status.value,
            details: "",
            command: __props.command
          }, null, 8 /* PROPS */, ["status", "command"])) : _createCommentVNode("v-if", true) ]) ], 8 /* PROPS */, ["id", "lang"]))
}
}

})
