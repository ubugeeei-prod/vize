import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'Toggle.server',
  props: {
    label: { type: String, required: true },
    description: { type: String, required: false },
    justify: { type: String, required: false, default: 'between' },
    reverseOrder: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  const _component_SkeletonBlock = _resolveComponent("SkeletonBlock")

  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createElementVNode("div", {
        class: _normalizeClass(["grid items-center gap-4 py-1 -my-1 grid-cols-[auto_1fr_auto]", [__props.justify === 'start' ? 'justify-start' : '']]),
        style: _normalizeStyle(
        props.reverseOrder
          ? 'grid-template-areas: \'toggle . label-text\''
          : 'grid-template-areas: \'label-text . toggle\''
      )
      }, [ (props.reverseOrder) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createVNode(_component_SkeletonBlock, {
              class: "h-6 w-11 shrink-0 rounded-full",
              style: "grid-area: toggle"
            }), (__props.label) ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                class: "text-sm text-fg font-medium text-start",
                style: "grid-area: label-text"
              }, _toDisplayString(__props.label), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ (__props.label) ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                class: "text-sm text-fg font-medium text-start",
                style: "grid-area: label-text"
              }, _toDisplayString(__props.label), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createVNode(_component_SkeletonBlock, {
              class: "h-6 w-11 shrink-0 rounded-full",
              style: "grid-area: toggle; justify-self: end"
            }) ], 64 /* STABLE_FRAGMENT */)) ], 6 /* CLASS, STYLE */), (__props.description) ? (_openBlock(), _createElementBlock("p", {
          key: 0,
          class: "text-sm text-fg-muted mt-2"
        }, _toDisplayString(__props.description), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */))
}
}

})
