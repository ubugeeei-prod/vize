import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue"

import type { mastodon } from 'masto'
import { reactedByStatusId } from '~/composables/dialog'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusReactedBy',
  setup(__props) {

const type = ref<'favourited-by' | 'boosted-by' | 'quoted-by'>('favourited-by')
const { client } = useMasto()
function load() {
  if (type.value !== 'quoted-by') {
    const accounts = client.value.v1.statuses.$select(reactedByStatusId.value!)[type.value === 'favourited-by' ? 'favouritedBy' : 'rebloggedBy'].list()
    return accounts
  }
  else {
    const quotes = client.value.v1.statuses.$select(reactedByStatusId.value!).quotes.list()
    return quotes
  }
}
function preprocess(items: mastodon.v1.Status[] | mastodon.v1.Account[]): mastodon.v1.Account[] {
  if (type.value !== 'quoted-by')
    return items as mastodon.v1.Account[]
  return (items as mastodon.v1.Status[]).map(quote => quote.account)
}
const paginator = computed(() => load())
function showFavouritedBy() {
  type.value = 'favourited-by'
}
function showRebloggedBy() {
  type.value = 'boosted-by'
}
function showQuotedBy() {
  type.value = 'quoted-by'
}
const { t } = useI18n()
const tabs = [
  {
    name: 'favourited-by',
    display: t('status.favourited_by'),
    onClick: showFavouritedBy,
  },
  {
    name: 'boosted-by',
    display: t('status.boosted_by'),
    onClick: showRebloggedBy,
  },
  {
    name: 'quoted-by',
    display: t('status.quoted_by'),
    onClick: showQuotedBy,
  },
]

return (_ctx: any,_cache: any) => {
  const _component_AccountPaginator = _resolveComponent("AccountPaginator")

  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createElementVNode("div", {
        flex: "",
        "w-full": "",
        "items-center": "",
        "lg:text-lg": "",
        "of-x-auto": "",
        "scrollbar-hide": ""
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(tabs, (option) => {
          return (_openBlock(), _createElementBlock("div", {
            key: option.name,
            relative: "",
            flex: "",
            "flex-auto": "",
            "cursor-pointer": "",
            "sm:px6": "",
            px2: "",
            rounded: "",
            "transition-all": "",
            tabindex: "0",
            "hover:bg-active": "",
            "transition-100": "",
            onClick: _cache[0] || (_cache[0] = ($event: any) => (option.onClick))
          }, [
            _createElementVNode("span", {
              "ws-nowrap": "",
              mxa: "",
              "sm:px2": "",
              "sm:py3": "",
              "xl:pb4": "",
              "xl:pt5": "",
              py2: "",
              "text-center": "",
              "border-b-3": "",
              class: _normalizeClass(option.name === type.value ? 'border-primary op100 text-base' : 'border-transparent text-secondary-light hover:text-secondary op50')
            }, _toDisplayString(option.display), 3 /* TEXT, CLASS */)
          ]))
        }), 128 /* KEYED_FRAGMENT */)) ]), _createVNode(_component_AccountPaginator, {
        key: `paginator-${type.value}`,
        paginator: paginator.value,
        preprocess: preprocess
      }, null, 8 /* PROPS */, ["paginator", "preprocess"]) ], 64 /* STABLE_FRAGMENT */))
}
}

})
