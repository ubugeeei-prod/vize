import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


const _hoisted_1 = { "sr-only": "true" }
const _hoisted_2 = { "text-xl": "true" }

export default /*@__PURE__*/_defineComponent({
  __name: 'notifications',
  setup(__props) {

definePageMeta({
  middleware: 'auth',
})
const { t } = useI18n()
useHydratedHead({
  title: () => `${t('settings.notifications.notifications.label')} | ${t('settings.notifications.label')} | ${t('nav.settings')}`,
})

return (_ctx: any,_cache: any) => {
  const _component_MainTitle = _resolveComponent("MainTitle")
  const _component_MainContent = _resolveComponent("MainContent")

  return (_openBlock(), _createBlock(_component_MainContent, { back: "" }, {
      title: _withCtx(() => [
        _createVNode(_component_MainTitle, {
          as: "h1",
          secondary: "",
          icon: "i-ri:test-tube-line"
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_ctx.$t('settings.notifications.notifications.label')), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          "text-center": "",
          "mt-10": ""
        }, [
          _createElementVNode("h1", { "text-4xl": "" }, [
            _createElementVNode("span", _hoisted_1, _toDisplayString(_ctx.$t('settings.notifications.under_construction')), 1 /* TEXT */),
            _createTextVNode("\n        🚧\n      ")
          ]),
          _createElementVNode("h3", _hoisted_2, _toDisplayString(_ctx.$t('settings.notifications.notifications.label')), 1 /* TEXT */)
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
