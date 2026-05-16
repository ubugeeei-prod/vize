import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:calendar-schedule-line": "true" })

export default /*@__PURE__*/_defineComponent({
  __name: 'scheduled-posts',
  setup(__props) {

definePageMeta({
  middleware: 'auth',
})
const { t } = useI18n()
useHydratedHead({
  title: () => t('nav.scheduled_posts'),
})

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_TimelineScheduledPosts = _resolveComponent("TimelineScheduledPosts")
  const _component_MainContent = _resolveComponent("MainContent")

  return (_openBlock(), _createBlock(_component_MainContent, null, {
      title: _withCtx(() => [
        _createVNode(_component_NuxtLink, {
          to: "/scheduled-post",
          "timeline-title-style": "",
          flex: "",
          "items-center": "",
          "gap-2": "",
          onClick: _cache[0] || (_cache[0] = (...args) => (_ctx.$scrollToTop && _ctx.$scrollToTop(...args)))
        }, {
          default: _withCtx(() => [
            _hoisted_1,
            _createElementVNode("span", null, _toDisplayString(_unref(t)('nav.scheduled_posts')), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })
      ]),
      default: _withCtx(() => [
        (_ctx.isHydrated)
          ? (_openBlock(), _createBlock(_component_TimelineScheduledPosts, { key: 0 }))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
