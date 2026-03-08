import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "text-2xl text-fg-subtle font-mono" }
const _hoisted_2 = { class: "font-mono text-2xl sm:text-3xl font-medium" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "i-simple-icons:npm w-4 h-4", "aria-hidden": "true" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:chart-line w-3.5 h-3.5", "aria-hidden": "true" })
const _hoisted_5 = { class: "font-mono" }
const _hoisted_6 = { class: "text-fg-muted mb-4" }
const _hoisted_7 = { class: "text-fg" }
const _hoisted_8 = { class: "text-fg-subtle text-sm mt-2" }
const _hoisted_9 = { class: "text-xs text-fg-subtle uppercase tracking-wider mb-4" }
import type { FilterChip, SortOption } from '#shared/types/preferences'
import { normalizeSearchParam } from '#shared/utils/url'
import { debounce } from 'perfect-debounce'

export default /*@__PURE__*/_defineComponent({
  __name: '[org]',
  setup(__props) {

definePageMeta({
  name: 'org',
})
const route = useRoute('org')
const router = useRouter()
const orgName = computed(() => route.params.org.toLowerCase())
const { isConnected } = useConnector()
// Fetch all packages in this org using the org packages API (lazy to not block navigation)
const { data: results, status, error } = useOrgPackages(orgName)
// Handle 404 errors reactively (since we're not awaiting)
watch(
  [status, error],
  ([newStatus, newError]) => {
    if (newStatus === 'error' && newError?.statusCode === 404) {
      showError({
        statusCode: 404,
        statusMessage: $t('org.page.not_found'),
        message: $t('org.page.not_found_message', { name: orgName.value }),
      })
    }
  },
  { immediate: true },
)
const packages = computed(() => results.value?.objects ?? [])
const packageCount = computed(() => packages.value.length)
// Preferences (persisted to localStorage)
const { viewMode, paginationMode, pageSize, columns, toggleColumn, resetColumns } =
  usePackageListPreferences()
// Structured filters and sorting
const {
  filters,
  sortOption,
  sortedPackages,
  availableKeywords,
  activeFilters,
  setTextFilter,
  setSearchScope,
  setDownloadRange,
  setSecurity,
  setUpdatedWithin,
  toggleKeyword,
  clearFilter,
  clearAllFilters,
  setSort,
} = useStructuredFilters({
  packages,
  initialSort: (normalizeSearchParam(route.query.sort) as SortOption) ?? 'updated-desc',
})
// Pagination state
const currentPage = shallowRef(1)
// Calculate total pages
const totalPages = computed(() => {
  if (pageSize.value === 'all') return 1
  const numericSize = typeof pageSize.value === 'number' ? pageSize.value : 25
  return Math.ceil(sortedPackages.value.length / numericSize)
})
// Reset to page 1 when filters change
watch([filters, sortOption], () => {
  currentPage.value = 1
})
// Clamp current page when total pages decreases (e.g., after filtering)
watch(totalPages, newTotal => {
  if (currentPage.value > newTotal && newTotal > 0) {
    currentPage.value = newTotal
  }
})
// Debounced URL update for filter/sort
const updateUrl = debounce((updates: { filter?: string; sort?: string }) => {
  router.replace({
    query: {
      ...route.query,
      q: updates.filter || undefined,
      sort: updates.sort && updates.sort !== 'updated-desc' ? updates.sort : undefined,
    },
  })
}, 300)
// Update URL when filter/sort changes (debounced)
watch(
  [() => filters.value.text, () => filters.value.keywords, () => sortOption.value] as const,
  ([text, keywords, sort]) => {
    const filter = [text, ...keywords.map(keyword => `keyword:${keyword}`)]
      .filter(Boolean)
      .join(' ')
    updateUrl({ filter, sort })
  },
)
const filteredCount = computed(() => sortedPackages.value.length)
// Total weekly downloads across displayed packages (updates with filter)
const totalWeeklyDownloads = computed(() =>
  sortedPackages.value.reduce((sum, pkg) => sum + (pkg.downloads?.weekly ?? 0), 0),
)
// Reset state when org changes
watch(orgName, () => {
  clearAllFilters()
  setSort('updated-desc')
  currentPage.value = 1
})
// Handle filter chip removal
function handleClearFilter(chip: FilterChip) {
  clearFilter(chip)
}
const activeTab = shallowRef<'members' | 'teams'>('members')
// Canonical URL for this org page
const canonicalUrl = computed(() => `https://npmx.dev/@${orgName.value}`)
useHead({
  link: [{ rel: 'canonical', href: canonicalUrl }],
})
useSeoMeta({
  title: () => `@${orgName.value} - npmx`,
  ogTitle: () => `@${orgName.value} - npmx`,
  twitterTitle: () => `@${orgName.value} - npmx`,
  description: () => `npm packages published by the ${orgName.value} organization`,
  ogDescription: () => `npm packages published by the ${orgName.value} organization`,
  twitterDescription: () => `npm packages published by the ${orgName.value} organization`,
})
defineOgImageComponent('Default', {
  title: () => `@${orgName.value}`,
  description: () => (packageCount.value ? `${packageCount.value} packages` : 'npm organization'),
  primaryColor: '#60a5fa',
})

return (_ctx: any,_cache: any) => {
  const _component_OrgMembersPanel = _resolveComponent("OrgMembersPanel")
  const _component_OrgTeamsPanel = _resolveComponent("OrgTeamsPanel")
  const _component_ClientOnly = _resolveComponent("ClientOnly")
  const _component_LoadingSpinner = _resolveComponent("LoadingSpinner")
  const _component_LinkBase = _resolveComponent("LinkBase")
  const _component_PackageListToolbar = _resolveComponent("PackageListToolbar")
  const _component_PackageList = _resolveComponent("PackageList")
  const _component_PaginationControls = _resolveComponent("PaginationControls")

  return (_openBlock(), _createElementBlock("main", { class: "container flex-1 py-8 sm:py-12 w-full" }, [ _createElementVNode("header", { class: "mb-8 pb-8 border-b border-border" }, [ _createElementVNode("div", { class: "flex flex-wrap items-end gap-4" }, [ _createElementVNode("div", {
            class: "size-16 shrink-0 rounded-lg bg-bg-muted border border-border flex items-center justify-center",
            "aria-hidden": "true"
          }, [ _createElementVNode("span", _hoisted_1, _toDisplayString(orgName.value.charAt(0).toUpperCase()), 1 /* TEXT */) ]), _createElementVNode("div", null, [ _createElementVNode("h1", _hoisted_2, "@" + _toDisplayString(orgName.value), 1 /* TEXT */), (_unref(status) === 'success') ? (_openBlock(), _createElementBlock("p", {
                key: 0,
                class: "text-fg-muted text-sm mt-1"
              }, _toDisplayString(_ctx.$t('org.public_packages', { count: _ctx.$n(packageCount.value) }, packageCount.value)), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", { class: "ms-auto text-end" }, [ _createElementVNode("nav", { "aria-label": "External links" }, [ _createElementVNode("a", {
                href: `https://www.npmjs.com/org/${orgName.value}`,
                target: "_blank",
                rel: "noopener noreferrer",
                class: "link-subtle font-mono text-sm inline-flex items-center gap-1.5",
                title: _ctx.$t('common.view_on_npm')
              }, [ _hoisted_3, _createTextVNode("\n              npm\n            ") ], 8 /* PROPS */, ["href", "title"]) ]), _createElementVNode("p", {
              class: "text-fg-subtle text-xs mt-1 flex items-center gap-1.5 justify-end cursor-help",
              title: _ctx.$t('common.vanity_downloads_hint', { count: filteredCount.value }, filteredCount.value)
            }, [ _hoisted_4, _createElementVNode("span", _hoisted_5, _toDisplayString(_ctx.$n(totalWeeklyDownloads.value)) + " " + _toDisplayString(_ctx.$t('common.per_week')), 1 /* TEXT */) ], 8 /* PROPS */, ["title"]) ]) ]) ]), _createTextVNode("\n\n    " + "\n    "), _createVNode(_component_ClientOnly, null, {
        default: _withCtx(() => [
          (_unref(isConnected))
            ? (_openBlock(), _createElementBlock("section", {
              key: 0,
              class: "mb-8",
              "aria-label": "Organization management"
            }, [
              _createElementVNode("div", { class: "flex items-center gap-1 mb-4" }, [
                _createElementVNode("button", {
                  type: "button",
                  class: _normalizeClass(["px-4 py-2 font-mono text-sm rounded-t-lg transition-colors duration-200", 
                activeTab.value === 'members'
                  ? 'bg-bg-subtle text-fg border border-border border-b-0'
                  : 'text-fg-muted hover:text-fg'
              ]),
                  onClick: _cache[0] || (_cache[0] = ($event: any) => (activeTab.value = 'members'))
                }, _toDisplayString(_ctx.$t('org.page.members_tab')), 3 /* TEXT, CLASS */),
                _createElementVNode("button", {
                  type: "button",
                  class: _normalizeClass(["px-4 py-2 font-mono text-sm rounded-t-lg transition-colors duration-200", 
                activeTab.value === 'teams'
                  ? 'bg-bg-subtle text-fg border border-border border-b-0'
                  : 'text-fg-muted hover:text-fg'
              ]),
                  onClick: _cache[1] || (_cache[1] = ($event: any) => (activeTab.value = 'teams'))
                }, _toDisplayString(_ctx.$t('org.page.teams_tab')), 3 /* TEXT, CLASS */)
              ]),
              _createTextVNode("\n\n        "),
              _createTextVNode("\n        "),
              (activeTab.value === 'members')
                ? (_openBlock(), _createBlock(_component_OrgMembersPanel, {
                  key: 0,
                  "org-name": orgName.value
                }, null, 8 /* PROPS */, ["org-name"]))
                : (_openBlock(), _createBlock(_component_OrgTeamsPanel, {
                  key: 1,
                  "org-name": orgName.value
                }, null, 8 /* PROPS */, ["org-name"]))
            ]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }), _createTextVNode("\n\n    " + "\n    "), (_unref(status) === 'pending') ? (_openBlock(), _createBlock(_component_LoadingSpinner, {
          key: 0,
          text: _ctx.$t('common.loading_packages')
        }, null, 8 /* PROPS */, ["text"])) : (_unref(status) === 'error') ? (_openBlock(), _createElementBlock("div", {
            key: 1,
            role: "alert",
            class: "py-12 text-center"
          }, [ _createElementVNode("p", _hoisted_6, _toDisplayString(_unref(error)?.message ?? _ctx.$t('org.page.failed_to_load')), 1 /* TEXT */), _createVNode(_component_LinkBase, {
              variant: "button-secondary",
              to: { name: 'index' }
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('common.go_back_home')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to"]) ])) : (packageCount.value === 0) ? (_openBlock(), _createElementBlock("div", {
            key: 2,
            class: "py-12 text-center"
          }, [ _createElementVNode("p", { class: "text-fg-muted font-mono" }, [ _createTextVNode(_toDisplayString(_ctx.$t('org.page.no_packages')) + " ", 1 /* TEXT */), _createElementVNode("span", _hoisted_7, "@" + _toDisplayString(orgName.value), 1 /* TEXT */) ]), _createElementVNode("p", _hoisted_8, _toDisplayString(_ctx.$t('org.page.no_packages_hint')), 1 /* TEXT */) ])) : (packages.value.length > 0) ? (_openBlock(), _createElementBlock("section", {
            key: 3,
            "aria-label": _ctx.$t('org.page.packages_title')
          }, [ _createElementVNode("h2", _hoisted_9, _toDisplayString(_ctx.$t('org.page.packages_title')), 1 /* TEXT */), _createTextVNode("\n\n      "), _createTextVNode("\n      "), _createVNode(_component_PackageListToolbar, {
              filters: _unref(filters),
              columns: _unref(columns),
              "total-count": packageCount.value,
              "filtered-count": filteredCount.value,
              "available-keywords": _unref(availableKeywords),
              "active-filters": _unref(activeFilters),
              onToggleColumn: _cache[2] || (_cache[2] = (...args) => (toggleColumn && toggleColumn(...args))),
              onResetColumns: _cache[3] || (_cache[3] = (...args) => (resetColumns && resetColumns(...args))),
              onClearFilter: handleClearFilter,
              onClearAllFilters: _cache[4] || (_cache[4] = (...args) => (clearAllFilters && clearAllFilters(...args))),
              "onUpdate:text": _cache[5] || (_cache[5] = (...args) => (setTextFilter && setTextFilter(...args))),
              "onUpdate:search-scope": _cache[6] || (_cache[6] = (...args) => (setSearchScope && setSearchScope(...args))),
              "onUpdate:download-range": _cache[7] || (_cache[7] = (...args) => (setDownloadRange && setDownloadRange(...args))),
              "onUpdate:security": _cache[8] || (_cache[8] = (...args) => (setSecurity && setSecurity(...args))),
              "onUpdate:updated-within": _cache[9] || (_cache[9] = (...args) => (setUpdatedWithin && setUpdatedWithin(...args))),
              onToggleKeyword: _cache[10] || (_cache[10] = (...args) => (toggleKeyword && toggleKeyword(...args))),
              "sort-option": _unref(sortOption),
              "onUpdate:sort-option": _cache[11] || (_cache[11] = ($event: any) => ((sortOption).value = $event)),
              "view-mode": _unref(viewMode),
              "onUpdate:view-mode": _cache[12] || (_cache[12] = ($event: any) => ((viewMode).value = $event)),
              "pagination-mode": _unref(paginationMode),
              "onUpdate:pagination-mode": _cache[13] || (_cache[13] = ($event: any) => ((paginationMode).value = $event)),
              "page-size": _unref(pageSize),
              "onUpdate:page-size": _cache[14] || (_cache[14] = ($event: any) => ((pageSize).value = $event))
            }, null, 8 /* PROPS */, ["filters", "columns", "total-count", "filtered-count", "available-keywords", "active-filters", "sort-option", "view-mode", "pagination-mode", "page-size"]), _createTextVNode("\n\n      "), _createTextVNode("\n      "), (_unref(sortedPackages).length === 0) ? (_openBlock(), _createElementBlock("p", {
                key: 0,
                class: "text-fg-muted py-8 text-center font-mono"
              }, _toDisplayString(_ctx.$t('org.page.no_match', { query: _unref(filters).text })), 1 /* TEXT */)) : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ _createVNode(_component_PackageList, {
                  results: _unref(sortedPackages),
                  "view-mode": _unref(viewMode),
                  columns: _unref(columns),
                  filters: _unref(filters),
                  "pagination-mode": _unref(paginationMode),
                  "page-size": _unref(pageSize),
                  "current-page": currentPage.value,
                  onClickKeyword: _cache[15] || (_cache[15] = (...args) => (toggleKeyword && toggleKeyword(...args))),
                  "sort-option": _unref(sortOption),
                  "onUpdate:sort-option": _cache[16] || (_cache[16] = ($event: any) => ((sortOption).value = $event))
                }, null, 8 /* PROPS */, ["results", "view-mode", "columns", "filters", "pagination-mode", "page-size", "current-page", "sort-option"]), _createTextVNode("\n\n        "), _createTextVNode("\n        "), _createVNode(_component_PaginationControls, {
                  "total-items": _unref(sortedPackages).length,
                  "view-mode": _unref(viewMode),
                  mode: _unref(paginationMode),
                  "onUpdate:mode": _cache[17] || (_cache[17] = ($event: any) => ((paginationMode).value = $event)),
                  "page-size": _unref(pageSize),
                  "onUpdate:page-size": _cache[18] || (_cache[18] = ($event: any) => ((pageSize).value = $event)),
                  "current-page": currentPage.value,
                  "onUpdate:current-page": _cache[19] || (_cache[19] = ($event: any) => ((currentPage).value = $event))
                }, null, 8 /* PROPS */, ["total-items", "view-mode", "mode", "page-size", "current-page"]) ], 64 /* STABLE_FRAGMENT */)), _createTextVNode("\n\n      "), _createTextVNode("\n      ") ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    " + "\n    " + "\n\n    " + "\n    " + "\n\n    " + "\n    ") ]))
}
}

})
