import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'favourites',
  setup(__props) {

definePageMeta({
  middleware: 'auth',
})
const { t } = useI18n()
const useStarFavoriteIcon = usePreferences('useStarFavoriteIcon')
useHydratedHead({
  title: () => t('nav.favourites'),
})

return (_ctx: any,_cache: any) => {
  const _component_MainTitle = _resolveComponent("MainTitle")
  const _component_TimelineFavourites = _resolveComponent("TimelineFavourites")
  const _component_MainContent = _resolveComponent("MainContent")

  return (_openBlock(), _createBlock(_component_MainContent, null, {
      title: _withCtx(() => [
        _createVNode(_component_MainTitle, {
          as: "router-link",
          to: "/favourites",
          icon: _unref(useStarFavoriteIcon) ? 'i-ri:star-line' : 'i-ri:heart-3-line'
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(t)('nav.favourites')), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["icon"])
      ]),
      default: _withCtx(() => [
        (_ctx.isHydrated)
          ? (_openBlock(), _createBlock(_component_TimelineFavourites, { key: 0 }))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
