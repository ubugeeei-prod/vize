import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withKeys as _withKeys } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", null, "#")
const _hoisted_2 = { "hover:underline": "true" }
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'TagCard',
  props: {
    tag: { type: null, required: true }
  },
  setup(__props: any) {

const to = computed(() => {
  const { hostname, pathname } = new URL(__props.tag.url)
  return `/${hostname}${pathname}`
})
const router = useRouter()
function onclick(evt: MouseEvent | KeyboardEvent) {
  const path = evt.composedPath() as HTMLElement[]
  const el = path.find(el => ['A', 'BUTTON'].includes(el.tagName?.toUpperCase()))
  const text = window.getSelection()?.toString()
  if (!el && !text)
    go(evt)
}
function go(evt: MouseEvent | KeyboardEvent) {
  if (evt.metaKey || evt.ctrlKey)
    window.open(to.value)
  else
    router.push(to.value)
}

return (_ctx: any,_cache: any) => {
  const _component_TagActionButton = _resolveComponent("TagActionButton")
  const _component_CommonTrending = _resolveComponent("CommonTrending")
  const _component_CommonTrendingCharts = _resolveComponent("CommonTrendingCharts")

  return (_openBlock(), _createElementBlock("div", {
      block: "",
      p4: "",
      "hover:bg-active": "",
      flex: "",
      "justify-between": "",
      "cursor-pointer": "",
      "flex-gap-2": "",
      onClick: onclick,
      onKeydown: _withKeys(onclick, ["enter"])
    }, [ _createElementVNode("div", {
        flex: "",
        "flex-gap-2": ""
      }, [ _createVNode(_component_TagActionButton, { tag: __props.tag }, null, 8 /* PROPS */, ["tag"]), _createElementVNode("div", null, [ _createElementVNode("h4", {
            flex: "",
            "items-center": "",
            "text-size-base": "",
            "leading-normal": "",
            "font-medium": "",
            "line-clamp-1": "",
            "break-all": "",
            "ws-pre-wrap": ""
          }, [ _createElementVNode("bdi", null, [ _hoisted_1, _createElementVNode("span", _hoisted_2, _toDisplayString(__props.tag.name), 1 /* TEXT */) ]) ]), (__props.tag.history) ? (_openBlock(), _createBlock(_component_CommonTrending, {
              key: 0,
              history: __props.tag.history,
              "text-sm": "",
              "text-secondary": "",
              "line-clamp-1": "",
              "ws-pre-wrap": "",
              "break-all": ""
            }, null, 8 /* PROPS */, ["history"])) : _createCommentVNode("v-if", true) ]) ]), (__props.tag.history) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          flex: "",
          "items-center": ""
        }, [ _createVNode(_component_CommonTrendingCharts, { history: __props.tag.history }, null, 8 /* PROPS */, ["history"]) ])) : _createCommentVNode("v-if", true) ], 32 /* NEED_HYDRATION */))
}
}

})
