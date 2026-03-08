import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeStyle as _normalizeStyle } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("path", { d: "m7.5 4.27 9 5.15" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("path", { d: "M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("path", { d: "m3.3 7 8.7 5 8.7-5" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("path", { d: "M12 22V12" })

interface Props {
  primaryColor?: string
  title?: string
  description?: string
}

export default /*@__PURE__*/_defineComponent({
  __name: 'Default',
  props: {
    primaryColor: { type: String, required: false, default: '#60a5fa' },
    title: { type: String, required: false, default: 'npmx' },
    description: { type: String, required: false, default: 'a fast }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: "h-full w-full flex flex-col justify-center px-20 bg-[#050505] text-[#fafafa] relative overflow-hidden",
      style: "font-family: 'Geist Mono', sans-serif"
    }, [ _createElementVNode("div", { class: "relative z-10 flex flex-col gap-6" }, [ _createElementVNode("div", { class: "flex items-start gap-4" }, [ _createElementVNode("div", {
            class: "flex items-start justify-center w-16 h-16 p-3.5 rounded-xl bg-gradient-to-tr from-[#3b82f6] shadow-lg",
            style: _normalizeStyle({ backgroundColor: props.primaryColor })
          }, [ _createElementVNode("svg", {
              width: "36",
              height: "36",
              viewBox: "0 0 24 24",
              fill: "none",
              stroke: "white",
              "stroke-width": "2.5",
              "stroke-linecap": "round",
              "stroke-linejoin": "round"
            }, [ _hoisted_1, _hoisted_2, _hoisted_3, _hoisted_4 ]) ], 4 /* STYLE */), _createElementVNode("h1", { class: "text-8xl font-bold" }, [ _createElementVNode("span", {
              class: "opacity-80 tracking-[-0.1em]",
              style: _normalizeStyle([{"margin-left":"-1rem","margin-right":"0.5rem"}, { color: props.primaryColor }])
            }, "./", 4 /* STYLE */), _createTextVNode(_toDisplayString(props.title), 1 /* TEXT */) ]) ]), _createElementVNode("div", {
          class: "flex flex-wrap items-center gap-x-3 text-4xl text-[#a3a3a3]",
          style: "font-family: 'Geist', sans-serif"
        }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(props.description.split(/(\*\*.*?\*\*)/), (part, index) => {
            return (_openBlock(), _createElementBlock(_Fragment, { key: index }, [
              (part.startsWith('**') && part.endsWith('**'))
                ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  class: "px-3 py-1 rounded-lg border font-normal",
                  style: _normalizeStyle({
                color: props.primaryColor,
                backgroundColor: props.primaryColor + '10',
                borderColor: props.primaryColor + '30',
                boxShadow: `0 0 20px ${props.primaryColor}25`,
              })
                }, _toDisplayString(part.replaceAll('**', '')), 1 /* TEXT */))
                : (part.trim() !== '')
                  ? (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(part), 1 /* TEXT */))
                : _createCommentVNode("v-if", true)
            ], 64 /* STABLE_FRAGMENT */))
          }), 128 /* KEYED_FRAGMENT */)) ]) ]), _createElementVNode("div", {
        class: "absolute -top-32 -inset-ie-32 w-[550px] h-[550px] rounded-full blur-3xl",
        style: _normalizeStyle({ backgroundColor: props.primaryColor + '10' })
      }, null, 4 /* STYLE */) ]))
}
}

})
