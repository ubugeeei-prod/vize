import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:hashtag": "true", "text-secondary": "true", "text-lg": "true" })
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'SearchHashtagInfo',
  props: {
    hashtag: { type: null, required: true }
  },
  setup(__props: any) {

const totalTrend = computed(() =>
  __props.hashtag.history?.reduce((total: number, item) => total + (Number(item.accounts) || 0), 0),
)

return (_ctx: any,_cache: any) => {
  const _component_CommonTrending = _resolveComponent("CommonTrending")
  const _component_CommonTrendingCharts = _resolveComponent("CommonTrendingCharts")

  return (_openBlock(), _createElementBlock("div", {
      flex: "",
      "flex-row": "",
      "items-center": "",
      gap2: "",
      relative: ""
    }, [ _createElementVNode("div", {
        "w-10": "",
        "h-10": "",
        "flex-none": "",
        "rounded-full": "",
        "bg-active": "",
        flex: "",
        "place-items-center": "",
        "place-content-center": ""
      }, [ _hoisted_1 ]), _createElementVNode("div", {
        flex: "",
        "flex-col": ""
      }, [ _createElementVNode("span", null, _toDisplayString(__props.hashtag.name), 1 /* TEXT */), (__props.hashtag.history) ? (_openBlock(), _createBlock(_component_CommonTrending, {
            key: 0,
            history: __props.hashtag.history,
            "text-xs": "",
            "text-secondary": "",
            truncate: ""
          }, null, 8 /* PROPS */, ["history"])) : _createCommentVNode("v-if", true) ]), (totalTrend.value && __props.hashtag.history) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          absolute: "",
          "left-15": "",
          "right-0": "",
          "top-0": "",
          "bottom-4": "",
          op35: "",
          flex: "",
          "place-items-center": "",
          "place-content-center": "",
          "ml-auto": ""
        }, [ _createVNode(_component_CommonTrendingCharts, {
            history: __props.hashtag.history,
            width: 150,
            height: 20,
            "text-xs": "",
            "text-secondary": "",
            "h-full": "",
            "w-full": ""
          }, null, 8 /* PROPS */, ["history", "width", "height"]) ])) : _createCommentVNode("v-if", true) ]))
}
}

})
