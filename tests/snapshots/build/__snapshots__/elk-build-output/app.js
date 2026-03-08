import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("path", { d: "M 0,0.5 C 0,0 0,0 0.5,0 S 1,0 1,0.5 1,1 0.5,1 0,1 0,0.5" })

export default /*@__PURE__*/_defineComponent({
  __name: 'app',
  setup(__props) {

setupPageHeader()
provideGlobalCommands()
const route = useRoute()
if (import.meta.server && !route.path.startsWith('/settings')) {
  const url = useRequestURL()
  useHead({
    meta: [
      { property: 'og:url', content: `${url.origin}${route.path}` },
    ],
  })
}
// We want to trigger rerendering the page when account changes
const key = computed(() => `${currentUser.value?.server ?? currentServer.value}:${currentUser.value?.account.id || ''}`)

return (_ctx: any,_cache: any) => {
  const _component_NuxtLoadingIndicator = _resolveComponent("NuxtLoadingIndicator")
  const _component_NuxtPage = _resolveComponent("NuxtPage")
  const _component_NuxtLayout = _resolveComponent("NuxtLayout")
  const _component_AriaAnnouncer = _resolveComponent("AriaAnnouncer")

  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createVNode(_component_NuxtLoadingIndicator, { color: "repeating-linear-gradient(to right,var(--c-primary) 0%,var(--c-primary-active) 100%)" }), _createVNode(_component_NuxtLayout, { key: key.value }, {
        default: _withCtx(() => [
          _createVNode(_component_NuxtPage)
        ]),
        _: 1 /* STABLE */
      }), _createVNode(_component_AriaAnnouncer), _createElementVNode("svg", {
        absolute: "",
        op0: "",
        width: "0",
        height: "0"
      }, [ _createElementVNode("defs", null, [ _createElementVNode("clipPath", {
            id: "avatar-mask",
            clipPathUnits: "objectBoundingBox"
          }, [ _hoisted_1 ]) ]) ]) ], 64 /* STABLE_FRAGMENT */))
}
}

})
