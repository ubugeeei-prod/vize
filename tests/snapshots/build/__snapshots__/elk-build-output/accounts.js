import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vModelText as _vModelText, withModifiers as _withModifiers, withKeys as _withKeys } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:search-2-line": "true", "pointer-events-none": "true", "text-secondary": "true", mt: "1px", class: "rtl-flip" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", class: "i-ri:close-line" })
import type { mastodon } from 'masto'
import AccountSearchResult from '~/components/list/AccountSearchResult.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'accounts',
  async setup(__props, { expose: __expose }) {

let __temp: any, __restore: any

definePageMeta({
  name: 'list-accounts',
})
const inputRef = ref<HTMLInputElement>()
const params = useRoute().params
const listId = computed(() => params.list as string)
const mastoListAccounts = useMastoClient().v1.lists.$select(listId.value).accounts
const paginator = mastoListAccounts.list()
// the limit parameter is set to 1000 while masto.js issue is still open: https://github.com/neet/masto.js/issues/1282
const accountsInList = ref((await useMastoClient().v1.lists.$select(listId.value).accounts.list({ limit: 1000 })))
const paginatorRef = ref()
// search stuff
const query = ref('')
const el = ref<HTMLElement>()
const { accounts, loading } = useSearch(query, {
  following: true,
})
const { focused } = useFocusWithin(el)
const index = ref(0)
function isInCurrentList(userId: string) {
  return accountsInList.value.map(account => account.id).includes(userId)
}
const results = computed(() => {
  if (query.value.length === 0)
    return []
  return [...accounts.value]
})
// Reset index when results change
watch([results, focused], () => index.value = -1)
function addAccount(account: mastodon.v1.Account) {
  try {
    mastoListAccounts.create({ accountIds: [account.id] })
    accountsInList.value.push(account)
    paginatorRef.value?.createEntry(account)
  }
  catch (err) {
    console.error(err)
  }
}
function removeAccount(account: mastodon.v1.Account) {
  try {
    mastoListAccounts.remove({ accountIds: [account.id] })
    const accountIdsInList = accountsInList.value.map(account => account.id)
    const index = accountIdsInList.indexOf(account.id)
    if (index > -1) {
      accountsInList.value.splice(index, 1)
      paginatorRef.value?.removeEntry(account.id)
    }
  }
  catch (err) {
    console.error(err)
  }
}
__expose({
  inputRef,
})

return (_ctx: any,_cache: any) => {
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")
  const _component_SearchResultSkeleton = _resolveComponent("SearchResultSkeleton")
  const _component_ListAccount = _resolveComponent("ListAccount")
  const _component_CommonPaginator = _resolveComponent("CommonPaginator")

  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createElementVNode("div", {
        ref_key: "el", ref: el,
        relative: "",
        group: ""
      }, [ _createElementVNode("form", {
          border: "t base",
          "p-4": "",
          "w-full": "",
          flex: "~ wrap",
          relative: "",
          "gap-3": ""
        }, [ _createElementVNode("div", {
            "bg-base": "",
            border: "~ base",
            "flex-1": "",
            h10: "",
            "ps-1": "",
            "pe-4": "",
            "rounded-2": "",
            "w-full": "",
            flex: "~ row",
            "items-center": "",
            relative: "",
            "focus-within:box-shadow-outline": "",
            "gap-3": "",
            "ps-4": ""
          }, [ _hoisted_1, _withDirectives(_createElementVNode("input", {
              ref_key: "inputRef", ref: inputRef,
              "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((query).value = $event)),
              "bg-transparent": "",
              outline: "focus:none",
              "ps-3": "",
              "rounded-3": "",
              pb: "1px",
              "h-full": "",
              "w-full": "",
              "placeholder-text-secondary": "",
              placeholder: _ctx.$t('list.search_following_placeholder'),
              onKeydown: [_withKeys(_withModifiers(($event: any) => (inputRef.value?.blur()), ["prevent"]), ["esc"]), _withKeys(_withModifiers(() => {}, ["prevent"]), ["enter"])]
            }, null, 40 /* PROPS, NEED_HYDRATION */, ["placeholder", "onKeydown"]), [ [_vModelText, query.value] ]), (query.value.length) ? (_openBlock(), _createElementBlock("button", {
                key: 0,
                "btn-action-icon": "",
                "text-secondary": "",
                onClick: _cache[1] || (_cache[1] = ($event: any) => { query.value = ''; inputRef.value?.focus() })
              }, [ _hoisted_2 ])) : _createCommentVNode("v-if", true) ]) ]), _createTextVNode("\n\n    " + "\n    "), _createElementVNode("div", {
          "left-0": "",
          "top-18": "",
          absolute: "",
          "w-full": "",
          "z-10": "",
          "group-focus-within": "pointer-events-auto visible",
          invisible: "",
          "pointer-events-none": ""
        }, [ _createElementVNode("div", {
            "w-full": "",
            "bg-base": "",
            border: "~ dark",
            "rounded-3": "",
            "max-h-100": "",
            "overflow-auto": "",
            class: _normalizeClass(results.value.length === 0 ? 'py2' : null)
          }, [ (query.value.trim().length === 0) ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                block: "",
                "text-center": "",
                "text-sm": "",
                "text-secondary": ""
              }, _toDisplayString(_ctx.$t('list.search_following_desc')), 1 /* TEXT */)) : (!_unref(loading)) ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ (results.value.length > 0) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(results.value, (result, i) => {
                        return (_openBlock(), _createElementBlock("div", {
                          key: result.id,
                          flex: "",
                          border: "b base",
                          py2: "",
                          px4: "",
                          "hover:bg-active": "",
                          "justify-between": "",
                          "transition-100": "",
                          "items-center": ""
                        }, [
                          _createVNode(AccountSearchResult, {
                            active: index.value === parseInt(i.toString()),
                            result: result,
                            tabindex: _unref(focused) ? 0 : -1
                          }, null, 8 /* PROPS */, ["active", "result", "tabindex"]),
                          _createVNode(_component_CommonTooltip, { content: isInCurrentList(result.id) ? _ctx.$t('list.remove_account') : _ctx.$t('list.add_account') }, {
                            default: _withCtx(() => [
                              _createElementVNode("button", {
                                "text-sm": "",
                                p2: "",
                                "border-1": "",
                                "transition-colors": "",
                                "border-dark": "",
                                "btn-action-icon": "",
                                "bg-base": "",
                                hover: isInCurrentList(result.id) ? 'text-red' : 'text-green',
                                onClick: _cache[2] || (_cache[2] =  () => isInCurrentList(result.id) ? removeAccount(result.data) : addAccount(result.data) )
                              }, [
                                _createElementVNode("span", {
                                  class: _normalizeClass(isInCurrentList(result.id) ? 'i-ri:user-unfollow-line' : 'i-ri:user-add-line')
                                }, null, 2 /* CLASS */)
                              ], 8 /* PROPS */, ["hover"])
                            ]),
                            _: 2 /* DYNAMIC */
                          }, 8 /* PROPS */, ["content"])
                        ]))
                      }), 128 /* KEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock("span", {
                      key: 1,
                      block: "",
                      "text-center": "",
                      "text-sm": "",
                      "text-secondary": ""
                    }, _toDisplayString(_ctx.$t('search.search_empty')), 1 /* TEXT */)) ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock("div", { key: 2 }, [ _createVNode(_component_SearchResultSkeleton), _createVNode(_component_SearchResultSkeleton), _createVNode(_component_SearchResultSkeleton) ])) ], 2 /* CLASS */) ]) ], 512 /* NEED_PATCH */), _createVNode(_component_CommonPaginator, {
        ref_key: "paginatorRef", ref: paginatorRef,
        paginator: _unref(paginator)
      }, {
        default: _withCtx(({ item }) => [
          _createVNode(_component_ListAccount, {
            account: item,
            list: listId.value,
            "hover-card": "",
            border: "b base",
            py2: "",
            px4: ""
          }, null, 8 /* PROPS */, ["account", "list"])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["paginator"]) ], 64 /* STABLE_FRAGMENT */))
}
}

})
