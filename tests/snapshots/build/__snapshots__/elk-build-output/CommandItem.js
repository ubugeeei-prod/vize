import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue"


const _hoisted_1 = { class: "text-xs text-secondary" }
const _hoisted_2 = { class: "text-xs text-secondary" }
import type { ResolvedCommand } from '~/composables/command'

export default /*@__PURE__*/_defineComponent({
  __name: 'CommandItem',
  props: {
    cmd: { type: null, required: true },
    index: { type: Number, required: true },
    active: { type: Boolean, required: false, default: false }
  },
  emits: ["activate"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit

return (_ctx: any,_cache: any) => {
  const _component_CommandKey = _resolveComponent("CommandKey")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["flex px-3 py-2 my-1 items-center rounded-lg hover:bg-active transition-all duration-65 ease-in-out cursor-pointer scroll-m-10", { 'bg-active': __props.active }]),
      "data-index": __props.index,
      onClick: _cache[0] || (_cache[0] = ($event: any) => (emit('activate')))
    }, [ (__props.cmd.icon) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          "me-2": "",
          class: _normalizeClass(__props.cmd.icon)
        })) : _createCommentVNode("v-if", true), _createElementVNode("div", { class: "flex-1 flex items-baseline gap-2" }, [ _createElementVNode("div", {
          class: _normalizeClass({ 'font-medium': __props.active })
        }, _toDisplayString(__props.cmd.name), 3 /* TEXT, CLASS */), (__props.cmd.description) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "text-xs text-secondary"
          }, _toDisplayString(__props.cmd.description), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]), (__props.cmd.onComplete) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: _normalizeClass(["flex items-center gap-1 transition-all duration-65 ease-in-out", __props.active ? 'opacity-100' : 'opacity-0'])
        }, [ _createElementVNode("div", _hoisted_1, _toDisplayString(_ctx.$t('command.complete')), 1 /* TEXT */), _createVNode(_component_CommandKey, { name: "Tab" }) ])) : _createCommentVNode("v-if", true), (__props.cmd.onActivate) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: _normalizeClass(["flex items-center gap-1 transition-all duration-65 ease-in-out", __props.active ? 'opacity-100' : 'opacity-0'])
        }, [ _createElementVNode("div", _hoisted_2, _toDisplayString(_ctx.$t('command.activate')), 1 /* TEXT */), _createVNode(_component_CommandKey, { name: "Enter" }) ])) : _createCommentVNode("v-if", true) ], 10 /* CLASS, PROPS */, ["data-index"]))
}
}

})
