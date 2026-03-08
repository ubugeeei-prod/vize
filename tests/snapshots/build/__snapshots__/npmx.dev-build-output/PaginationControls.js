import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"


const _hoisted_1 = { class: "text-sm font-mono text-fg-muted" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:chevron-left block rtl-flip w-4 h-4", "aria-hidden": "true" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:chevron-right block rtl-flip w-4 h-4", "aria-hidden": "true" })
import type { PageSize, PaginationMode, ViewMode } from '#shared/types/preferences'
import { PAGE_SIZE_OPTIONS } from '#shared/types/preferences'

export default /*@__PURE__*/_defineComponent({
  __name: 'PaginationControls',
  props: {
    totalItems: { type: Number, required: true },
    viewMode: { type: null, required: false },
    "mode": { required: true },
    "pageSize": { required: true },
    "currentPage": { required: true }
  },
  emits: ["update:mode", "update:pageSize", "update:currentPage"],
  setup(__props: any) {

const props = __props
const mode = _useModel(__props, "mode")
const pageSize = _useModel(__props, "pageSize")
const currentPage = _useModel(__props, "currentPage")
const pageSizeSelectValue = computed(() => String(pageSize.value))
// Whether we should show pagination controls (table view always uses pagination)
const shouldShowControls = computed(() => props.viewMode === 'table' || mode.value === 'paginated')
// Table view forces pagination mode, otherwise use the provided mode
const effectiveMode = computed<PaginationMode>(() =>
  shouldShowControls.value ? 'paginated' : 'infinite',
)
// When 'all' is selected, there's only 1 page with everything
const isShowingAll = computed(() => pageSize.value === 'all')
const totalPages = computed(() =>
  isShowingAll.value ? 1 : Math.ceil(props.totalItems / (pageSize.value as number)),
)
// Whether to show the mode toggle (hidden in table view since table always uses pagination)
const showModeToggle = computed(() => props.viewMode !== 'table')
const startItem = computed(() => {
  if (props.totalItems === 0) return 0
  if (isShowingAll.value) return 1
  return (currentPage.value - 1) * (pageSize.value as number) + 1
})
const endItem = computed(() => {
  if (isShowingAll.value) return props.totalItems
  return Math.min(currentPage.value * (pageSize.value as number), props.totalItems)
})
const canGoPrev = computed(() => currentPage.value > 1)
const canGoNext = computed(() => currentPage.value < totalPages.value)
function goToPage(page: number) {
  if (page >= 1 && page <= totalPages.value) {
    currentPage.value = page
  }
}
function goPrev() {
  if (canGoPrev.value) {
    currentPage.value = currentPage.value - 1
  }
}
function goNext() {
  if (canGoNext.value) {
    currentPage.value = currentPage.value + 1
  }
}
// Generate visible page numbers with ellipsis
const visiblePages = computed(() => {
  const total = totalPages.value
  const current = currentPage.value
  const pages: (number | 'ellipsis')[] = []

  if (total <= 7) {
    // Show all pages
    for (let i = 1; i <= total; i++) {
      pages.push(i)
    }
  } else {
    // Always show first page
    pages.push(1)

    if (current > 3) {
      pages.push('ellipsis')
    }

    // Show pages around current
    const start = Math.max(2, current - 1)
    const end = Math.min(total - 1, current + 1)

    for (let i = start; i <= end; i++) {
      pages.push(i)
    }

    if (current < total - 2) {
      pages.push('ellipsis')
    }

    // Always show last page
    if (total > 1) {
      pages.push(total)
    }
  }

  return pages
})
function handlePageSizeChange(event: Event) {
  const target = event.target as HTMLSelectElement
  const value = target.value
  // Handle 'all' as a special string value, otherwise parse as number
  const newSize = (value === 'all' ? 'all' : Number(value)) as PageSize
  pageSize.value = newSize
  // Reset to page 1 when changing page size
  currentPage.value = 1
}

return (_ctx: any,_cache: any) => {
  const _component_SelectField = _resolveComponent("SelectField")

  return (shouldShowControls.value)
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        class: "flex flex-wrap items-center justify-between gap-4 py-4 mt-2"
      }, [ _createElementVNode("div", { class: "flex items-center gap-4" }, [ (showModeToggle.value) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "inline-flex rounded-md border border-border p-0.5 bg-bg-subtle",
              role: "group",
              "aria-label": _ctx.$t('filters.pagination.mode_label')
            }, [ _createElementVNode("button", {
                type: "button",
                class: _normalizeClass(["px-2.5 py-1 text-xs font-mono rounded-sm transition-colors duration-200 focus-visible:ring-2 focus-visible:ring-fg focus-visible:ring-offset-1", mode.value === 'infinite' ? 'bg-bg-muted text-fg' : 'text-fg-muted hover:text-fg']),
                "aria-pressed": mode.value === 'infinite',
                onClick: _cache[0] || (_cache[0] = ($event: any) => (mode.value = 'infinite'))
              }, _toDisplayString(_ctx.$t('filters.pagination.infinite')), 11 /* TEXT, CLASS, PROPS */, ["aria-pressed"]), _createElementVNode("button", {
                type: "button",
                class: _normalizeClass(["px-2.5 py-1 text-xs font-mono rounded-sm transition-colors duration-200 focus-visible:ring-2 focus-visible:ring-fg focus-visible:ring-offset-1", mode.value === 'paginated' ? 'bg-bg-muted text-fg' : 'text-fg-muted hover:text-fg']),
                "aria-pressed": mode.value === 'paginated',
                onClick: _cache[1] || (_cache[1] = ($event: any) => (mode.value = 'paginated'))
              }, _toDisplayString(_ctx.$t('filters.pagination.paginated')), 11 /* TEXT, CLASS, PROPS */, ["aria-pressed"]) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n      " + "\n      "), (effectiveMode.value === 'paginated') ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "relative shrink-0"
            }, [ _createVNode(_component_SelectField, {
                label: _ctx.$t('filters.pagination.items_per_page'),
                "hidden-label": "",
                id: "page-size",
                onChange: handlePageSizeChange,
                items: _unref(PAGE_SIZE_OPTIONS).map((size) => ({
  	label: size === "all" ? _ctx.$t("filters.pagination.all_yolo") : _ctx.$t("filters.pagination.per_page", { count: size }),
  	value: String(size)
  })),
                modelValue: pageSizeSelectValue.value,
                "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((pageSizeSelectValue).value = $event))
              }, null, 8 /* PROPS */, ["label", "items", "modelValue"]) ])) : _createCommentVNode("v-if", true) ]), _createTextVNode("\n\n    "), _createTextVNode("\n    "), (effectiveMode.value === 'paginated') ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "flex items-center gap-4"
          }, [ _createElementVNode("span", _hoisted_1, _toDisplayString(_ctx.$t('filters.pagination.showing', {
              start: startItem.value,
              end: endItem.value,
              total: _ctx.$n(__props.totalItems),
            })), 1 /* TEXT */), _createTextVNode("\n\n      "), _createTextVNode("\n      "), (totalPages.value > 1) ? (_openBlock(), _createElementBlock("nav", {
                key: 0,
                class: "flex items-center gap-1",
                "aria-label": _ctx.$t('filters.pagination.nav_label')
              }, [ _createElementVNode("button", {
                  type: "button",
                  class: "p-1.5 rounded hover:bg-bg-muted text-fg-muted hover:text-fg disabled:opacity-40 disabled:cursor-not-allowed transition-colors duration-200 focus-visible:ring-2 focus-visible:ring-fg focus-visible:ring-offset-1",
                  disabled: !canGoPrev.value,
                  "aria-label": _ctx.$t('filters.pagination.previous'),
                  onClick: goPrev
                }, [ _hoisted_2 ], 8 /* PROPS */, ["disabled", "aria-label"]), _createTextVNode("\n\n        "), _createTextVNode("\n        "), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(visiblePages.value, (page, idx) => {
                  return (_openBlock(), _createElementBlock(_Fragment, { key: idx }, [
                    (page === 'ellipsis')
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: "px-2 text-fg-subtle font-mono"
                      }, "…"))
                      : (_openBlock(), _createElementBlock("button", {
                        key: 1,
                        type: "button",
                        class: _normalizeClass(["min-w-[32px] h-8 px-2 font-mono text-sm rounded transition-colors duration-200 focus-visible:ring-2 focus-visible:ring-fg focus-visible:ring-offset-1", 
                page === currentPage.value
                  ? 'bg-fg text-bg'
                  : 'text-fg-muted hover:text-fg hover:bg-bg-muted'
              ]),
                        "aria-current": page === currentPage.value ? 'page' : undefined,
                        onClick: _cache[3] || (_cache[3] = ($event: any) => (goToPage(page)))
                      }, _toDisplayString(page), 1 /* TEXT */))
                  ], 64 /* STABLE_FRAGMENT */))
                }), 128 /* KEYED_FRAGMENT */)), _createTextVNode("\n\n        "), _createTextVNode("\n        "), _createElementVNode("button", {
                  type: "button",
                  class: "p-1.5 rounded hover:bg-bg-muted text-fg-muted hover:text-fg disabled:opacity-40 disabled:cursor-not-allowed transition-colors duration-200 focus-visible:ring-2 focus-visible:ring-fg focus-visible:ring-offset-1",
                  disabled: !canGoNext.value,
                  "aria-label": _ctx.$t('filters.pagination.next'),
                  onClick: goNext
                }, [ _hoisted_3 ], 8 /* PROPS */, ["disabled", "aria-label"]) ])) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true) ]))
      : _createCommentVNode("v-if", true)
}
}

})
