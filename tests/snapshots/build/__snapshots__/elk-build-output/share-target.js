import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderSlot as _renderSlot, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:share-line": "true" })

export default /*@__PURE__*/_defineComponent({
  __name: 'share-target',
  setup(__props) {

definePageMeta({
  middleware: () => {
    if (!useAppConfig().pwaEnabled)
      return navigateTo('/')
  },
})
useWebShareTarget()
const pwaIsInstalled = import.meta.client && !!useNuxtApp().$pwa?.isInstalled

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_MainContent = _resolveComponent("MainContent")

  return (_openBlock(), _createBlock(_component_MainContent, null, {
      title: _withCtx(() => [
        _createVNode(_component_NuxtLink, {
          to: "/share-target",
          flex: "",
          "items-center": "",
          "gap-2": ""
        }, {
          default: _withCtx(() => [
            _hoisted_1,
            _createElementVNode("span", null, _toDisplayString(_ctx.$t('share_target.title')), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })
      ]),
      default: _withCtx(() => [
        _renderSlot(_ctx.$slots, "default", {}, () => [
          _createElementVNode("div", {
            flex: "~ col",
            px5: "",
            py2: "",
            "gap-y-4": ""
          }, [
            (!_unref(pwaIsInstalled) || !_ctx.currentUser)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                role: "alert",
                "gap-1": "",
                "p-2": "",
                "text-red-600": "",
                "dark:text-red-400": "",
                border: "~ base rounded red-600 dark:red-400"
              }, _toDisplayString(_ctx.$t('share_target.hint')), 1 /* TEXT */))
              : _createCommentVNode("v-if", true),
            _createElementVNode("div", null, _toDisplayString(_ctx.$t('share_target.description')), 1 /* TEXT */)
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
