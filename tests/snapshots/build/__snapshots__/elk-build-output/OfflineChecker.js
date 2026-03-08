import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:wifi-off-line": "true" })

export default /*@__PURE__*/_defineComponent({
  __name: 'OfflineChecker',
  setup(__props) {

const online = useOnline()

return (_ctx: any,_cache: any) => {
  return (!_unref(online))
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        "w-full": "",
        "min-h-30px": "",
        px4: "",
        py3: "",
        "text-primary": "",
        "bg-base": "",
        border: "t base",
        flex: "~ gap-2 center"
      }, [ _hoisted_1, _createTextVNode("\n    "), _toDisplayString(_ctx.$t('common.offline_desc')) ]))
      : _createCommentVNode("v-if", true)
}
}

})
