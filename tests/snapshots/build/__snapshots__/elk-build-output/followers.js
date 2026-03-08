import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, unref as _unref } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'followers',
  async setup(__props) {

let __temp: any, __restore: any

const { t } = useI18n()
const params = useRoute().params
const handle = computed(() => params.account as string)
definePageMeta({ name: 'account-followers' })
const account = await fetchAccountByHandle(handle.value)
const paginator = account ? useMastoClient().v1.accounts.$select(account.id).followers.list() : null
const isSelf = useSelfAccount(account)
if (account) {
  useHydratedHead({
    title: () => `${t('account.followers')} | ${getDisplayName(account)} (@${account.acct})`,
  })
}

return (_ctx: any,_cache: any) => {
  const _component_AccountPaginator = _resolveComponent("AccountPaginator")

  return (_unref(paginator))
      ? (_openBlock(), _createBlock(_component_AccountPaginator, {
        key: 0,
        paginator: _unref(paginator),
        "relationship-context": _unref(isSelf) ? 'followedBy' : undefined,
        context: "followers",
        account: _unref(account)
      }, null, 8 /* PROPS */, ["paginator", "relationship-context", "account"]))
      : _createCommentVNode("v-if", true)
}
}

})
