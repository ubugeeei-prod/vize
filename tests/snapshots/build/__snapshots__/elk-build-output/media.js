import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, resolveComponent as _resolveComponent, unref as _unref } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'media',
  async setup(__props) {

let __temp: any, __restore: any

definePageMeta({ name: 'account-media' })
const { t } = useI18n()
const params = useRoute().params
const handle = computed(() => params.account as string)
const account = await fetchAccountByHandle(handle.value)
const paginator = useMastoClient().v1.accounts.$select(account.id).statuses.list({ onlyMedia: true, excludeReplies: false })
if (account) {
  useHydratedHead({
    title: () => `${t('tab.media')} | ${getDisplayName(account)} (@${account.acct})`,
  })
}

return (_ctx: any,_cache: any) => {
  const _component_AccountTabs = _resolveComponent("AccountTabs")
  const _component_TimelinePaginator = _resolveComponent("TimelinePaginator")

  return (_openBlock(), _createElementBlock("div", null, [ _createVNode(_component_AccountTabs), _createVNode(_component_TimelinePaginator, {
        paginator: _unref(paginator),
        preprocess: _ctx.filterAndReorderTimeline,
        context: "account",
        account: _unref(account)
      }, null, 8 /* PROPS */, ["paginator", "preprocess", "account"]) ]))
}
}

})
