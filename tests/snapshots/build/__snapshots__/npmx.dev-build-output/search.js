import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref, vShow as _vShow } from "vue"


const _hoisted_1 = { class: "font-mono text-2xl sm:text-3xl font-medium" }
const _hoisted_2 = { class: "font-mono text-sm text-fg" }
const _hoisted_3 = { class: "text-xs text-fg-muted mt-0.5" }
const _hoisted_4 = { class: "text-fg-muted font-mono mb-6 text-center" }
const _hoisted_5 = { class: "text-fg-muted font-mono mb-6 text-center" }
const _hoisted_6 = { class: "text-sm text-fg-muted mb-3" }
const _hoisted_7 = { class: "text-fg-subtle font-mono text-sm" }
import type { FilterChip, SortKey } from '#shared/types/preferences'
import { parseSortOption, PROVIDER_SORT_KEYS } from '#shared/types/preferences'
import { onKeyDown } from '@vueuse/core'
import { debounce } from 'perfect-debounce'
import { isValidNewPackageName } from '~/utils/package-name'
import { isPlatformSpecificPackage } from '~/utils/platform-packages'
import { normalizeSearchParam } from '#shared/utils/url'
const pageSize = 25

export default /*@__PURE__*/_defineComponent({
  __name: 'search',
  setup(__props) {

const route = useRoute()
// Preferences (persisted to localStorage)
const {
  viewMode,
  paginationMode,
  pageSize: preferredPageSize,
  columns,
  toggleColumn,
  resetColumns,
} = usePackageListPreferences()
// Debounced URL update for page (less aggressive to avoid too many URL changes)
//Use History API directly to update URL without triggering Router's scroll-to-top
const updateUrlPage = debounce((page: number) => {
  const url = new URL(window.location.href)
  if (page > 1) {
    url.searchParams.set('page', page.toString())
  } else {
    url.searchParams.delete('page')
  }
  // This updates the address bar "silently"
  window.history.replaceState(window.history.state, '', url)
}, 500)
const { model: searchQuery, provider: searchProvider } = useGlobalSearch()
const query = computed(() => searchQuery.value)
// Track if page just loaded (for hiding "Searching..." during view transition)
const hasInteracted = shallowRef(false)
onMounted(() => {
  // Small delay to let view transition complete
  setTimeout(() => {
    hasInteracted.value = true
  }, 300)
})
// Infinite scroll / pagination state
const currentPage = shallowRef(1)
// Get initial page from URL (for scroll restoration on reload)
const initialPage = computed(() => {
  const p = Number.parseInt(normalizeSearchParam(route.query.page), 10)
  return Number.isNaN(p) ? 1 : Math.max(1, p)
})
// Initialize current page from URL on mount
onMounted(() => {
  if (initialPage.value > 1) {
    currentPage.value = initialPage.value
  }
})
// Results to display (directly from incremental search)
const rawVisibleResults = computed(() => results.value)
// Settings for platform package filtering
const { settings } = useSettings()
/**
 * Reorder results to put exact package name match at the top,
 * and optionally filter out platform-specific packages.
 */
const visibleResults = computed(() => {
  const raw = rawVisibleResults.value
  if (!raw) return raw

  let objects = raw.objects

  // Filter out platform-specific packages if setting is enabled
  if (settings.value.hidePlatformPackages) {
    objects = objects.filter(r => !isPlatformSpecificPackage(r.package.name))
  }

  const q = query.value.trim().toLowerCase()
  if (!q) {
    return objects === raw.objects ? raw : { ...raw, objects }
  }

  // Find exact match index
  const exactIdx = objects.findIndex(r => r.package.name.toLowerCase() === q)
  if (exactIdx <= 0) {
    return objects === raw.objects ? raw : { ...raw, objects }
  }

  // Move exact match to top
  const reordered = [...objects]
  const [exactMatch] = reordered.splice(exactIdx, 1)
  if (exactMatch) {
    reordered.unshift(exactMatch)
  }

  return {
    ...raw,
    objects: reordered,
  }
})
// Use structured filters for client-side refinement of search results
const resultsArray = computed(() => visibleResults.value?.objects ?? [])
// All possible non-relevance sort keys
const ALL_SORT_KEYS: SortKey[] = [
  'downloads-week',
  'downloads-day',
  'downloads-month',
  'downloads-year',
  'updated',
  'name',
  'quality',
  'popularity',
  'maintenance',
  'score',
]
// Disable sort keys the current provider can't meaningfully sort by
const disabledSortKeys = computed<SortKey[]>(() => {
  const supported = PROVIDER_SORT_KEYS[searchProvider.value]
  return ALL_SORT_KEYS.filter(k => !supported.has(k))
})
// Minimal structured filters usage for search context (no client-side filtering)
const {
  filters,
  sortOption,
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
} = useStructuredFilters({
  packages: resultsArray,
  initialSort: 'relevance-desc', // Default to search relevance
  searchQueryModel: searchQuery,
})
const isRelevanceSort = computed(
  () => sortOption.value === 'relevance-desc' || sortOption.value === 'relevance-asc',
)
// Maximum eager-load sizes per provider for client-side sorting.
// Algolia supports up to 1000 with offset/length pagination.
// npm supports pagination via `from` parameter (no hard cap, but diminishing relevance).
const EAGER_LOAD_SIZE = { algolia: 500, npm: 500 } as const
// Calculate how many results we need based on current page and preferred page size
const requestedSize = computed(() => {
  const numericPrefSize = preferredPageSize.value === 'all' ? 250 : preferredPageSize.value
  const base = Math.max(pageSize, currentPage.value * numericPrefSize)
  // When sorting by something other than relevance, fetch a large batch
  // so client-side sorting operates on a meaningful pool of matching results
  if (!isRelevanceSort.value) {
    const cap = EAGER_LOAD_SIZE[searchProvider.value]
    return Math.max(base, cap)
  }
  return base
})
// Reset to relevance sort when switching to a provider that doesn't support the current sort key
watch(searchProvider, provider => {
  const { key } = parseSortOption(sortOption.value)
  const supported = PROVIDER_SORT_KEYS[provider]
  if (!supported.has(key)) {
    sortOption.value = 'relevance-desc'
  }
})
// Use incremental search with client-side caching + org/user suggestions
const {
  data: results,
  status,
  isLoadingMore,
  hasMore,
  fetchMore,
  isRateLimited,
  suggestions: validatedSuggestions,
  packageAvailability,
} = useSearch(
  query,
  searchProvider,
  () => ({
    size: requestedSize.value,
  }),
  { suggestions: true },
)
// Client-side sorted results for display
// The search API already handles text filtering, so we only need to sort.
const displayResults = computed(() => {
  if (isRelevanceSort.value) {
    return resultsArray.value
  }

  // Sort the fetched results client-side — neither Algolia nor npm support
  // arbitrary sort orders server-side, so we fetch a large batch and sort here
  const { key, direction } = parseSortOption(sortOption.value)
  const multiplier = direction === 'asc' ? 1 : -1

  return [...resultsArray.value].sort((a, b) => {
    let diff: number
    switch (key) {
      case 'downloads-week':
      case 'downloads-day':
      case 'downloads-month':
      case 'downloads-year':
        diff = (a.downloads?.weekly ?? 0) - (b.downloads?.weekly ?? 0)
        break
      case 'updated':
        diff = new Date(a.package.date).getTime() - new Date(b.package.date).getTime()
        break
      case 'name':
        diff = a.package.name.localeCompare(b.package.name)
        break
      default:
        diff = 0
    }
    return diff * multiplier
  })
})
const resultCount = computed(() => displayResults.value.length)
/**
 * The effective total for display and pagination purposes.
 * When sorting by non-relevance, we're working with a fetched subset (e.g. 250),
 * not the full Algolia total (e.g. 92,324). Show the actual working set size.
 */
const effectiveTotal = computed(() => {
  if (isRelevanceSort.value) {
    return visibleResults.value?.total ?? 0
  }
  // When sorting, the total is the number of results we actually fetched and sorted
  return displayResults.value.length
})
// Handle filter chip removal
function handleClearFilter(chip: FilterChip) {
  clearFilter(chip)
}
// Should we show the loading spinner?
const showSearching = computed(() => {
  // Don't show during initial page load (view transition)
  if (!hasInteracted.value) return false
  // Show if pending and no results yet
  return status.value === 'pending' && displayResults.value.length === 0
})
// Load more when triggered by infinite scroll
async function loadMore() {
  if (isLoadingMore.value || !hasMore.value) return
  // Increase requested size to trigger fetch
  currentPage.value++
  await fetchMore(requestedSize.value)
}
onBeforeUnmount(() => {
  updateUrlPage.cancel()
})
// Update URL when page changes from scrolling
function handlePageChange(page: number) {
  updateUrlPage(page)
}
// Reset page when query changes
watch(query, (newQuery, oldQuery) => {
  if (newQuery.trim() === (oldQuery || '').trim()) return
  currentPage.value = 1
  hasInteracted.value = true
})
// Check if current query could be a valid package name
const isValidPackageName = computed(() => isValidNewPackageName(query.value.trim()))
// Get connector state
const { isConnected, npmUser, listOrgUsers } = useConnector()
// Check if this is a scoped package and extract scope
const packageScope = computed(() => {
  const q = query.value.trim()
  if (!q.startsWith('@')) return null
  const match = q.match(/^@([^/]+)\//)
  return match ? match[1] : null
})
// Track org membership for scoped packages
const orgMembership = ref<Record<string, boolean>>({})
// Check org membership when scope changes
watch(
  [packageScope, isConnected, npmUser],
  async ([scope, connected, user]) => {
    if (!scope || !connected || !user) return
    // Skip if already checked
    if (scope in orgMembership.value) return
    try {
      const users = await listOrgUsers(scope)
      // Check if current user is in the org's user list
      if (users && user in users) {
        orgMembership.value[scope] = true
      } else {
        orgMembership.value[scope] = false
      }
    } catch {
      orgMembership.value[scope] = false
    }
  },
  { immediate: true },
)
// Check if user can publish to scope (either their username or an org they're a member of)
const canPublishToScope = computed(() => {
  const scope = packageScope.value
  if (!scope) return true // Unscoped package
  if (!npmUser.value) return false
  // Can publish if scope matches username
  if (scope.toLowerCase() === npmUser.value.toLowerCase()) return true
  // Can publish if user is a member of the org
  return orgMembership.value[scope] === true
})
// Show claim prompt when valid name, available, either not connected or connected and has permission
const showClaimPrompt = computed(() => {
  return (
    isValidPackageName.value &&
    packageAvailability.value?.available === true &&
    packageAvailability.value.name === query.value.trim() &&
    (!isConnected.value || (isConnected.value && canPublishToScope.value)) &&
    status.value !== 'pending'
  )
})
const claimPackageModalRef = useTemplateRef('claimPackageModalRef')
/** Check if there's an exact package match in results */
const hasExactPackageMatch = computed(() => {
  const q = query.value.trim().toLowerCase()
  if (!q || !visibleResults.value) return false
  return visibleResults.value.objects.some(r => r.package.name.toLowerCase() === q)
})
/** Check if query is an exact org match (e.g., @nuxt matches org nuxt) */
const isExactOrgQuery = computed(() => {
  const q = query.value.trim()
  if (!q.startsWith('@') || q.includes('/')) return false
  const orgName = q.slice(1).toLowerCase()
  return validatedSuggestions.value.some(
    s => s.type === 'org' && s.name.toLowerCase() === orgName && s.exists,
  )
})
/** Determine which item should be highlighted as exact match */
const exactMatchType = computed<'package' | 'org' | 'user' | null>(() => {
  // Package match takes priority
  if (hasExactPackageMatch.value) return 'package'
  // Then org match for @org queries
  if (isExactOrgQuery.value) return 'org'
  // Could extend to user matches for ~user queries
  const q = query.value.trim()
  if (q.startsWith('~')) {
    const userName = q.slice(1).toLowerCase()
    if (
      validatedSuggestions.value.some(
        s => s.type === 'user' && s.name.toLowerCase() === userName && s.exists,
      )
    ) {
      return 'user'
    }
  }
  return null
})
const suggestionCount = computed(() => validatedSuggestions.value.length)
const totalSelectableCount = computed(() => suggestionCount.value + resultCount.value)
/**
 * Get all focusable result elements in DOM order (suggestions first, then packages)
 */
function getFocusableElements(): HTMLElement[] {
  const isVisible = (el: HTMLElement) => el.getClientRects().length > 0
  const suggestions = Array.from(document.querySelectorAll<HTMLElement>('[data-suggestion-index]'))
    .filter(isVisible)
    .sort((a, b) => {
      const aIdx = Number.parseInt(a.dataset.suggestionIndex ?? '0', 10)
      const bIdx = Number.parseInt(b.dataset.suggestionIndex ?? '0', 10)
      return aIdx - bIdx
    })
  const packages = Array.from(document.querySelectorAll<HTMLElement>('[data-result-index]'))
    .filter(isVisible)
    .sort((a, b) => {
      const aIdx = Number.parseInt(a.dataset.resultIndex ?? '0', 10)
      const bIdx = Number.parseInt(b.dataset.resultIndex ?? '0', 10)
      return aIdx - bIdx
    })
  return [...suggestions, ...packages]
}
/**
 * Focus an element and scroll it into view
 */
function focusElement(el: HTMLElement) {
  el.focus()
  el.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
}
// Navigate to package page
async function navigateToPackage(packageName: string) {
  await navigateTo(packageRoute(packageName))
}
// Track the input value when user pressed Enter (for navigating when results arrive)
const pendingEnterQuery = shallowRef<string | null>(null)
// Watch for results to navigate when Enter was pressed before results arrived
watch(displayResults, results => {
  if (!pendingEnterQuery.value) return
  // Check if input is still focused (user hasn't started navigating or clicked elsewhere)
  if (document.activeElement?.tagName !== 'INPUT') {
    pendingEnterQuery.value = null
    return
  }
  // Navigate if first result matches the query that was entered
  const firstResult = results[0]
  // eslint-disable-next-line no-console
  console.log('[search] watcher fired', {
    pending: pendingEnterQuery.value,
    firstResult: firstResult?.package.name,
  })
  if (firstResult?.package.name === pendingEnterQuery.value) {
    pendingEnterQuery.value = null
    navigateToPackage(firstResult.package.name)
  }
})
/**
 * Focus the header search input
 */
function focusSearchInput() {
  const searchInput = document.querySelector<HTMLInputElement>(
    'input[type="search"], input[name="q"]',
  )
  searchInput?.focus()
}
function handleResultsKeydown(e: KeyboardEvent) {
  // If the active element is an input, navigate to exact match or wait for results
  if (e.key === 'Enter' && document.activeElement?.tagName === 'INPUT') {
    // Get value directly from input (not from route query, which may be debounced)
    const inputValue = (document.activeElement as HTMLInputElement).value.trim()
    if (!inputValue) return
    // Check if first result matches the input value exactly
    const firstResult = displayResults.value[0]
    if (firstResult?.package.name === inputValue) {
      pendingEnterQuery.value = null
      return navigateToPackage(firstResult.package.name)
    }
    // No match yet - store input value, watcher will handle navigation when results arrive
    pendingEnterQuery.value = inputValue
    return
  }
  if (totalSelectableCount.value <= 0) return
  const elements = getFocusableElements()
  if (elements.length === 0) return
  const currentIndex = elements.findIndex(el => el === document.activeElement)
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    const nextIndex = currentIndex < 0 ? 0 : Math.min(currentIndex + 1, elements.length - 1)
    const el = elements[nextIndex]
    if (el) focusElement(el)
    return
  }
  if (e.key === 'ArrowUp') {
    e.preventDefault()
    // At first result or no result focused: return focus to search input
    if (currentIndex <= 0) {
      focusSearchInput()
      return
    }
    const nextIndex = currentIndex - 1
    const el = elements[nextIndex]
    if (el) focusElement(el)
    return
  }
  if (e.key === 'Enter') {
    // Browser handles Enter on focused links naturally, but handle for non-link elements
    if (document.activeElement && elements.includes(document.activeElement as HTMLElement)) {
      const el = document.activeElement as HTMLElement
      // Only prevent default and click if it's not already a link (links handle Enter natively)
      if (el.tagName !== 'A') {
        e.preventDefault()
        el.click()
      }
    }
  }
}
onKeyDown(['ArrowDown', 'ArrowUp', 'Enter'], handleResultsKeydown)
useSeoMeta({
  title: () =>
    `${query.value ? $t('search.title_search', { search: query.value }) : $t('search.title_packages')} - npmx`,
  ogTitle: () =>
    `${query.value ? $t('search.title_search', { search: query.value }) : $t('search.title_packages')} - npmx`,
  twitterTitle: () =>
    `${query.value ? $t('search.title_search', { search: query.value }) : $t('search.title_packages')} - npmx`,
  description: () =>
    query.value
      ? $t('search.meta_description', { search: query.value })
      : $t('search.meta_description_packages'),
  ogDescription: () =>
    query.value
      ? $t('search.meta_description', { search: query.value })
      : $t('search.meta_description_packages'),
  twitterDescription: () =>
    query.value
      ? $t('search.meta_description', { search: query.value })
      : $t('search.meta_description_packages'),
})
defineOgImageComponent('Default', {
  title: () =>
    `${query.value ? $t('search.title_search', { search: query.value }) : $t('search.title_packages')} - npmx`,
  description: () =>
    query.value
      ? $t('search.meta_description', { search: query.value })
      : $t('search.meta_description_packages'),
  primaryColor: '#60a5fa',
})

return (_ctx: any,_cache: any) => {
  const _component_SearchProviderToggle = _resolveComponent("SearchProviderToggle")
  const _component_LoadingSpinner = _resolveComponent("LoadingSpinner")
  const _component_SearchSuggestionCard = _resolveComponent("SearchSuggestionCard")
  const _component_PackageListToolbar = _resolveComponent("PackageListToolbar")
  const _component_PackageList = _resolveComponent("PackageList")
  const _component_PaginationControls = _resolveComponent("PaginationControls")
  const _component_PackageClaimPackageModal = _resolveComponent("PackageClaimPackageModal")

  return (_openBlock(), _createElementBlock("main", {
      class: _normalizeClass(["flex-1 py-8 search-page", { 'overflow-x-hidden': _unref(viewMode) !== 'table' }])
    }, [ _createElementVNode("div", { class: "container-sm" }, [ _createElementVNode("div", { class: "flex items-center justify-between gap-4 mb-4" }, [ _createElementVNode("h1", _hoisted_1, _toDisplayString(_ctx.$t('search.title')), 1 /* TEXT */), _createVNode(_component_SearchProviderToggle) ]), (query.value) ? (_openBlock(), _createElementBlock("section", {
            key: 0,
            class: "results-layout"
          }, [ (showSearching.value) ? (_openBlock(), _createBlock(_component_LoadingSpinner, {
                key: 0,
                text: _ctx.$t('search.searching')
              }, null, 8 /* PROPS */, ["text"])) : _createCommentVNode("v-if", true), _withDirectives(_createElementVNode("div", null, [ (_unref(validatedSuggestions).length > 0 && displayResults.value.length > 0) ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: "mb-6 space-y-3"
                }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(validatedSuggestions), (suggestion, idx) => {
                    return (_openBlock(), _createBlock(_component_SearchSuggestionCard, {
                      key: `${suggestion.type}-${suggestion.name}`,
                      type: suggestion.type,
                      name: suggestion.name,
                      index: idx,
                      "is-exact-match": 
                  (exactMatchType.value === 'org' && suggestion.type === 'org') ||
                  (exactMatchType.value === 'user' && suggestion.type === 'user')
              
                    }, null, 8 /* PROPS */, ["type", "name", "index", "is-exact-match"]))
                  }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), (showClaimPrompt.value && visibleResults.value && displayResults.value.length > 0) ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: "mb-6 p-4 bg-bg-subtle border border-border rounded-lg sm:flex hidden flex-row sm:items-center gap-3 sm:gap-4"
                }, [ _createElementVNode("div", { class: "flex-1 min-w-0" }, [ _createElementVNode("p", _hoisted_2, _toDisplayString(_ctx.$t('search.not_taken', { name: query.value })), 1 /* TEXT */), _createElementVNode("p", _hoisted_3, _toDisplayString(_ctx.$t('search.claim_prompt')), 1 /* TEXT */) ]), _createElementVNode("button", {
                    type: "button",
                    class: "shrink-0 px-4 py-2 font-mono text-sm text-bg bg-fg rounded-md motion-safe:transition-colors motion-safe:duration-200 hover:bg-fg/90 focus-visible:outline-accent/70",
                    onClick: _cache[0] || (_cache[0] = ($event: any) => (_unref(claimPackageModalRef)?.open()))
                  }, _toDisplayString(_ctx.$t('search.claim_button', { name: query.value })), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true), (_unref(isRateLimited)) ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  role: "status",
                  class: "py-12"
                }, [ _createElementVNode("p", _hoisted_4, _toDisplayString(_ctx.$t('search.rate_limited')), 1 /* TEXT */) ])) : (visibleResults.value && displayResults.value.length > 0) ? (_openBlock(), _createElementBlock("div", {
                    key: 1,
                    class: "mb-6"
                  }, [ _createVNode(_component_PackageListToolbar, {
                      filters: _unref(filters),
                      columns: _unref(columns),
                      "total-count": effectiveTotal.value,
                      "filtered-count": displayResults.value.length,
                      "available-keywords": _unref(availableKeywords),
                      "active-filters": _unref(activeFilters),
                      "disabled-sort-keys": disabledSortKeys.value,
                      "search-context": "",
                      onToggleColumn: _cache[1] || (_cache[1] = (...args) => (toggleColumn && toggleColumn(...args))),
                      onResetColumns: _cache[2] || (_cache[2] = (...args) => (resetColumns && resetColumns(...args))),
                      onClearFilter: handleClearFilter,
                      onClearAllFilters: _cache[3] || (_cache[3] = (...args) => (clearAllFilters && clearAllFilters(...args))),
                      "onUpdate:text": _cache[4] || (_cache[4] = (...args) => (setTextFilter && setTextFilter(...args))),
                      "onUpdate:search-scope": _cache[5] || (_cache[5] = (...args) => (setSearchScope && setSearchScope(...args))),
                      "onUpdate:download-range": _cache[6] || (_cache[6] = (...args) => (setDownloadRange && setDownloadRange(...args))),
                      "onUpdate:security": _cache[7] || (_cache[7] = (...args) => (setSecurity && setSecurity(...args))),
                      "onUpdate:updated-within": _cache[8] || (_cache[8] = (...args) => (setUpdatedWithin && setUpdatedWithin(...args))),
                      onToggleKeyword: _cache[9] || (_cache[9] = (...args) => (toggleKeyword && toggleKeyword(...args))),
                      "sort-option": _unref(sortOption),
                      "onUpdate:sort-option": _cache[10] || (_cache[10] = ($event: any) => ((sortOption).value = $event)),
                      "view-mode": _unref(viewMode),
                      "onUpdate:view-mode": _cache[11] || (_cache[11] = ($event: any) => ((viewMode).value = $event)),
                      "pagination-mode": _unref(paginationMode),
                      "onUpdate:pagination-mode": _cache[12] || (_cache[12] = ($event: any) => ((paginationMode).value = $event)),
                      "page-size": _unref(preferredPageSize),
                      "onUpdate:page-size": _cache[13] || (_cache[13] = ($event: any) => ((preferredPageSize).value = $event))
                    }, null, 8 /* PROPS */, ["filters", "columns", "total-count", "filtered-count", "available-keywords", "active-filters", "disabled-sort-keys", "sort-option", "view-mode", "pagination-mode", "page-size"]), (_unref(viewMode) === 'cards' && _unref(paginationMode) === 'infinite') ? (_openBlock(), _createElementBlock("p", {
                        key: 0,
                        role: "status",
                        class: "text-fg-muted text-sm mt-4 font-mono"
                      }, [ (isRelevanceSort.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _toDisplayString(_ctx.$t( 'search.found_packages', { count: _ctx.$n(visibleResults.value.total) }, visibleResults.value.total, )) ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ _toDisplayString(_ctx.$t('search.found_packages_sorted', { count: _ctx.$n(effectiveTotal.value) }, effectiveTotal.value)) ], 64 /* STABLE_FRAGMENT */)), (_unref(status) === 'pending') ? (_openBlock(), _createElementBlock("span", {
                            key: 0,
                            class: "text-fg-subtle"
                          }, _toDisplayString(_ctx.$t('search.updating')), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true), (_unref(viewMode) === 'table' || _unref(paginationMode) === 'paginated') ? (_openBlock(), _createElementBlock("p", {
                        key: 0,
                        role: "status",
                        class: "text-fg-muted text-sm mt-4 font-mono"
                      }, _toDisplayString(_ctx.$t( 'filters.count.showing_paginated', {
                      pageSize:
                        _unref(preferredPageSize) === 'all'
                          ? _ctx.$n(effectiveTotal.value)
                          : Math.min(_unref(preferredPageSize), effectiveTotal.value),
                      count: _ctx.$n(effectiveTotal.value),
                    }, effectiveTotal.value, )), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ])) : (_unref(status) === 'success' || _unref(status) === 'error') ? (_openBlock(), _createElementBlock("div", {
                    key: 2,
                    role: "status",
                    class: "py-12"
                  }, [ _createElementVNode("p", _hoisted_5, _toDisplayString(_ctx.$t('search.no_results', { query: query.value })), 1 /* TEXT */), (_unref(validatedSuggestions).length > 0) ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        class: "max-w-md mx-auto mb-6 space-y-3"
                      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(validatedSuggestions), (suggestion, idx) => {
                          return (_openBlock(), _createBlock(_component_SearchSuggestionCard, {
                            key: `${suggestion.type}-${suggestion.name}`,
                            type: suggestion.type,
                            name: suggestion.name,
                            index: idx,
                            "is-exact-match": 
                    (exactMatchType.value === 'org' && suggestion.type === 'org') ||
                    (exactMatchType.value === 'user' && suggestion.type === 'user')
                
                          }, null, 8 /* PROPS */, ["type", "name", "index", "is-exact-match"]))
                        }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), (showClaimPrompt.value) ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        class: "max-w-md mx-auto text-center hidden sm:block"
                      }, [ _createElementVNode("div", { class: "p-4 bg-bg-subtle border border-border rounded-lg" }, [ _createElementVNode("p", _hoisted_6, _toDisplayString(_ctx.$t('search.want_to_claim')), 1 /* TEXT */), _createElementVNode("button", {
                            type: "button",
                            class: "px-4 py-2 font-mono text-sm text-bg bg-fg rounded-md transition-colors duration-200 hover:bg-fg/90 focus-visible:outline-accent/70",
                            onClick: _cache[14] || (_cache[14] = ($event: any) => (_unref(claimPackageModalRef)?.open()))
                          }, _toDisplayString(_ctx.$t('search.claim_button', { name: query.value })), 1 /* TEXT */) ]) ])) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true), _withDirectives(_createVNode(_component_PackageList, {
                results: displayResults.value,
                "search-query": query.value,
                filters: _unref(filters),
                "search-context": "",
                "heading-level": "h2",
                "show-publisher": "",
                "has-more": _unref(hasMore),
                "is-loading": _unref(isLoadingMore),
                "page-size": _unref(preferredPageSize),
                "initial-page": initialPage.value,
                "view-mode": _unref(viewMode),
                columns: _unref(columns),
                "pagination-mode": _unref(paginationMode),
                "current-page": currentPage.value,
                onLoadMore: loadMore,
                onPageChange: handlePageChange,
                onClickKeyword: _cache[15] || (_cache[15] = (...args) => (toggleKeyword && toggleKeyword(...args))),
                "sort-option": _unref(sortOption),
                "onUpdate:sort-option": _cache[16] || (_cache[16] = ($event: any) => ((sortOption).value = $event))
              }, null, 8 /* PROPS */, ["results", "search-query", "filters", "has-more", "is-loading", "page-size", "initial-page", "view-mode", "columns", "pagination-mode", "current-page", "sort-option"]), [ [_vShow, displayResults.value.length > 0 && !_unref(isRateLimited)] ]), (displayResults.value.length > 0 && !_unref(isRateLimited)) ? (_openBlock(), _createBlock(_component_PaginationControls, {
                  key: 0,
                  "total-items": effectiveTotal.value,
                  "view-mode": _unref(viewMode),
                  mode: _unref(paginationMode),
                  "onUpdate:mode": _cache[17] || (_cache[17] = ($event: any) => ((paginationMode).value = $event)),
                  "page-size": _unref(preferredPageSize),
                  "onUpdate:page-size": _cache[18] || (_cache[18] = ($event: any) => ((preferredPageSize).value = $event)),
                  "current-page": currentPage.value,
                  "onUpdate:current-page": _cache[19] || (_cache[19] = ($event: any) => ((currentPage).value = $event))
                }, null, 8 /* PROPS */, ["total-items", "view-mode", "mode", "page-size", "current-page"])) : _createCommentVNode("v-if", true) ], 512 /* NEED_PATCH */), [ [_vShow,  _unref(results) || displayResults.value.length > 0 || _unref(isRateLimited) || _unref(status) === 'error' || _unref(status) === 'success' ] ]) ])) : (_openBlock(), _createElementBlock("section", {
            key: 1,
            class: "py-20 text-center"
          }, [ _createElementVNode("p", _hoisted_7, _toDisplayString(_ctx.$t('search.start_typing')), 1 /* TEXT */) ])) ]), _createVNode(_component_PackageClaimPackageModal, {
        ref_key: "claimPackageModalRef", ref: claimPackageModalRef,
        "package-name": query.value,
        "package-scope": packageScope.value,
        "can-publish-to-scope": canPublishToScope.value
      }, null, 8 /* PROPS */, ["package-name", "package-scope", "can-publish-to-scope"]) ], 2 /* CLASS */))
}
}

})
