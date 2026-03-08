import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeProps as _normalizeProps, guardReactiveProps as _guardReactiveProps, withCtx as _withCtx, unref as _unref } from "vue"

import { STORAGE_KEY_HIDE_EXPLORE_TAGS_TIPS, STORAGE_KEY_LAST_ACCESSED_EXPLORE_ROUTE } from '~/constants'

export default /*@__PURE__*/_defineComponent({
  __name: 'tags',
  setup(__props) {

const { t } = useI18n()
const route = useRoute()
const { client } = useMasto()
const paginator = client.value.v1.trends.tags.list({
  limit: 20,
})
const hideTagsTips = useLocalStorage(STORAGE_KEY_HIDE_EXPLORE_TAGS_TIPS, false)
useHydratedHead({
  title: () => `${t('tab.hashtags')} | ${t('nav.explore')}`,
})
const lastAccessedExploreRoute = useLocalStorage(STORAGE_KEY_LAST_ACCESSED_EXPLORE_ROUTE, '')
lastAccessedExploreRoute.value = route.path.replace(/(.*\/explore\/?)/, '')
onActivated(() => {
  lastAccessedExploreRoute.value = route.path.replace(/(.*\/explore\/?)/, '')
})

return (_ctx: any,_cache: any) => {
  const _component_CommonAlert = _resolveComponent("CommonAlert")
  const _component_TagCardPaginator = _resolveComponent("TagCardPaginator")

  return (_openBlock(), _createElementBlock(_Fragment, null, [ (!_unref(hideTagsTips)) ? (_openBlock(), _createBlock(_component_CommonAlert, {
          key: 0,
          onClose: _cache[0] || (_cache[0] = ($event: any) => (hideTagsTips.value = true))
        }, {
          default: _withCtx(() => [
            _createElementVNode("p", null, _toDisplayString(_ctx.$t('tooltip.explore_tags_intro')), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })) : _createCommentVNode("v-if", true), _createVNode(_component_TagCardPaginator, _normalizeProps(_guardReactiveProps({ paginator: _unref(paginator) })), null, 16 /* FULL_PROPS */) ], 64 /* STABLE_FRAGMENT */))
}
}

})
