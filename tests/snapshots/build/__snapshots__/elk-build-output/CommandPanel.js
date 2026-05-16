import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, unref as _unref, vModelText as _vModelText } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "mx-1": "true", "i-ri:search-line": "true" })
const _hoisted_2 = { class: "text-sm" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "text-secondary" }, "/")
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("div", { class: "w-full border-b-1 border-base" })
const _hoisted_5 = { class: "mt-2 px-2 py-1 text-sm text-secondary" }
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("div", { class: "w-full border-b-1 border-base" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:lightbulb-flash-line": "true" })
import type { CommandScope, QueryResult, QueryResultItem } from '~/composables/command'
import type { SearchResult as SearchResultType } from '~/composables/masto/search'

export default /*@__PURE__*/_defineComponent({
  __name: 'CommandPanel',
  emits: ["close"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const registry = useCommandRegistry()
const router = useRouter()
const inputEl = ref<HTMLInputElement>()
const resultEl = ref<HTMLDivElement>()
const scopes = ref<CommandScope[]>([])
const input = commandPanelInput
onMounted(() => {
  inputEl.value?.focus()
})
const commandMode = computed(() => input.value.startsWith('>'))
const query = computed(() => commandMode.value ? '' : input.value.trim())
const { accounts, hashtags, loading } = useSearch(query)
function toSearchQueryResultItem(search: SearchResultType): QueryResultItem {
  return {
    index: 0,
    type: 'search',
    search,
    onActivate: () => router.push(search.to),
  }
}
const searchResult = computed<QueryResult>(() => {
  if (query.value.length === 0 || loading.value)
    return { length: 0, items: [], grouped: {} as any }

  // TODO extract this scope
  // duplicate in SearchWidget.vue
  const hashtagList = hashtags.value.slice(0, 3).map(toSearchQueryResultItem)
  const accountList = accounts.value.map(toSearchQueryResultItem)

  const grouped: QueryResult['grouped'] = new Map()
  grouped.set('Hashtags', hashtagList)
  grouped.set('Users', accountList)

  let index = 0
  for (const items of grouped.values()) {
    for (const item of items)
      item.index = index++
  }

  return {
    grouped,
    items: [...hashtagList, ...accountList],
    length: hashtagList.length + accountList.length,
  }
})
const result = computed<QueryResult>(() => commandMode.value
  ? registry.query(scopes.value.map(s => s.id).join('.'), input.value.slice(1).trim())
  : searchResult.value,
)
const isMac = useIsMac()
const modifierKeyName = computed(() => isMac.value ? '⌘' : 'Ctrl')
const active = ref(0)
watch(result, (n, o) => {
  if (n.length !== o.length || !n.items.every((i, idx) => i === o.items[idx]))
    active.value = 0
})
function findItemEl(index: number) {
  return resultEl.value?.querySelector(`[data-index="${index}"]`) as HTMLDivElement | null
}
function onCommandActivate(item: QueryResultItem) {
  if (item.onActivate) {
    item.onActivate()
    emit('close')
  }
  else if (item.onComplete) {
    scopes.value.push(item.onComplete())
    input.value = '> '
  }
}
function onCommandComplete(item: QueryResultItem) {
  if (item.onComplete) {
    scopes.value.push(item.onComplete())
    input.value = '> '
  }
  else if (item.onActivate) {
    item.onActivate()
    emit('close')
  }
}
function intoView(index: number) {
  const el = findItemEl(index)
  if (el)
    el.scrollIntoView({ block: 'nearest' })
}
function setActive(index: number) {
  const len = result.value.length
  active.value = (index + len) % len
  intoView(active.value)
}
function onKeyDown(e: KeyboardEvent) {
  switch (e.key) {
    case 'p':
    case 'ArrowUp': {
      if (e.key === 'p' && !e.ctrlKey)
        break
      e.preventDefault()
      setActive(active.value - 1)
      break
    }
    case 'n':
    case 'ArrowDown': {
      if (e.key === 'n' && !e.ctrlKey)
        break
      e.preventDefault()
      setActive(active.value + 1)
      break
    }
    case 'Home': {
      e.preventDefault()
      active.value = 0
      intoView(active.value)
      break
    }
    case 'End': {
      e.preventDefault()
      setActive(result.value.length - 1)
      break
    }
    case 'Enter': {
      e.preventDefault()
      const cmd = result.value.items[active.value]
      if (cmd)
        onCommandActivate(cmd)
      break
    }
    case 'Tab': {
      e.preventDefault()
      const cmd = result.value.items[active.value]
      if (cmd)
        onCommandComplete(cmd)
      break
    }
    case 'Backspace': {
      if (input.value === '>' && scopes.value.length) {
        e.preventDefault()
        scopes.value.pop()
      }
      break
    }
  }
}

return (_ctx: any,_cache: any) => {
  const _component_CommandKey = _resolveComponent("CommandKey")
  const _component_SearchResultSkeleton = _resolveComponent("SearchResultSkeleton")
  const _component_SearchResult = _resolveComponent("SearchResult")
  const _component_CommandItem = _resolveComponent("CommandItem")

  return (_openBlock(), _createElementBlock("div", { class: "flex flex-col w-50vw max-w-180 h-50vh max-h-120" }, [ _createElementVNode("label", { class: "flex mx-3 my-1 items-center" }, [ _hoisted_1, (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(scopes.value, (scope) => {
          return (_openBlock(), _createElementBlock("div", {
            key: scope.id,
            class: "flex items-center mx-1 gap-2"
          }, [
            _createElementVNode("div", _hoisted_2, _toDisplayString(scope.display), 1 /* TEXT */),
            _hoisted_3
          ]))
        }), 128 /* KEYED_FRAGMENT */)), _withDirectives(_createElementVNode("input", {
          ref_key: "inputEl", ref: inputEl,
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((input).value = $event)),
          class: "focus:outline-none flex-1 p-2 rounded bg-base",
          placeholder: "Search",
          onKeydown: onKeyDown
        }, null, 32 /* NEED_HYDRATION */), [ [_vModelText, _unref(input)] ]), _createVNode(_component_CommandKey, { name: "Escape" }) ]), _hoisted_4, _createTextVNode("\n\n    " + "\n    "), _createElementVNode("div", {
        ref_key: "resultEl", ref: resultEl,
        class: "flex-1 mx-1 overflow-y-auto"
      }, [ (_unref(loading)) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createVNode(_component_SearchResultSkeleton), _createVNode(_component_SearchResultSkeleton), _createVNode(_component_SearchResultSkeleton) ], 64 /* STABLE_FRAGMENT */)) : (result.value.length) ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(result.value.grouped, ([scope, group]) => {
                return (_openBlock(), _createElementBlock(_Fragment, { key: scope }, [
                  _createElementVNode("div", _hoisted_5, _toDisplayString(scope), 1 /* TEXT */),
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(group, (item) => {
                    return (_openBlock(), _createElementBlock(_Fragment, { key: item.index }, [
                      (item.type === 'search')
                        ? (_openBlock(), _createBlock(_component_SearchResult, {
                          key: 0,
                          active: active.value === item.index,
                          result: item.search
                        }, null, 8 /* PROPS */, ["active", "result"]))
                        : (_openBlock(), _createBlock(_component_CommandItem, {
                          key: 1,
                          index: item.index,
                          cmd: item.cmd,
                          active: active.value === item.index,
                          onActivate: _cache[1] || (_cache[1] = ($event: any) => (onCommandActivate(item)))
                        }, null, 8 /* PROPS */, ["index", "cmd", "active"]))
                    ], 64 /* STABLE_FRAGMENT */))
                  }), 128 /* KEYED_FRAGMENT */))
                ], 64 /* STABLE_FRAGMENT */))
              }), 128 /* KEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock("div", {
            key: 2,
            p5: "",
            "text-center": "",
            "text-secondary": "",
            italic: ""
          }, _toDisplayString(_unref(input).trim().length ? _ctx.$t('common.not_found') : _ctx.$t('search.search_desc')), 1 /* TEXT */)) ], 512 /* NEED_PATCH */), _hoisted_6, _createTextVNode("\n\n    " + "\n    "), _createElementVNode("div", { class: "flex items-center px-3 py-1 text-xs" }, [ _hoisted_7, _createTextVNode(" Tip: Use\n      "), _createVNode(_component_CommandKey, { name: `${modifierKeyName.value}+K` }, null, 8 /* PROPS */, ["name"]), _createTextVNode(" to search,\n      "), _createVNode(_component_CommandKey, { name: `${modifierKeyName.value}+/` }, null, 8 /* PROPS */, ["name"]), _createTextVNode(" to activate command mode.\n    ") ]) ]))
}
}

})
