import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { "text-secondary": "true" }, ":")
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { "text-secondary": "true" }, ":")

export interface SearchEmoji {
  title: string
  src: string
}

export default /*@__PURE__*/_defineComponent({
  __name: 'SearchEmojiInfo',
  props: {
    emoji: { type: Object, required: true }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      flex: "~ gap3",
      "items-center": "",
      "text-base": ""
    }, [ _createElementVNode("img", {
        width: "20",
        height: "20",
        src: __props.emoji.src,
        loading: "lazy"
      }, null, 8 /* PROPS */, ["src"]), _createElementVNode("span", {
        shrink: "",
        "overflow-hidden": "",
        "leading-none": "",
        "text-base": ""
      }, [ _hoisted_1, _createTextVNode(_toDisplayString(__props.emoji.title), 1 /* TEXT */), _hoisted_2 ]) ]))
}
}

})
