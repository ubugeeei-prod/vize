import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'LogoImg',
  props: {
    src: { type: [null, String, Object], required: true },
    alt: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  return (typeof __props.src === 'string')
      ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createElementVNode("img", {
          src: __props.src,
          loading: "lazy",
          height: "36",
          class: "w-auto block h-full",
          alt: __props.alt
        }, null, 8 /* PROPS */, ["src", "alt"]) ]))
      : (__props.src.light === 'auto')
        ? (_openBlock(), _createElementBlock("div", {
          key: 1,
          class: "h-full"
        }, [ _createElementVNode("img", {
            src: __props.src.dark,
            loading: "lazy",
            height: "36",
            class: "w-auto block light:invert light:grayscale h-full",
            alt: __props.alt
          }, null, 8 /* PROPS */, ["src", "alt"]) ]))
      : (_openBlock(), _createElementBlock("div", {
        key: 2,
        class: "h-full"
      }, [ _createElementVNode("img", {
          src: __props.src.dark,
          loading: "lazy",
          height: "36",
          class: "w-auto block light:hidden h-full",
          alt: __props.alt
        }, null, 8 /* PROPS */, ["src", "alt"]), _createElementVNode("img", {
          src: __props.src.light,
          loading: "lazy",
          height: "36",
          class: "w-auto block hidden light:block h-full",
          alt: __props.alt
        }, null, 8 /* PROPS */, ["src", "alt"]) ]))
}
}

})
