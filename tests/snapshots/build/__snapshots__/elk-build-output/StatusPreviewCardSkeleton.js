import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, normalizeClass as _normalizeClass } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "w-full": "true", "h-full": "true", class: "skeleton-loading-bg" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { flex: "true", class: "skeleton-loading-bg", "h-5": "true", w: "4/5", rounded: "true" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("div", { flex: "true", class: "skeleton-loading-bg", "h-4": "true", "w-full": "true", rounded: "true" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("div", { "sm:flex": "true", hidden: "true", class: "skeleton-loading-bg", "h-4": "true", w: "2/5", rounded: "true" })

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusPreviewCardSkeleton',
  props: {
    square: { type: Boolean, required: false },
    root: { type: Boolean, required: false }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      "of-hidden": "",
      class: _normalizeClass({
        'flex': __props.square,
        'p-4': __props.root,
        'rounded-lg border border-base': !__props.root,
      })
    }, [ _createElementVNode("div", {
        flex: "",
        "flex-col": "",
        "display-block": "",
        "of-hidden": "",
        border: "base",
        class: _normalizeClass({
          'sm:(min-w-32 w-32 h-32) min-w-22 w-22 h-22 border-r': __props.square,
          'w-full aspect-[1.91] border-b': !__props.square,
          'rounded-lg': __props.root,
        })
      }, [ _hoisted_1 ], 2 /* CLASS */), _createElementVNode("div", {
        px3: "",
        "max-h-2xl": "",
        "flex-1": "",
        flex: "",
        "flex-col": "",
        "flex-gap-2": "",
        "sm:flex-gap-3": "",
        class: _normalizeClass([
          __props.root ? 'py2.5 sm:py3' : 'py3  justify-center sm:justify-start',
        ])
      }, [ _createElementVNode("div", {
          flex: "",
          "h-4": "",
          "w-30": "",
          rounded: "",
          class: _normalizeClass(["skeleton-loading-bg", __props.root ? '' : 'hidden sm:block'])
        }, null, 2 /* CLASS */), _hoisted_2, _createElementVNode("div", { flex: "~ col gap-2" }, [ _hoisted_3, _hoisted_4 ]) ], 2 /* CLASS */) ], 2 /* CLASS */))
}
}

})
