import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { h1px: "true", bg: "gray/20", my2: "true" })
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusEditPreview',
  props: {
    edit: { type: null, required: true }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  const _component_AccountInlineInfo = _resolveComponent("AccountInlineInfo")
  const _component_StatusBody = _resolveComponent("StatusBody")
  const _component_StatusMedia = _resolveComponent("StatusMedia")
  const _component_StatusSpoiler = _resolveComponent("StatusSpoiler")

  return (_openBlock(), _createElementBlock("div", {
      px3: "",
      "py-4": "",
      flex: "~ col"
    }, [ _createElementVNode("div", {
        "text-center": "",
        flex: "~ row gap-1 wrap"
      }, [ _createVNode(_component_AccountInlineInfo, { account: __props.edit.account }, null, 8 /* PROPS */, ["account"]), _createElementVNode("span", null, _toDisplayString(_ctx.$t('status_history.edited', [_ctx.useFormattedDateTime(__props.edit.createdAt).value])), 1 /* TEXT */) ]), _hoisted_1, _createVNode(_component_StatusSpoiler, { enabled: __props.edit.sensitive }, {
        spoiler: _withCtx(() => [
          _createTextVNode(_toDisplayString(__props.edit.spoilerText), 1 /* TEXT */)
        ]),
        default: _withCtx(() => [
          _createVNode(_component_StatusBody, { status: __props.edit }, null, 8 /* PROPS */, ["status"]),
          (__props.edit.mediaAttachments.length)
            ? (_openBlock(), _createBlock(_component_StatusMedia, {
              key: 0,
              status: __props.edit
            }, null, 8 /* PROPS */, ["status"]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["enabled"]) ]))
}
}

})
