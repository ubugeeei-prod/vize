import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:funnel w-4 h-4", "aria-hidden": "true" })
const _hoisted_2 = { for: "filter-search", class: "text-sm font-mono text-fg-muted" }
const _hoisted_3 = { class: "block text-sm font-mono text-fg-muted mb-1" }
const _hoisted_4 = { class: "block text-sm font-mono text-fg-muted mb-1" }
const _hoisted_5 = { class: "text-xs px-1.5 py-0.5 rounded bg-bg-muted text-fg-subtle" }
const _hoisted_6 = { class: "block text-sm font-mono text-fg-muted mb-1" }
import type { DownloadRange, SearchScope, SecurityFilter, StructuredFilters, UpdatedWithin } from '#shared/types/preferences'
import { DOWNLOAD_RANGES, SEARCH_SCOPE_VALUES, SECURITY_FILTER_VALUES, UPDATED_WITHIN_OPTIONS } from '#shared/types/preferences'

export default /*@__PURE__*/_defineComponent({
  __name: 'Panel',
  props: {
    filters: { type: null, required: true },
    availableKeywords: { type: Array, required: false }
  },
  emits: ["update", "update", "update", "update", "update", "toggleKeyword"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const { t } = useI18n()
const isExpanded = shallowRef(false)
const showAllKeywords = shallowRef(false)
const filterText = computed({
  get: () => props.filters.text,
  set: value => emit('update:text', value),
})
const displayedKeywords = computed(() => {
  const keywords = props.availableKeywords ?? []
  return showAllKeywords.value ? keywords : keywords.slice(0, 20)
})
const searchPlaceholder = computed(() => {
  switch (props.filters.searchScope) {
    case 'name':
      return $t('filters.search_placeholder_name')
    case 'description':
      return $t('filters.search_placeholder_description')
    case 'keywords':
      return $t('filters.search_placeholder_keywords')
    case 'all':
      return $t('filters.search_placeholder_all')
    default:
      return $t('filters.search_placeholder_name')
  }
})
const hasMoreKeywords = computed(() => {
  return !showAllKeywords.value && (props.availableKeywords?.length ?? 0) > 20
})
// i18n mappings for filter options
const scopeLabelKeys = computed(
  () =>
    ({
      name: t('filters.scope_name'),
      description: t('filters.scope_description'),
      keywords: t('filters.scope_keywords'),
      all: t('filters.scope_all'),
    }) as const,
)
const scopeDescriptionKeys = computed(
  () =>
    ({
      name: t('filters.scope_name_description'),
      description: t('filters.scope_description_description'),
      keywords: t('filters.scope_keywords_description'),
      all: t('filters.scope_all_description'),
    }) as const,
)
const downloadRangeLabelKeys = computed(
  () =>
    ({
      'any': t('filters.download_range.any'),
      'lt100': t('filters.download_range.lt100'),
      '100-1k': t('filters.download_range.100_1k'),
      '1k-10k': t('filters.download_range.1k_10k'),
      '10k-100k': t('filters.download_range.10k_100k'),
      'gt100k': t('filters.download_range.gt100k'),
    }) as const,
)
const updatedWithinLabelKeys = computed(
  () =>
    ({
      any: t('filters.updated.any'),
      week: t('filters.updated.week'),
      month: t('filters.updated.month'),
      quarter: t('filters.updated.quarter'),
      year: t('filters.updated.year'),
    }) as const,
)
const securityLabelKeys = computed(
  () =>
    ({
      all: t('filters.security_options.all'),
      secure: t('filters.security_options.secure'),
      warnings: t('filters.security_options.insecure'),
    }) as const,
)
// Type-safe accessor functions
function getScopeLabelKey(value: SearchScope): string {
  return scopeLabelKeys.value[value]
}
function getScopeDescriptionKey(value: SearchScope): string {
  return scopeDescriptionKeys.value[value]
}
function getDownloadRangeLabelKey(value: DownloadRange): string {
  return downloadRangeLabelKeys.value[value]
}
function getUpdatedWithinLabelKey(value: UpdatedWithin): string {
  return updatedWithinLabelKeys.value[value]
}
function getSecurityLabelKey(value: SecurityFilter): string {
  return securityLabelKeys.value[value]
}
// Compact summary of active filters for collapsed header using operator syntax
const filterSummary = computed(() => {
  const parts: string[] = []

  // Text search with operator format
  if (props.filters.text) {
    if (props.filters.searchScope === 'all') {
      // Show raw text (may already contain operators)
      parts.push(props.filters.text)
    } else {
      // Convert scope to operator format
      const operatorMap: Record<string, string> = {
        name: 'name',
        description: 'desc',
        keywords: 'kw',
      }
      const op = operatorMap[props.filters.searchScope] ?? 'name'
      parts.push(`${op}:${props.filters.text}`)
    }
  }

  // Keywords from filter (not from text operators)
  if (props.filters.keywords.length > 0) {
    parts.push(`kw:${props.filters.keywords.join(',')}`)
  }

  // Download range (use compact value, not human label)
  if (props.filters.downloadRange !== 'any') {
    parts.push(`dl:${props.filters.downloadRange}`)
  }

  // Updated within (use compact value, not human label)
  if (props.filters.updatedWithin !== 'any') {
    parts.push(`updated:${props.filters.updatedWithin}`)
  }

  // Security (when enabled)
  if (props.filters.security !== 'all') {
    const label = props.filters.security === 'secure' ? 'secure' : 'warnings'
    parts.push(`security:${label}`)
  }

  return parts.length > 0 ? parts.join(' ') : null
})
const hasActiveFilters = computed(() => !!filterSummary.value)

return (_ctx: any,_cache: any) => {
  const _component_InputBase = _resolveComponent("InputBase")
  const _component_TagRadioButton = _resolveComponent("TagRadioButton")
  const _component_ButtonBase = _resolveComponent("ButtonBase")

  return (_openBlock(), _createElementBlock("div", { class: "border border-border rounded-lg bg-bg-subtle" }, [ _createElementVNode("button", {
        type: "button",
        class: "w-full flex items-center gap-3 px-4 py-3 text-start hover:bg-bg-muted transition-colors duration-200 focus-visible:ring-2 focus-visible:ring-fg focus-visible:ring-inset",
        "aria-expanded": isExpanded.value,
        onClick: _cache[0] || (_cache[0] = ($event: any) => (isExpanded.value = !isExpanded.value))
      }, [ _createElementVNode("span", { class: "flex items-center gap-2 text-sm font-mono text-fg shrink-0" }, [ _hoisted_1, _createTextVNode("\n        " + _toDisplayString(_ctx.$t('filters.title')), 1 /* TEXT */) ]), (!isExpanded.value && hasActiveFilters.value) ? (_openBlock(), _createElementBlock("span", {
            key: 0,
            class: "text-xs font-mono text-fg-muted truncate"
          }, _toDisplayString(filterSummary.value), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createElementVNode("span", {
          class: _normalizeClass(["i-lucide:chevron-down w-4 h-4 text-fg-subtle transition-transform duration-200 shrink-0 ms-auto", { 'rotate-180': isExpanded.value }]),
          "aria-hidden": "true"
        }, null, 2 /* CLASS */) ], 8 /* PROPS */, ["aria-expanded"]), _createTextVNode("\n\n    " + "\n    "), _createVNode(_Transition, { name: "expand" }, {
        default: _withCtx(() => [
          (isExpanded.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "px-4 pb-5 border-t border-border"
            }, [
              _createElementVNode("div", { class: "pt-4" }, [
                _createElementVNode("div", { class: "flex items-center gap-3 mb-1" }, [
                  _createElementVNode("label", _hoisted_2, _toDisplayString(_ctx.$t('filters.search')), 1 /* TEXT */),
                  _createTextVNode("\n            " + "\n            "),
                  _createElementVNode("div", {
                    class: "inline-flex rounded-md border border-border p-0.5 bg-bg-muted",
                    role: "group",
                    "aria-label": _ctx.$t('filters.search_scope')
                  }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(SEARCH_SCOPE_VALUES), (scope) => {
                      return (_openBlock(), _createElementBlock("button", {
                        key: scope,
                        type: "button",
                        class: _normalizeClass(["px-2 py-0.5 text-xs font-mono rounded-sm border transition-colors duration-200 focus-visible:ring-2 focus-visible:ring-fg focus-visible:ring-offset-1", 
                    __props.filters.searchScope === scope
                      ? 'bg-bg-subtle text-fg border-fg-subtle'
                      : 'text-fg-muted hover:text-fg border-transparent'
                  ]),
                        "aria-pressed": __props.filters.searchScope === scope,
                        title: getScopeDescriptionKey(scope),
                        onClick: _cache[1] || (_cache[1] = ($event: any) => (emit('update:searchScope', scope)))
                      }, _toDisplayString(getScopeLabelKey(scope)), 11 /* TEXT, CLASS, PROPS */, ["aria-pressed", "title"]))
                    }), 128 /* KEYED_FRAGMENT */))
                  ], 8 /* PROPS */, ["aria-label"])
                ]),
                _createVNode(_component_InputBase, {
                  id: "filter-search",
                  type: "text",
                  placeholder: searchPlaceholder.value,
                  autocomplete: "off",
                  class: "w-full min-w-25",
                  size: "medium",
                  "no-correct": "",
                  modelValue: filterText.value,
                  "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((filterText).value = $event))
                }, null, 8 /* PROPS */, ["placeholder", "modelValue"])
              ]),
              _createTextVNode("\n\n        "),
              _createTextVNode("\n        "),
              _createElementVNode("fieldset", { class: "border-0 p-0 m-0 mt-4" }, [
                _createElementVNode("legend", _hoisted_3, _toDisplayString(_ctx.$t('filters.weekly_downloads')), 1 /* TEXT */),
                _createElementVNode("div", {
                  class: "flex flex-wrap gap-2",
                  role: "radiogroup",
                  "aria-label": _ctx.$t('filters.weekly_downloads')
                }, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(DOWNLOAD_RANGES), (range) => {
                    return (_openBlock(), _createBlock(_component_TagRadioButton, {
                      key: range.value,
                      "model-value": __props.filters.downloadRange,
                      value: range.value,
                      "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => (emit("update:downloadRange", $event))),
                      name: "range"
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(getDownloadRangeLabelKey(range.value)), 1 /* TEXT */)
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["model-value", "value"]))
                  }), 128 /* KEYED_FRAGMENT */))
                ], 8 /* PROPS */, ["aria-label"])
              ]),
              _createTextVNode("\n\n        "),
              _createTextVNode("\n        "),
              _createElementVNode("fieldset", { class: "border-0 p-0 m-0 mt-4" }, [
                _createElementVNode("legend", _hoisted_4, _toDisplayString(_ctx.$t('filters.updated_within')), 1 /* TEXT */),
                _createElementVNode("div", {
                  class: "flex flex-wrap gap-2",
                  role: "radiogroup",
                  "aria-label": _ctx.$t('filters.updated_within')
                }, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(UPDATED_WITHIN_OPTIONS), (option) => {
                    return (_openBlock(), _createBlock(_component_TagRadioButton, {
                      key: option.value,
                      "model-value": __props.filters.updatedWithin,
                      value: option.value,
                      name: "updatedWithin",
                      "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => (emit("update:updatedWithin", $event)))
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(getUpdatedWithinLabelKey(option.value)), 1 /* TEXT */)
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["model-value", "value"]))
                  }), 128 /* KEYED_FRAGMENT */))
                ], 8 /* PROPS */, ["aria-label"])
              ]),
              _createTextVNode("\n\n        "),
              _createTextVNode("\n        "),
              _createElementVNode("fieldset", { class: "border-0 p-0 m-0 mt-4" }, [
                _createElementVNode("legend", { class: "flex items-center gap-2 text-sm font-mono text-fg-muted mb-1" }, [
                  _createTextVNode(_toDisplayString(_ctx.$t('filters.security')) + "\n            ", 1 /* TEXT */),
                  _createElementVNode("span", _hoisted_5, _toDisplayString(_ctx.$t('filters.columns.coming_soon')), 1 /* TEXT */)
                ]),
                _createElementVNode("div", {
                  class: "flex flex-wrap gap-2",
                  role: "radiogroup",
                  "aria-label": _ctx.$t('filters.security')
                }, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(SECURITY_FILTER_VALUES), (security) => {
                    return (_openBlock(), _createBlock(_component_TagRadioButton, {
                      key: security,
                      disabled: "",
                      "model-value": __props.filters.security,
                      value: security,
                      name: "security"
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(getSecurityLabelKey(security)), 1 /* TEXT */)
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["model-value", "value"]))
                  }), 128 /* KEYED_FRAGMENT */))
                ], 8 /* PROPS */, ["aria-label"])
              ]),
              _createTextVNode("\n\n        "),
              _createTextVNode("\n        "),
              (displayedKeywords.value.length > 0)
                ? (_openBlock(), _createElementBlock("fieldset", {
                  key: 0,
                  class: "border-0 p-0 m-0 mt-4"
                }, [
                  _createElementVNode("legend", _hoisted_6, _toDisplayString(_ctx.$t('filters.keywords')), 1 /* TEXT */),
                  _createElementVNode("div", {
                    class: "flex flex-wrap gap-1.5",
                    role: "group",
                    "aria-label": _ctx.$t('filters.keywords')
                  }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(displayedKeywords.value, (keyword) => {
                      return (_openBlock(), _createBlock(_component_ButtonBase, {
                        key: keyword,
                        size: "small",
                        "aria-pressed": __props.filters.keywords.includes(keyword),
                        onClick: _cache[5] || (_cache[5] = ($event: any) => (emit('toggleKeyword', keyword)))
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(keyword), 1 /* TEXT */)
                        ]),
                        _: 2 /* DYNAMIC */
                      }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["aria-pressed"]))
                    }), 128 /* KEYED_FRAGMENT */)),
                    (hasMoreKeywords.value)
                      ? (_openBlock(), _createElementBlock("button", {
                        key: 0,
                        type: "button",
                        class: "text-xs text-fg-subtle self-center font-mono hover:text-fg transition-colors duration-200 focus-visible:ring-2 focus-visible:ring-fg focus-visible:ring-offset-1",
                        onClick: _cache[6] || (_cache[6] = ($event: any) => (showAllKeywords.value = true))
                      }, _toDisplayString(_ctx.$t('filters.more_keywords', { count: (__props.availableKeywords?.length ?? 0) - 20 })), 1 /* TEXT */))
                      : _createCommentVNode("v-if", true)
                  ], 8 /* PROPS */, ["aria-label"])
                ]))
                : _createCommentVNode("v-if", true)
            ]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }) ]))
}
}

})
