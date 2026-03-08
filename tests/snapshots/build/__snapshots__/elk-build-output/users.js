import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, resolveComponent as _resolveComponent, withCtx as _withCtx, unref as _unref } from "vue"

import { STORAGE_KEY_LAST_ACCESSED_EXPLORE_ROUTE } from '~/constants'

export default /*@__PURE__*/_defineComponent({
  __name: 'users',
  setup(__props) {

const { t } = useI18n()
const route = useRoute()
// limit: 20 is the default configuration of the official client
const paginator = useMastoClient().v2.suggestions.list({ limit: 20 })
useHydratedHead({
  title: () => `${t('tab.for_you')} | ${t('nav.explore')}`,
})
const lastAccessedExploreRoute = useLocalStorage(STORAGE_KEY_LAST_ACCESSED_EXPLORE_ROUTE, '')
lastAccessedExploreRoute.value = route.path.replace(/(.*\/explore\/?)/, '')
onActivated(() => {
  lastAccessedExploreRoute.value = route.path.replace(/(.*\/explore\/?)/, '')
})

return (_ctx: any,_cache: any) => {
  const _component_AccountBigCard = _resolveComponent("AccountBigCard")
  const _component_AccountBigCardSkeleton = _resolveComponent("AccountBigCardSkeleton")
  const _component_CommonPaginator = _resolveComponent("CommonPaginator")

  return (_openBlock(), _createBlock(_component_CommonPaginator, {
      paginator: _unref(paginator),
      "key-prop": "account"
    }, {
      default: _withCtx(({ item }) => [
        _createVNode(_component_AccountBigCard, {
          account: item.account,
          as: "router-link",
          to: _ctx.getAccountRoute(item.account),
          border: "b base"
        }, null, 8 /* PROPS */, ["account", "to"])
      ]),
      loading: _withCtx(() => [
        _createVNode(_component_AccountBigCardSkeleton, { border: "b base" }),
        _createVNode(_component_AccountBigCardSkeleton, {
          border: "b base",
          op50: ""
        }),
        _createVNode(_component_AccountBigCardSkeleton, {
          border: "b base",
          op25: ""
        })
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["paginator"]))
}
}

})
