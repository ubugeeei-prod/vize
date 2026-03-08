import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "font-mono text-3xl sm:text-4xl font-medium" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:arrow-left rtl-flip w-4 h-4", "aria-hidden": "true" })
const _hoisted_3 = { class: "hidden sm:inline" }
const _hoisted_4 = { class: "text-fg-muted text-lg" }
const _hoisted_5 = { id: "packages-heading", class: "text-xs text-fg-subtle uppercase tracking-wider mb-3" }
const _hoisted_6 = { id: "facets-heading", class: "text-xs text-fg-subtle uppercase tracking-wider" }
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("span", { class: "text-3xs text-fg-muted/40", "aria-hidden": "true" }, "/")
const _hoisted_8 = { id: "comparison-heading", class: "text-xs text-fg-subtle uppercase tracking-wider" }
const _hoisted_9 = { id: "trends-comparison-heading", class: "text-xs text-fg-subtle uppercase tracking-wider mb-4 mt-10" }
const _hoisted_10 = { class: "text-fg-muted" }
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("div", { class: "i-lucide:git-compare w-12 h-12 text-fg-subtle mx-auto mb-4", "aria-hidden": "true" })
const _hoisted_12 = { class: "font-mono text-lg text-fg-muted mb-2" }
const _hoisted_13 = { class: "text-sm text-fg-subtle max-w-md mx-auto" }
import { NO_DEPENDENCY_ID } from '~/composables/usePackageComparison'
import { useRouteQuery } from '@vueuse/router'

export default /*@__PURE__*/_defineComponent({
  __name: 'compare',
  setup(__props) {

definePageMeta({
  name: 'compare',
})
const { locale } = useI18n()
const router = useRouter()
const canGoBack = useCanGoBack()
const { copied, copy } = useClipboard({ copiedDuring: 2000 })
// Sync packages with URL query param (stable ref - doesn't change on other query changes)
const packagesParam = useRouteQuery<string>('packages', '', { mode: 'replace' })
// Parse package names from comma-separated string
const packages = computed({
  get() {
    if (!packagesParam.value) return []
    return packagesParam.value
      .split(',')
      .map(p => p.trim())
      .filter(p => p.length > 0)
      .slice(0, 4)
  },
  set(value) {
    packagesParam.value = value.length > 0 ? value.join(',') : ''
  },
})
// Facet selection and info
const { selectedFacets, selectAll, deselectAll, isAllSelected, isNoneSelected } =
  useFacetSelection()
// Fetch comparison data
const { packagesData, status, getFacetValues, isFacetLoading, isColumnLoading } =
  usePackageComparison(packages)
// Fetch module replacement suggestions
const { noDepSuggestions, infoSuggestions, replacements } = useCompareReplacements(packages)
// Whether the "no dependency" baseline column is active
const showNoDependency = computed(() => packages.value.includes(NO_DEPENDENCY_ID))
// Build column definitions for real packages only (no-dep is handled separately by the grid)
const gridColumns = computed(() =>
  packages.value
    .map((pkg, i) => ({ pkg, originalIndex: i }))
    .filter(({ pkg }) => pkg !== NO_DEPENDENCY_ID)
    .map(({ pkg, originalIndex }) => {
      const data = packagesData.value?.[originalIndex]
      return {
        name: data?.package.name || pkg,
        version: data?.package.version,
        replacement: replacements.value.get(pkg) ?? null,
      }
    }),
)
// Whether we can add the no-dep column (not already added and have room)
const canAddNoDep = computed(
  () => packages.value.length < 4 && !packages.value.includes(NO_DEPENDENCY_ID),
)
// Add "no dependency" column to comparison
function addNoDep() {
  if (packages.value.length >= 4) return
  if (packages.value.includes(NO_DEPENDENCY_ID)) return
  packages.value = [...packages.value, NO_DEPENDENCY_ID]
}
// Get loading state for each column
const columnLoading = computed(() => packages.value.map((_, i) => isColumnLoading(i)))
// Check if we have enough packages to compare
const canCompare = computed(() => packages.value.length >= 2)
// Extract headers from columns for facet rows
const gridHeaders = computed(() =>
  gridColumns.value.map(col => (col.version ? `${col.name}@${col.version}` : col.name)),
)
/*
 * Convert the comparison grid data to a Markdown table.
 */
function exportComparisonDataAsMarkdown() {
  const mdData: Array<Array<string>> = []
  const headers = [
    '',
    ...gridHeaders.value,
    ...(showNoDependency.value ? [$t('compare.no_dependency.label')] : []),
  ]
  mdData.push(headers)
  const maxLengths = headers.map(item => item.length)
  selectedFacets.value.forEach((facet, index) => {
    const label = facet.label
    const data = getFacetValues(facet.id)
    mdData.push([
      label,
      ...data.map(item =>
        item?.type === 'date'
          ? new Date(item.display).toLocaleDateString(locale.value, {
              year: 'numeric',
              month: 'short',
              day: 'numeric',
            })
          : item?.display || '',
      ),
    ])
    mdData?.[index + 1]?.forEach((item, itemIndex) => {
      if (item.length > (maxLengths?.[itemIndex] || 0)) {
        maxLengths[itemIndex] = item.length
      }
    })
  })
  const markdown = mdData.reduce((result, row, index) => {
    // replacing pipe `|` with `ǀ` (U+01C0 Latin Letter Dental Click) to avoid breaking tables
    result += `| ${row
      .map((el, ind) => el.padEnd(maxLengths[ind] || 0, ' ').replace(/\|/g, 'ǀ'))
      .join(' | ')} |`
    if (index === 0) {
      result += `\n|`
      maxLengths.forEach(len => (result += ` ${'-'.padEnd(len, '-')} |`))
    }
    result += `\n`
    return result
  }, '')
  copy(markdown)
}
useSeoMeta({
  title: () =>
    packages.value.length > 0
      ? $t('compare.packages.meta_title', { packages: packages.value.join(' vs ') })
      : $t('compare.packages.meta_title_empty'),
  ogTitle: () =>
    packages.value.length > 0
      ? $t('compare.packages.meta_title', { packages: packages.value.join(' vs ') })
      : $t('compare.packages.meta_title_empty'),
  twitterTitle: () =>
    packages.value.length > 0
      ? $t('compare.packages.meta_title', { packages: packages.value.join(' vs ') })
      : $t('compare.packages.meta_title_empty'),
  description: () =>
    packages.value.length > 0
      ? $t('compare.packages.meta_description', { packages: packages.value.join(', ') })
      : $t('compare.packages.meta_description_empty'),
  ogDescription: () =>
    packages.value.length > 0
      ? $t('compare.packages.meta_description', { packages: packages.value.join(', ') })
      : $t('compare.packages.meta_description_empty'),
  twitterDescription: () =>
    packages.value.length > 0
      ? $t('compare.packages.meta_description', { packages: packages.value.join(', ') })
      : $t('compare.packages.meta_description_empty'),
})

return (_ctx: any,_cache: any) => {
  const _component_ComparePackageSelector = _resolveComponent("ComparePackageSelector")
  const _component_CompareReplacementSuggestion = _resolveComponent("CompareReplacementSuggestion")
  const _component_ButtonBase = _resolveComponent("ButtonBase")
  const _component_CompareFacetSelector = _resolveComponent("CompareFacetSelector")
  const _component_CopyToClipboardButton = _resolveComponent("CopyToClipboardButton")
  const _component_LoadingSpinner = _resolveComponent("LoadingSpinner")
  const _component_CompareFacetRow = _resolveComponent("CompareFacetRow")
  const _component_CompareComparisonGrid = _resolveComponent("CompareComparisonGrid")
  const _component_CompareFacetCard = _resolveComponent("CompareFacetCard")
  const _component_CompareLineChart = _resolveComponent("CompareLineChart")

  return (_openBlock(), _createElementBlock("main", { class: "container flex-1 py-12 sm:py-16 w-full" }, [ _createElementVNode("div", { class: "max-w-2xl mx-auto" }, [ _createElementVNode("header", { class: "mb-12" }, [ _createElementVNode("div", { class: "flex items-baseline justify-between gap-4 mb-4" }, [ _createElementVNode("h1", _hoisted_1, _toDisplayString(_ctx.$t('compare.packages.title')), 1 /* TEXT */), (_unref(canGoBack)) ? (_openBlock(), _createElementBlock("button", {
                key: 0,
                type: "button",
                class: "cursor-pointer inline-flex items-center gap-2 font-mono text-sm text-fg-muted hover:text-fg transition-colors duration-200 rounded focus-visible:outline-accent/70 shrink-0",
                onClick: _cache[0] || (_cache[0] = ($event: any) => (_unref(router).back()))
              }, [ _hoisted_2, _createElementVNode("span", _hoisted_3, _toDisplayString(_ctx.$t('nav.back')), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true) ]), _createElementVNode("p", _hoisted_4, _toDisplayString(_ctx.$t('compare.packages.tagline')), 1 /* TEXT */) ]), _createTextVNode("\n\n      " + "\n      "), _createElementVNode("section", {
          class: "mb-8",
          "aria-labelledby": "packages-heading"
        }, [ _createElementVNode("h2", _hoisted_5, _toDisplayString(_ctx.$t('compare.packages.section_packages')), 1 /* TEXT */), _createVNode(_component_ComparePackageSelector, {
            max: 4,
            modelValue: packages.value,
            "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((packages).value = $event))
          }, null, 8 /* PROPS */, ["max", "modelValue"]), _createTextVNode("\n\n        " + "\n        "), (_unref(noDepSuggestions).length > 0) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "mt-3 space-y-2"
            }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(noDepSuggestions), (suggestion) => {
                return (_openBlock(), _createBlock(_component_CompareReplacementSuggestion, {
                  key: suggestion.forPackage,
                  "package-name": suggestion.forPackage,
                  replacement: suggestion.replacement,
                  variant: "nodep",
                  "show-action": canAddNoDep.value,
                  onAddNoDep: addNoDep
                }, null, 8 /* PROPS */, ["package-name", "replacement", "show-action"]))
              }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n        " + "\n        "), (_unref(infoSuggestions).length > 0) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "mt-3 space-y-2"
            }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(infoSuggestions), (suggestion) => {
                return (_openBlock(), _createBlock(_component_CompareReplacementSuggestion, {
                  key: suggestion.forPackage,
                  "package-name": suggestion.forPackage,
                  replacement: suggestion.replacement,
                  variant: "info"
                }, null, 8 /* PROPS */, ["package-name", "replacement"]))
              }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true) ]), _createTextVNode("\n\n      " + "\n      "), _createElementVNode("section", {
          class: "mb-8",
          "aria-labelledby": "facets-heading"
        }, [ _createElementVNode("div", { class: "flex items-center gap-2 mb-3" }, [ _createElementVNode("h2", _hoisted_6, _toDisplayString(_ctx.$t('compare.packages.section_facets')), 1 /* TEXT */), _createVNode(_component_ButtonBase, {
              size: "small",
              "aria-pressed": _unref(isAllSelected),
              disabled: _unref(isAllSelected),
              "aria-label": _ctx.$t('compare.facets.select_all'),
              onClick: _cache[2] || (_cache[2] = (...args) => (selectAll && selectAll(...args)))
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('compare.facets.all')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["aria-pressed", "disabled", "aria-label"]), _hoisted_7, _createVNode(_component_ButtonBase, {
              size: "small",
              "aria-pressed": _unref(isNoneSelected),
              disabled: _unref(isNoneSelected),
              "aria-label": _ctx.$t('compare.facets.deselect_all'),
              onClick: _cache[3] || (_cache[3] = (...args) => (deselectAll && deselectAll(...args)))
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('compare.facets.none')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["aria-pressed", "disabled", "aria-label"]) ]), _createVNode(_component_CompareFacetSelector) ]), _createTextVNode("\n\n      " + "\n      "), (canCompare.value) ? (_openBlock(), _createElementBlock("section", {
            key: 0,
            class: "mt-10",
            "aria-labelledby": "comparison-heading"
          }, [ (_unref(packagesData) && _unref(packagesData).some(p => p !== null)) ? (_openBlock(), _createBlock(_component_CopyToClipboardButton, {
                key: 0,
                copied: _unref(copied),
                "copy-text": _ctx.$t('compare.packages.copy_as_markdown'),
                class: "mb-4",
                "button-attrs": { class: 'hidden md:inline-flex' },
                onClick: exportComparisonDataAsMarkdown
              }, {
                default: _withCtx(() => [
                  _createElementVNode("h2", _hoisted_8, _toDisplayString(_ctx.$t('compare.packages.section_comparison')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["copied", "copy-text", "button-attrs"])) : (_openBlock(), _createElementBlock("h2", {
                key: 1,
                id: "comparison-heading",
                class: "text-xs text-fg-subtle uppercase tracking-wider mb-4"
              }, _toDisplayString(_ctx.$t('compare.packages.section_comparison')), 1 /* TEXT */)), ( (_unref(status) === 'pending' || _unref(status) === 'idle') && (!_unref(packagesData) || _unref(packagesData).every(p => p === null)) ) ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: "flex items-center justify-center py-12"
              }, [ _createVNode(_component_LoadingSpinner, { text: _ctx.$t('compare.packages.loading') }, null, 8 /* PROPS */, ["text"]) ])) : (_unref(packagesData) && _unref(packagesData).some(p => p !== null)) ? (_openBlock(), _createElementBlock("div", { key: 1 }, [ _createElementVNode("div", { class: "hidden md:block overflow-x-auto" }, [ _createVNode(_component_CompareComparisonGrid, {
                      columns: gridColumns.value,
                      "show-no-dependency": showNoDependency.value
                    }, {
                      default: _withCtx(() => [
                        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(selectedFacets), (facet) => {
                          return (_openBlock(), _createBlock(_component_CompareFacetRow, {
                            key: facet.id,
                            label: facet.label,
                            description: facet.description,
                            values: _unref(getFacetValues)(facet.id),
                            "facet-loading": _unref(isFacetLoading)(facet.id),
                            "column-loading": columnLoading.value,
                            bar: facet.id !== 'lastUpdated',
                            headers: gridHeaders.value
                          }, null, 8 /* PROPS */, ["label", "description", "values", "facet-loading", "column-loading", "bar", "headers"]))
                        }), 128 /* KEYED_FRAGMENT */))
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["columns", "show-no-dependency"]) ]), _createTextVNode("\n\n          "), _createTextVNode("\n          "), _createElementVNode("div", { class: "md:hidden space-y-3" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(selectedFacets), (facet) => {
                      return (_openBlock(), _createBlock(_component_CompareFacetCard, {
                        key: facet.id,
                        label: facet.label,
                        description: facet.description,
                        values: _unref(getFacetValues)(facet.id),
                        "facet-loading": _unref(isFacetLoading)(facet.id),
                        "column-loading": columnLoading.value,
                        bar: facet.id !== 'lastUpdated',
                        headers: gridHeaders.value
                      }, null, 8 /* PROPS */, ["label", "description", "values", "facet-loading", "column-loading", "bar", "headers"]))
                    }), 128 /* KEYED_FRAGMENT */)) ]), _createElementVNode("h2", _hoisted_9, _toDisplayString(_ctx.$t('compare.facets.trends.title')), 1 /* TEXT */), _createVNode(_component_CompareLineChart, { packages: packages.value.filter(p => p !== _unref(NO_DEPENDENCY_ID)) }, null, 8 /* PROPS */, ["packages"]) ])) : (_unref(status) === 'error') ? (_openBlock(), _createElementBlock("div", {
                  key: 2,
                  class: "text-center py-12",
                  role: "alert"
                }, [ _createElementVNode("p", _hoisted_10, _toDisplayString(_ctx.$t('compare.packages.error')), 1 /* TEXT */) ])) : (_openBlock(), _createElementBlock("div", {
                key: 3,
                class: "flex items-center justify-center py-12"
              }, [ _createVNode(_component_LoadingSpinner, { text: _ctx.$t('compare.packages.loading') }, null, 8 /* PROPS */, ["text"]) ])) ])) : (_openBlock(), _createElementBlock("section", {
            key: 1,
            class: "text-center px-1.5 py-16 border border-dashed border-border-hover rounded-lg"
          }, [ _hoisted_11, _createElementVNode("h2", _hoisted_12, _toDisplayString(_ctx.$t('compare.packages.empty_title')), 1 /* TEXT */), _createElementVNode("p", _hoisted_13, _toDisplayString(_ctx.$t('compare.packages.empty_description')), 1 /* TEXT */) ])), _createTextVNode("\n\n      " + "\n      ") ]) ]))
}
}

})
