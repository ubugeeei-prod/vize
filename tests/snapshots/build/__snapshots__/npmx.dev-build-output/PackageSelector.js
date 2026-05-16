import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:leaf w-3.5 h-3.5", "aria-hidden": "true" })
const _hoisted_2 = { for: "package-search", class: "sr-only" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "absolute inset-is-3 text-fg-subtle font-mono text-md pointer-events-none transition-colors duration-200 motion-reduce:transition-none [.group:hover:not(:focus-within)_&]:text-fg/80 group-focus-within:text-accent z-1" }, "\n          /\n        ")
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:leaf w-4 h-4", "aria-hidden": "true" })
const _hoisted_5 = { class: "text-xs text-fg-muted truncate mt-0.5" }
const _hoisted_6 = { class: "font-mono text-sm text-fg block" }
import { NO_DEPENDENCY_ID } from '~/composables/usePackageComparison'
const PAGE_JUMP = 5

export default /*@__PURE__*/_defineComponent({
  __name: 'PackageSelector',
  props: {
    max: { type: Number, required: false },
    "modelValue": { required: true }
  },
  emits: ["update:modelValue"],
  setup(__props: any) {

const props = __props
const packages = _useModel(__props, "modelValue")
const maxPackages = computed(() => props.max ?? 4)
// Input state
const inputValue = shallowRef('')
const isInputFocused = shallowRef(false)
// Keyboard navigation state
const highlightedIndex = shallowRef(-1)
const listRef = useTemplateRef('listRef')
// Use the shared search composable (supports both npm and Algolia providers)
const { searchProvider } = useSearchProvider()
const { data: searchData, status } = useSearch(inputValue, searchProvider, { size: 15 })
const isSearching = computed(() => status.value === 'pending')
// Trigger strings for "What Would James Do?" typeahead Easter egg
// Intentionally not localized
const EASTER_EGG_TRIGGERS = new Set([
  'no dep',
  'none',
  'vanilla',
  'diy',
  'zero',
  'nothing',
  '0',
  "don't",
  'native',
  'use the platform',
])
// Check if "no dependency" option should show in typeahead
const showNoDependencyOption = computed(() => {
  if (packages.value.includes(NO_DEPENDENCY_ID)) return false
  const input = inputValue.value.toLowerCase().trim()
  if (!input) return false
  return EASTER_EGG_TRIGGERS.has(input)
})
// Filter out already selected packages
const filteredResults = computed(() => {
  if (!searchData.value?.objects) return []
  return searchData.value.objects
    .map(o => ({
      name: o.package.name,
      description: o.package.description,
    }))
    .filter(r => !packages.value.includes(r.name))
})
// Unified list of navigable items for keyboard navigation
const navigableItems = computed(() => {
  const items: { type: 'no-dependency' | 'package'; name: string }[] = []
  if (showNoDependencyOption.value) {
    items.push({ type: 'no-dependency', name: NO_DEPENDENCY_ID })
  }
  for (const r of filteredResults.value) {
    items.push({ type: 'package', name: r.name })
  }
  return items
})
const resultIndexOffset = computed(() => (showNoDependencyOption.value ? 1 : 0))
const numberFormatter = useNumberFormatter()
function addPackage(name: string) {
  if (packages.value.length >= maxPackages.value) return
  if (packages.value.includes(name)) return
  // Keep NO_DEPENDENCY_ID always last
  if (name === NO_DEPENDENCY_ID) {
    packages.value = [...packages.value, name]
  } else if (packages.value.includes(NO_DEPENDENCY_ID)) {
    // Insert before the no-dep entry
    const withoutNoDep = packages.value.filter(p => p !== NO_DEPENDENCY_ID)
    packages.value = [...withoutNoDep, name, NO_DEPENDENCY_ID]
  } else {
    packages.value = [...packages.value, name]
  }
  inputValue.value = ''
  highlightedIndex.value = -1
}
function removePackage(name: string) {
  packages.value = packages.value.filter(p => p !== name)
}
function handleKeydown(e: KeyboardEvent) {
  const items = navigableItems.value
  const count = items.length
  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault()
      if (count === 0) return
      if (highlightedIndex.value < count - 1) {
        highlightedIndex.value++
      } else {
        highlightedIndex.value = 0
      }
      break
    case 'ArrowUp':
      e.preventDefault()
      if (count === 0) return
      if (highlightedIndex.value > 0) {
        highlightedIndex.value--
      } else {
        highlightedIndex.value = count - 1
      }
      break
    case 'PageDown':
      e.preventDefault()
      if (count === 0) return
      if (highlightedIndex.value === -1) {
        highlightedIndex.value = Math.min(PAGE_JUMP - 1, count - 1)
      } else {
        highlightedIndex.value = Math.min(highlightedIndex.value + PAGE_JUMP, count - 1)
      }
      break
    case 'PageUp':
      e.preventDefault()
      if (count === 0) return
      highlightedIndex.value = Math.max(highlightedIndex.value - PAGE_JUMP, 0)
      break
    case 'Enter': {
      const inputValueTrim = inputValue.value.trim()
      if (!inputValueTrim) return
      e.preventDefault()
      // If an item is highlighted, select it
      if (highlightedIndex.value >= 0 && highlightedIndex.value < count) {
        addPackage(items[highlightedIndex.value]!.name)
        return
      }
      // Fallback: exact match or easter egg (preserves existing behavior)
      if (showNoDependencyOption.value) {
        addPackage(NO_DEPENDENCY_ID)
      } else {
        const hasMatch = filteredResults.value.find(r => r.name === inputValueTrim)
        if (hasMatch) {
          addPackage(inputValueTrim)
        }
      }
      break
    }
    case 'Escape':
      inputValue.value = ''
      highlightedIndex.value = -1
      break
  }
}
// Reset highlight when user types
watch(inputValue, () => {
  highlightedIndex.value = -1
})
// Scroll highlighted item into view
watch(highlightedIndex, index => {
  if (index >= 0 && listRef.value) {
    const items = listRef.value.querySelectorAll('[data-navigable]')
    const item = items[index] as HTMLElement | undefined
    item?.scrollIntoView({ block: 'nearest' })
  }
})
const containerRef = useTemplateRef('containerRef')
onClickOutside(containerRef, () => {
  isInputFocused.value = false
  highlightedIndex.value = -1
})

return (_ctx: any,_cache: any) => {
  const _component_LinkBase = _resolveComponent("LinkBase")
  const _component_ButtonBase = _resolveComponent("ButtonBase")
  const _component_TagStatic = _resolveComponent("TagStatic")
  const _component_InputBase = _resolveComponent("InputBase")

  return (_openBlock(), _createElementBlock("div", { class: "space-y-3" }, [ (packages.value.length > 0) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "flex flex-wrap gap-2"
        }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(packages.value, (pkg) => {
            return (_openBlock(), _createBlock(_component_TagStatic, { key: pkg }, {
              default: _withCtx(() => [
                (pkg === _unref(NO_DEPENDENCY_ID))
                  ? (_openBlock(), _createElementBlock("span", {
                    key: 0,
                    class: "text-sm text-accent italic flex items-center gap-1.5"
                  }, [
                    _hoisted_1,
                    _createTextVNode("\n            "),
                    _toDisplayString(_ctx.$t('compare.no_dependency.label'))
                  ]))
                  : (_openBlock(), _createBlock(_component_LinkBase, {
                    key: 1,
                    to: _ctx.packageRoute(pkg),
                    class: "text-sm"
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(pkg), 1 /* TEXT */)
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 8 /* PROPS */, ["to"])),
                _createVNode(_component_ButtonBase, {
                  size: "small",
                  "aria-label": 
              _ctx.$t('compare.selector.remove_package', {
                package: pkg === _unref(NO_DEPENDENCY_ID) ? _ctx.$t('compare.no_dependency.label') : pkg,
              })
            ,
                  onClick: _cache[0] || (_cache[0] = ($event: any) => (removePackage(pkg))),
                  classicon: "i-lucide:x"
                }, null, 8 /* PROPS */, ["aria-label"])
              ]),
              _: 2 /* DYNAMIC */
            }, 1024 /* DYNAMIC_SLOTS */))
          }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    " + "\n    "), (packages.value.length < maxPackages.value) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          ref: "containerRef",
          class: "relative"
        }, [ _createElementVNode("div", { class: "relative group flex items-center" }, [ _createElementVNode("label", _hoisted_2, _toDisplayString(_ctx.$t('compare.selector.search_label')), 1 /* TEXT */), _hoisted_3, _createVNode(_component_InputBase, {
              id: "package-search",
              type: "text",
              placeholder: 
              packages.value.length === 0
                ? _ctx.$t('compare.selector.search_first')
                : _ctx.$t('compare.selector.search_add')
            ,
              "no-correct": "",
              size: "medium",
              class: "w-full min-w-25 ps-7",
              "aria-autocomplete": "list",
              ref: "inputRef",
              onFocus: _cache[1] || (_cache[1] = ($event: any) => (isInputFocused.value = true)),
              onKeydown: handleKeydown,
              modelValue: inputValue.value,
              "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((inputValue).value = $event))
            }, null, 8 /* PROPS */, ["placeholder", "modelValue"]) ]), _createTextVNode("\n\n      "), _createTextVNode("\n      "), _createVNode(_Transition, {
            "enter-active-class": "transition-opacity duration-150",
            "enter-from-class": "opacity-0",
            "leave-active-class": "transition-opacity duration-100",
            "leave-from-class": "opacity-100",
            "leave-to-class": "opacity-0"
          }, {
            default: _withCtx(() => [
              (isInputFocused.value && (navigableItems.value.length > 0 || isSearching.value))
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  ref: "listRef",
                  class: "absolute top-full inset-x-0 mt-1 bg-bg-elevated border border-border rounded-lg shadow-lg z-50 max-h-64 overflow-y-auto"
                }, [
                  (showNoDependencyOption.value)
                    ? (_openBlock(), _createBlock(_component_ButtonBase, {
                      key: 0,
                      "data-navigable": "",
                      class: _normalizeClass(["block w-full text-start", highlightedIndex.value === 0 ? '!bg-accent/15' : '']),
                      "aria-label": _ctx.$t('compare.no_dependency.add_column'),
                      onMouseenter: _cache[3] || (_cache[3] = ($event: any) => (highlightedIndex.value = 0)),
                      onClick: _cache[4] || (_cache[4] = ($event: any) => (addPackage(_unref(NO_DEPENDENCY_ID))))
                    }, {
                      default: _withCtx(() => [
                        _createElementVNode("span", { class: "text-sm text-accent italic flex items-center gap-2" }, [
                          _hoisted_4,
                          _createTextVNode("\n              " + _toDisplayString(_ctx.$t('compare.no_dependency.typeahead_title')), 1 /* TEXT */)
                        ]),
                        _createElementVNode("span", _hoisted_5, _toDisplayString(_ctx.$t('compare.no_dependency.typeahead_description')), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 10 /* CLASS, PROPS */, ["aria-label"]))
                    : _createCommentVNode("v-if", true),
                  (isSearching.value && navigableItems.value.length === 0)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "px-4 py-3 text-sm text-fg-muted"
                    }, _toDisplayString(_ctx.$t('compare.selector.searching')), 1 /* TEXT */))
                    : _createCommentVNode("v-if", true),
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(filteredResults.value, (result, index) => {
                    return (_openBlock(), _createBlock(_component_ButtonBase, {
                      key: result.name,
                      "data-navigable": "",
                      class: _normalizeClass(["block w-full text-start", highlightedIndex.value === index + resultIndexOffset.value ? '!bg-accent/15' : '']),
                      onMouseenter: _cache[5] || (_cache[5] = ($event: any) => (highlightedIndex.value = index + resultIndexOffset.value)),
                      onClick: _cache[6] || (_cache[6] = ($event: any) => (addPackage(result.name)))
                    }, {
                      default: _withCtx(() => [
                        _createElementVNode("span", _hoisted_6, _toDisplayString(result.name), 1 /* TEXT */),
                        (result.description)
                          ? (_openBlock(), _createElementBlock("span", {
                            key: 0,
                            class: "text-xs text-fg-muted truncate mt-0.5 w-full block"
                          }, _toDisplayString(_ctx.stripHtmlTags(_ctx.decodeHtmlEntities(result.description))), 1 /* TEXT */))
                          : _createCommentVNode("v-if", true)
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 1026 /* CLASS, DYNAMIC_SLOTS */))
                  }), 128 /* KEYED_FRAGMENT */))
                ]))
                : _createCommentVNode("v-if", true)
            ]),
            _: 1 /* STABLE */
          }) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    " + "\n    "), _createElementVNode("p", { class: "text-xs text-fg-subtle" }, [ _createTextVNode(_toDisplayString(_ctx.$t('compare.selector.packages_selected', {
            count: _unref(numberFormatter).format(packages.value.length),
            max: _unref(numberFormatter).format(maxPackages.value),
          })) + "\n      ", 1 /* TEXT */), (packages.value.length < 2) ? (_openBlock(), _createElementBlock("span", { key: 0 }, _toDisplayString(_ctx.$t('compare.selector.add_hint')), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]) ]))
}
}

})
