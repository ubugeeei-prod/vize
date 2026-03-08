import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, unref as _unref } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'Avatar',
  props: {
    username: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const { data: gravatarUrl } = useLazyFetch(() => `/api/gravatar/${props.username}`, {
  transform: res => (res.hash ? `/_avatar/${res.hash}?s=128&d=404` : null),
  getCachedData(key, nuxtApp) {
    return nuxtApp.static.data[key] ?? nuxtApp.payload.data[key]
  },
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: "size-16 shrink-0 rounded-full bg-bg-muted border border-border flex items-center justify-center overflow-hidden",
      role: "img",
      "aria-label": `Avatar for ${__props.username}`
    }, [ (_unref(gravatarUrl)) ? (_openBlock(), _createElementBlock("img", {
          key: 0,
          src: _unref(gravatarUrl),
          alt: "",
          width: "64",
          height: "64",
          class: "w-full h-full object-cover"
        })) : (_openBlock(), _createElementBlock("span", {
          key: 1,
          class: "text-2xl text-fg-subtle font-mono",
          "aria-hidden": "true"
        }, _toDisplayString(__props.username.charAt(0).toUpperCase()), 1 /* TEXT */)), _createTextVNode("\n    " + "\n    ") ], 8 /* PROPS */, ["aria-label"]))
}
}

})
