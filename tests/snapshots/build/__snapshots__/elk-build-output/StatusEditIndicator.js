import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderSlot as _renderSlot, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { "text-center": "true", mb1: "true" }
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusEditIndicator',
  props: {
    status: { type: null, required: true },
    inline: { type: Boolean, required: true }
  },
  setup(__props: any) {

const editedAt = computed(() => __props.status.editedAt)
const formatted = useFormattedDateTime(editedAt)

return (_ctx: any,_cache: any) => {
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")
  const _component_StatusEditHistory = _resolveComponent("StatusEditHistory")
  const _component_CommonDropdown = _resolveComponent("CommonDropdown")

  return (editedAt.value)
      ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (__props.inline) ? (_openBlock(), _createBlock(_component_CommonTooltip, {
            key: 0,
            content: _ctx.$t('status.edited', [_unref(formatted)])
          }, {
            default: _withCtx(() => [
              _createTextVNode("\n       \n      "),
              _createElementVNode("time", {
                title: editedAt.value,
                datetime: editedAt.value,
                "font-bold": "",
                underline: "",
                "decoration-dashed": "",
                "text-secondary": ""
              }, " * ", 8 /* PROPS */, ["title", "datetime"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["content"])) : (_openBlock(), _createBlock(_component_CommonDropdown, { key: 1 }, {
            popper: _withCtx(() => [
              _createElementVNode("div", {
                "text-sm": "",
                p2: ""
              }, [
                _createElementVNode("div", _hoisted_1, _toDisplayString(_ctx.$t('status.edited', [_unref(formatted)])), 1 /* TEXT */),
                _createVNode(_component_StatusEditHistory, { status: __props.status }, null, 8 /* PROPS */, ["status"])
              ])
            ]),
            default: _withCtx(() => [
              _renderSlot(_ctx.$slots, "default")
            ]),
            _: 1 /* STABLE */
          })) ], 64 /* STABLE_FRAGMENT */))
      : _createCommentVNode("v-if", true)
}
}

})
