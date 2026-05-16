import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'home',
  setup(__props) {

definePageMeta({
  middleware: 'auth',
  alias: ['/signin/callback'],
})
const route = useRoute()
const router = useRouter()
if (import.meta.client && route.path === '/signin/callback')
  router.push('/home')
const { t } = useI18n()
useHydratedHead({
  title: () => t('nav.home'),
})

return (_ctx: any,_cache: any) => {
  const _component_MainTitle = _resolveComponent("MainTitle")
  const _component_TimelineHome = _resolveComponent("TimelineHome")
  const _component_MainContent = _resolveComponent("MainContent")

  return (_openBlock(), _createBlock(_component_MainContent, null, {
      title: _withCtx(() => [
        _createVNode(_component_MainTitle, {
          as: "router-link",
          to: "/home",
          icon: "i-ri:home-5-line"
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_ctx.$t('nav.home')), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })
      ]),
      default: _withCtx(() => [
        (_ctx.isHydrated)
          ? (_openBlock(), _createBlock(_component_TimelineHome, { key: 0 }))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
