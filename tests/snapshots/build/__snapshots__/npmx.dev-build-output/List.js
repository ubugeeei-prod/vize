import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeStyle as _normalizeStyle, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "w-5 h-5 border-2 border-fg-subtle border-t-fg rounded-full motion-safe:animate-spin" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "w-5 h-5 border-2 border-fg-subtle border-t-fg rounded-full motion-safe:animate-spin" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "w-4 h-4 border-2 border-fg-subtle border-t-fg rounded-full motion-safe:animate-spin" })
import type { NpmSearchResult } from '#shared/types'
import type { WindowVirtualizerHandle } from '~/composables/useVirtualInfiniteScroll'
import type { ColumnConfig, PageSize, PaginationMode, SortOption, ViewMode } from '#shared/types/preferences'
import { DEFAULT_COLUMNS } from '#shared/types/preferences'
import { WindowVirtualizer } from 'virtua/vue'
const SSR_COUNT = 20

export default /*@__PURE__*/_defineComponent({
  __name: 'List',
  props: {
    results: { type: Array, required: true },
    filters: { type: null, required: false },
    headingLevel: { type: String, required: false },
    showPublisher: { type: Boolean, required: false },
    hasMore: { type: Boolean, required: false },
    isLoading: { type: Boolean, required: false },
    pageSize: { type: null, required: false },
    initialPage: { type: Number, required: false },
    searchQuery: { type: String, required: false },
    viewMode: { type: null, required: false },
    columns: { type: Array, required: false },
    paginationMode: { type: null, required: false },
    currentPage: { type: Number, required: false },
    searchContext: { type: Boolean, required: false },
    "sortOption": {}
  },
  emits: ["loadMore", "pageChange", "update", "clickKeyword", "update:sortOption"],
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const sortOption = _useModel(__props, "sortOption")
/** Number of items to render statically during SSR */
// Reference to WindowVirtualizer for infinite scroll detection
const listRef = useTemplateRef<WindowVirtualizerHandle>('listRef')
/** Sort option for table header sorting */
// View mode and columns
const viewMode = computed(() => props.viewMode ?? 'cards')
const columns = computed(() => {
  const targetColumns = props.columns ?? DEFAULT_COLUMNS
  if (props.searchContext) return targetColumns.map(column => ({ ...column, sortable: false }))
  return targetColumns
})
// Table view forces pagination mode (no virtualization for tables)
const paginationMode = computed(() =>
  viewMode.value === 'table' ? 'paginated' : (props.paginationMode ?? 'infinite'),
)
const currentPage = computed(() => props.currentPage ?? 1)
const pageSize = computed(() => props.pageSize ?? 25)
// Numeric page size for virtual scroll and arithmetic (when 'all' is selected, use 25 as default)
const numericPageSize = computed(() => (pageSize.value === 'all' ? 25 : pageSize.value))
// Compute paginated results for paginated mode
const displayedResults = computed(() => {
  if (paginationMode.value === 'infinite') {
    return props.results
  }
  // 'all' page size means show everything (YOLO)
  if (pageSize.value === 'all') {
    return props.results
  }
  const start = (currentPage.value - 1) * numericPageSize.value
  const end = start + numericPageSize.value
  return props.results.slice(start, end)
})
// Set up infinite scroll if hasMore is provided
const hasMore = computed(() => props.hasMore ?? false)
const isLoading = computed(() => props.isLoading ?? false)
const itemCount = computed(() => props.results.length)
const { handleScroll, scrollToPage } = useVirtualInfiniteScroll({
  listRef,
  itemCount,
  hasMore,
  isLoading,
  pageSize: numericPageSize,
  threshold: 5,
  onLoadMore: () => emit('loadMore'),
  onPageChange: page => emit('pageChange', page),
})
// Scroll to initial page once list is ready and has items
const hasScrolledToInitial = shallowRef(false)
watch(
  [() => props.results.length, () => props.initialPage, listRef],
  ([length, initialPage, list]) => {
    if (!hasScrolledToInitial.value && list && length > 0 && initialPage && initialPage > 1) {
      // Wait for next tick to ensure list is rendered
      nextTick(() => {
        scrollToPage(initialPage)
        hasScrolledToInitial.value = true
      })
    }
  },
  { immediate: true },
)
// Reset scroll state when results change significantly (new search)
watch(
  () => props.results,
  (newResults, oldResults) => {
    // If this looks like a new search (different first item or much shorter), reset
    if (
      !oldResults ||
      newResults.length === 0 ||
      (oldResults.length > 0 && newResults[0]?.package.name !== oldResults[0]?.package.name)
    ) {
      hasScrolledToInitial.value = false
    }
  },
)
function scrollToIndex(index: number, smooth = true) {
  listRef.value?.scrollToIndex(index, { align: 'center', smooth })
}
__expose({
  scrollToIndex,
})

return (_ctx: any,_cache: any) => {
  const _component_PackageTable = _resolveComponent("PackageTable")
  const _component_PackageCard = _resolveComponent("PackageCard")
  const _component_ClientOnly = _resolveComponent("ClientOnly")

  return (_openBlock(), _createElementBlock("div", null, [ (viewMode.value === 'table') ? (_openBlock(), _createBlock(_component_PackageTable, {
          key: 0,
          results: displayedResults.value,
          filters: __props.filters,
          columns: columns.value,
          "is-loading": isLoading.value,
          onClickKeyword: _cache[0] || (_cache[0] = ($event: any) => (emit('clickKeyword', $event))),
          "sort-option": sortOption.value,
          "onUpdate:sort-option": _cache[1] || (_cache[1] = ($event: any) => ((sortOption).value = $event))
        }, null, 8 /* PROPS */, ["results", "filters", "columns", "is-loading", "sort-option"])) : (paginationMode.value === 'infinite') ? (_openBlock(), _createBlock(_component_ClientOnly, { key: 1 }, {
            fallback: _withCtx(() => [
              _createElementVNode("ol", { class: "list-none m-0 p-0" }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.results.slice(0, SSR_COUNT), (item, index) => {
                  return (_openBlock(), _createElementBlock("li", { key: item.package.name }, [
                    _createElementVNode("div", { class: "pb-4" }, [
                      _createVNode(_component_PackageCard, {
                        result: item,
                        "heading-level": __props.headingLevel,
                        "show-publisher": __props.showPublisher,
                        index: index,
                        "search-query": __props.searchQuery,
                        filters: __props.filters,
                        onClickKeyword: _cache[2] || (_cache[2] = ($event: any) => (emit('clickKeyword', $event)))
                      }, null, 8 /* PROPS */, ["result", "heading-level", "show-publisher", "index", "search-query", "filters"])
                    ])
                  ]))
                }), 128 /* KEYED_FRAGMENT */))
              ])
            ]),
            default: _withCtx(() => [
              _createVNode(WindowVirtualizer, {
                ref_key: "listRef", ref: listRef,
                data: __props.results,
                "item-size": 140,
                as: "ol",
                item: "li",
                class: "list-none m-0 p-0",
                onScroll: _cache[3] || (_cache[3] = (...args) => (handleScroll && handleScroll(...args)))
              }, {
                default: _withCtx(({ item, index }) => [
                  _createElementVNode("div", { class: "pb-4" }, [
                    _createVNode(_component_PackageCard, {
                      result: item,
                      "heading-level": __props.headingLevel,
                      "show-publisher": __props.showPublisher,
                      index: index,
                      "search-query": __props.searchQuery,
                      class: "motion-safe:animate-fade-in motion-safe:animate-fill-both",
                      filters: __props.filters,
                      style: { animationDelay: `${Math.min(index * 0.02, 0.3)}s` },
                      onClickKeyword: _cache[4] || (_cache[4] = ($event: any) => (emit('clickKeyword', $event)))
                    }, null, 8 /* PROPS */, ["result", "heading-level", "show-publisher", "index", "search-query", "filters"])
                  ])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["data", "item-size"]),
              _createTextVNode("\n\n        "),
              _createTextVNode("\n        ")
            ]),
            _: 1 /* STABLE */
          })) : (_openBlock(), _createElementBlock(_Fragment, { key: 2 }, [ (isLoading.value && displayedResults.value.length === 0) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "py-12 flex items-center justify-center"
            }, [ _createElementVNode("div", { class: "flex items-center gap-3 text-fg-muted font-mono text-sm" }, [ _hoisted_1, _createTextVNode("\n          " + _toDisplayString(_ctx.$t('common.loading')), 1 /* TEXT */) ]) ])) : (_openBlock(), _createElementBlock("ol", {
              key: 1,
              class: "list-none m-0 p-0"
            }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(displayedResults.value, (item, index) => {
                return (_openBlock(), _createElementBlock("li", {
                  key: item.package.name,
                  class: "pb-4"
                }, [
                  _createVNode(_component_PackageCard, {
                    result: item,
                    "heading-level": __props.headingLevel,
                    "show-publisher": __props.showPublisher,
                    index: index,
                    "search-query": __props.searchQuery,
                    class: "motion-safe:animate-fade-in motion-safe:animate-fill-both",
                    style: _normalizeStyle({ animationDelay: `${Math.min(index * 0.02, 0.3)}s` }),
                    filters: __props.filters,
                    onClickKeyword: _cache[5] || (_cache[5] = ($event: any) => (emit('clickKeyword', $event)))
                  }, null, 12 /* STYLE, PROPS */, ["result", "heading-level", "show-publisher", "index", "search-query", "filters"])
                ]))
              }), 128 /* KEYED_FRAGMENT */)) ])) ], 64 /* STABLE_FRAGMENT */)), _createTextVNode("\n\n    " + "\n    " + "\n\n    " + "\n    " + "\n\n    " + "\n    "), (isLoading.value && __props.results.length === 0 && viewMode.value !== 'table') ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "py-12 flex items-center justify-center"
        }, [ _createElementVNode("div", { class: "flex items-center gap-3 text-fg-muted font-mono text-sm" }, [ _hoisted_2, _createTextVNode("\n        " + _toDisplayString(_ctx.$t('common.loading')), 1 /* TEXT */) ]) ])) : (isLoading.value && paginationMode.value === 'infinite') ? (_openBlock(), _createElementBlock("div", {
            key: 1,
            class: "py-4 flex items-center justify-center"
          }, [ _createElementVNode("div", { class: "flex items-center gap-3 text-fg-muted font-mono text-sm" }, [ _hoisted_3, _createTextVNode("\n        " + _toDisplayString(_ctx.$t('common.loading_more')), 1 /* TEXT */) ]) ])) : (!hasMore.value && __props.results.length > 0 && paginationMode.value === 'infinite') ? (_openBlock(), _createElementBlock("p", {
            key: 2,
            class: "py-4 text-center text-fg-subtle font-mono text-sm"
          }, _toDisplayString(_ctx.$t('common.end_of_results')), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    " + "\n    " + "\n\n    " + "\n    " + "\n\n    " + "\n    "), (__props.results.length === 0 && !isLoading.value && viewMode.value !== 'table') ? (_openBlock(), _createElementBlock("p", {
          key: 0,
          class: "py-12 text-center text-fg-subtle font-mono text-sm"
        }, _toDisplayString(_ctx.$t('filters.table.no_packages')), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]))
}
}

})
