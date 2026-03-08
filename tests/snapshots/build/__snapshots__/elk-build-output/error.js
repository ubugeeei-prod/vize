import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderSlot as _renderSlot, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "text-lg": "true" }, "\n            Something went wrong\n          ")
const _hoisted_2 = { "text-secondary": "true" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { block: "true", "i-ri:loader-2-fill": "true" })
import type { NuxtError } from '#app'
const defaultMessage = 'Something went wrong'

export default /*@__PURE__*/_defineComponent({
  __name: 'error',
  props: {
    error: { type: null, required: true }
  },
  setup(__props: any) {

// prevent reactive update when clearing error
// add more custom status codes messages here
const errorCodes: Record<number, string> = {
  404: 'Page not found',
}
if (import.meta.dev)
  console.error(__props.error)
const message = __props.error.message ?? errorCodes[error.statusCode!] ?? defaultMessage
const state = ref<'error' | 'reloading'>('error')
async function reload() {
  state.value = 'reloading'
  try {
    clearError({ redirect: currentUser.value ? '/home' : `/${currentServer.value}/public/local` })
  }
  catch (err) {
    console.error(err)
    state.value = 'error'
  }
}

return (_ctx: any,_cache: any) => {
  const _component_NuxtLoadingIndicator = _resolveComponent("NuxtLoadingIndicator")
  const _component_MainTitle = _resolveComponent("MainTitle")
  const _component_MainContent = _resolveComponent("MainContent")
  const _component_NuxtLayout = _resolveComponent("NuxtLayout")
  const _component_AriaAnnouncer = _resolveComponent("AriaAnnouncer")

  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createVNode(_component_NuxtLoadingIndicator, { color: "repeating-linear-gradient(to right,var(--c-primary) 0%,var(--c-primary-active) 100%)" }), _createVNode(_component_NuxtLayout, null, {
        default: _withCtx(() => [
          _createVNode(_component_MainContent, null, {
            title: _withCtx(() => [
              _createVNode(_component_MainTitle, null, {
                default: _withCtx(() => [
                  _createTextVNode("Error")
                ]),
                _: 1 /* STABLE */
              })
            ]),
            default: _withCtx(() => [
              _renderSlot(_ctx.$slots, "default", {}, () => [
                _createElementVNode("form", {
                  p5: "",
                  grid: "",
                  "gap-y-4": "",
                  onSubmit: reload
                }, [
                  _hoisted_1,
                  _createElementVNode("div", _hoisted_2, _toDisplayString(_unref(message)), 1 /* TEXT */),
                  _createElementVNode("button", {
                    flex: "",
                    "items-center": "",
                    "gap-2": "",
                    "justify-center": "",
                    "btn-solid": "",
                    "text-center": "",
                    disabled: state.value === 'reloading'
                  }, [
                    (state.value === 'reloading')
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        block: "",
                        "animate-spin": "",
                        "preserve-3d": ""
                      }, [
                        _hoisted_3
                      ]))
                      : _createCommentVNode("v-if", true),
                    _createTextVNode("\n            " + _toDisplayString(state.value === 'reloading' ? 'Reloading' : 'Reload'), 1 /* TEXT */)
                  ], 8 /* PROPS */, ["disabled"])
                ], 32 /* NEED_HYDRATION */)
              ])
            ]),
            _: 1 /* STABLE */
          })
        ]),
        _: 1 /* STABLE */
      }), _createVNode(_component_AriaAnnouncer) ], 64 /* STABLE_FRAGMENT */))
}
}

})
