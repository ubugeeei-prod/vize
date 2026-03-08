import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString } from "vue"


const _hoisted_1 = { class: "font-mono text-8xl sm:text-9xl font-medium text-fg-subtle mb-4" }
const _hoisted_2 = { class: "font-mono text-2xl sm:text-3xl font-medium mb-4" }
import type { NuxtError } from '#app'

export default /*@__PURE__*/_defineComponent({
  __name: 'error',
  props: {
    error: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const status = computed(() => props.error.statusCode || 500)
const statusText = computed(() => {
  if (props.error.statusMessage) return props.error.statusMessage
  switch (status.value) {
    case 401:
      return 'Unauthorized'
    case 404:
      return 'Page not found'
    case 500:
      return 'Internal server error'
    case 503:
      return 'Service unavailable'
    default:
      return 'Something went wrong'
  }
})
function handleError() {
  clearError({ redirect: '/' })
}
useHead({
  title: `${status.value} - ${statusText.value}`,
})

return (_ctx: any,_cache: any) => {
  const _component_AppHeader = _resolveComponent("AppHeader")
  const _component_AppFooter = _resolveComponent("AppFooter")

  return (_openBlock(), _createElementBlock("div", { class: "min-h-screen flex flex-col bg-bg text-fg" }, [ _createVNode(_component_AppHeader), _createElementVNode("main", { class: "flex-1 container flex flex-col items-center justify-center py-20 text-center" }, [ _createElementVNode("p", _hoisted_1, _toDisplayString(status.value), 1 /* TEXT */), _createElementVNode("h1", _hoisted_2, _toDisplayString(statusText.value), 1 /* TEXT */), (__props.error.message && __props.error.message !== statusText.value) ? (_openBlock(), _createElementBlock("p", {
            key: 0,
            class: "text-fg-muted text-base max-w-md mb-8"
          }, _toDisplayString(__props.error.message), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createElementVNode("button", {
          type: "button",
          class: "font-mono text-sm px-6 py-3 bg-fg text-bg rounded-lg transition-all duration-200 hover:bg-fg/90 active:scale-95",
          onClick: handleError
        }, _toDisplayString(_ctx.$t('common.go_back_home')), 1 /* TEXT */) ]), _createVNode(_component_AppFooter) ]))
}
}

})
