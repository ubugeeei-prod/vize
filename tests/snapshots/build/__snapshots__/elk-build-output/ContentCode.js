import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'ContentCode',
  props: {
    code: { type: String, required: true },
    lang: { type: String, required: false }
  },
  setup(__props: any) {

const raw = computed(() => decodeURIComponent(__props.code).replace(/&#39;/g, '\''))
const langMap: Record<string, string> = {
  js: 'javascript',
  ts: 'typescript',
  vue: 'html',
}
const highlighted = computed(() => {
  return __props.lang ? highlightCode(raw.value, (langMap[lang] || __props.lang) as any) : raw
})

return (_ctx: any,_cache: any) => {
  return (__props.lang)
      ? (_openBlock(), _createElementBlock("pre", {
        key: 0,
        class: "code-block",
        innerHTML: highlighted.value
      }))
      : (_openBlock(), _createElementBlock("pre", {
        key: 1,
        class: "code-block"
      }, _toDisplayString(raw.value), 1 /* TEXT */))
}
}

})
