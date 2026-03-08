import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'search',
  setup(__props) {

const keys = useMagicKeys()
const { t } = useI18n()
useHydratedHead({
  title: () => t('nav.search'),
})
const search = ref<{ input?: HTMLInputElement }>()
watchEffect(() => {
  if (search.value?.input)
    search.value?.input?.focus()
})
onActivated(() => search.value?.input?.focus())
onDeactivated(() => search.value?.input?.blur())
watch(keys['/'], (v) => {
  // focus on input when '/' is up to avoid '/' being typed
  if (!v)
    search.value?.input?.focus()
})

return (_ctx: any,_cache: any) => {
  const _component_MainTitle = _resolveComponent("MainTitle")
  const _component_SearchWidget = _resolveComponent("SearchWidget")
  const _component_MainContent = _resolveComponent("MainContent")

  return (_openBlock(), _createBlock(_component_MainContent, null, {
      title: _withCtx(() => [
        _createVNode(_component_MainTitle, {
          as: "router-link",
          to: "/search",
          icon: "i-ri:search-line rtl-flip"
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_ctx.$t('nav.search')), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          px2: "",
          mt3: ""
        }, [
          (_ctx.isHydrated)
            ? (_openBlock(), _createBlock(_component_SearchWidget, {
              key: 0,
              ref: "search",
              "m-1": ""
            }, null, 512 /* NEED_PATCH */))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
