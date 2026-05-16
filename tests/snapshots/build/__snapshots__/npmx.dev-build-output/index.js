import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "font-mono text-2xl sm:text-3xl font-medium" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "i-simple-icons:npm w-4 h-4", "aria-hidden": "true" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:chart-line w-3.5 h-3.5", "aria-hidden": "true" })
const _hoisted_4 = { class: "font-mono" }
const _hoisted_5 = { class: "text-fg-muted mb-4" }
const _hoisted_6 = { class: "text-xs text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_7 = { class: "text-fg" }
const _hoisted_8 = { class: "text-fg-subtle text-sm mt-2" }
import { debounce } from 'perfect-debounce'
import { normalizeSearchParam } from '#shared/utils/url'

type SortOption = 'downloads' | 'updated' | 'name-asc' | 'name-desc'

export default /*@__PURE__*/_defineComponent({
  __name: 'index',
  setup(__props) {

const route = useRoute('~username')
const router = useRouter()
const username = computed(() => route.params.username.toLowerCase())
// Debounced URL update for page and filter/sort
const updateUrl = debounce((updates: { page?: number; filter?: string; sort?: string }) => {
  router.replace({
    query: {
      ...route.query,
      page: updates.page && updates.page > 1 ? updates.page : undefined,
      q: updates.filter || undefined,
      sort: updates.sort && updates.sort !== 'downloads' ? updates.sort : undefined,
    },
  })
}, 300)
// Filter and sort state (from URL)
const filterText = shallowRef(normalizeSearchParam(route.query.q))
const sortOption = shallowRef<SortOption>(
  (normalizeSearchParam(route.query.sort) as SortOption) || 'downloads',
)
// Update URL when filter/sort changes (debounced)
const debouncedUpdateUrl = debounce((filter: string, sort: string) => {
  updateUrl({ filter, sort })
}, 300)
// Load all results when user starts filtering/sorting (so client-side filter works on full set)
watch([filterText, sortOption], ([filter, sort]) => {
  if (filter !== '' || sort !== 'downloads') {
    loadAll()
  }
  debouncedUpdateUrl(filter, sort)
})
// Fetch packages (composable manages pagination & provider dispatch internally)
const {
  data: results,
  status,
  error,
  isLoadingMore,
  hasMore,
  loadMore,
  loadAll,
  pageSize,
} = useUserPackages(username)
// Get initial page from URL (for scroll restoration on reload)
const initialPage = computed(() => {
  const p = Number.parseInt(normalizeSearchParam(route.query.page), 10)
  return Number.isNaN(p) ? 1 : Math.max(1, p)
})
// Get the base packages list
const packages = computed(() => results.value?.objects ?? [])
const packageCount = computed(() => packages.value.length)
// Apply client-side filter and sort
const filteredAndSortedPackages = computed(() => {
  let pkgs = [...packages.value]

  // Apply text filter
  if (filterText.value) {
    const search = filterText.value.toLowerCase()
    pkgs = pkgs.filter(
      pkg =>
        pkg.package.name.toLowerCase().includes(search) ||
        pkg.package.description?.toLowerCase().includes(search),
    )
  }

  // Apply sort
  switch (sortOption.value) {
    case 'updated':
      pkgs.sort((a, b) => {
        const dateA = a.package.date || ''
        const dateB = b.package.date || ''
        return dateB.localeCompare(dateA)
      })
      break
    case 'name-asc':
      pkgs.sort((a, b) => a.package.name.localeCompare(b.package.name))
      break
    case 'name-desc':
      pkgs.sort((a, b) => b.package.name.localeCompare(a.package.name))
      break
    case 'downloads':
    default:
      pkgs.sort((a, b) => (b.downloads?.weekly ?? 0) - (a.downloads?.weekly ?? 0))
      break
  }

  return pkgs
})
const filteredCount = computed(() => filteredAndSortedPackages.value.length)
// Total weekly downloads across displayed packages (updates with filter)
const totalWeeklyDownloads = computed(() =>
  filteredAndSortedPackages.value.reduce((sum, pkg) => sum + (pkg.downloads?.weekly ?? 0), 0),
)
// Update URL when page changes from scrolling
function handlePageChange(page: number) {
  updateUrl({ page, filter: filterText.value, sort: sortOption.value })
}
// Reset state when username changes
watch(username, () => {
  filterText.value = ''
  sortOption.value = 'downloads'
})
useSeoMeta({
  title: () => `~${username.value} - npmx`,
  ogTitle: () => `~${username.value} - npmx`,
  twitterTitle: () => `~${username.value} - npmx`,
  description: () => `npm packages maintained by ${username.value}`,
  ogDescription: () => `npm packages maintained by ${username.value}`,
  twitterDescription: () => `npm packages maintained by ${username.value}`,
})
defineOgImageComponent('Default', {
  title: () => `~${username.value}`,
  description: () => (results.value ? `${results.value.total} packages` : 'npm user profile'),
  primaryColor: '#60a5fa',
})

return (_ctx: any,_cache: any) => {
  const _component_UserAvatar = _resolveComponent("UserAvatar")
  const _component_LoadingSpinner = _resolveComponent("LoadingSpinner")
  const _component_LinkBase = _resolveComponent("LinkBase")
  const _component_PackageListControls = _resolveComponent("PackageListControls")
  const _component_PackageList = _resolveComponent("PackageList")

  return (_openBlock(), _createElementBlock("main", { class: "container flex-1 flex flex-col py-8 sm:py-12 w-full" }, [ _createElementVNode("header", { class: "mb-8 pb-8 border-b border-border" }, [ _createElementVNode("div", { class: "flex flex-wrap items-center gap-4" }, [ _createVNode(_component_UserAvatar, { username: username.value }, null, 8 /* PROPS */, ["username"]), _createElementVNode("div", null, [ _createElementVNode("h1", _hoisted_1, "~" + _toDisplayString(username.value), 1 /* TEXT */), (_unref(results)?.total) ? (_openBlock(), _createElementBlock("p", {
                key: 0,
                class: "text-fg-muted text-sm mt-1"
              }, _toDisplayString(_ctx.$t('org.public_packages', { count: _ctx.$n(_unref(results).total) }, _unref(results).total)), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", { class: "ms-auto text-end" }, [ _createElementVNode("nav", { "aria-label": "External links" }, [ _createElementVNode("a", {
                href: `https://www.npmjs.com/~${username.value}`,
                target: "_blank",
                rel: "noopener noreferrer",
                class: "link-subtle font-mono text-sm inline-flex items-center gap-1.5",
                title: _ctx.$t('common.view_on_npm')
              }, [ _hoisted_2, _createTextVNode("\n              npm\n            ") ], 8 /* PROPS */, ["href", "title"]) ]), _createElementVNode("p", {
              class: "text-fg-subtle text-xs mt-1 flex items-center gap-1.5 justify-end cursor-help",
              title: _ctx.$t('common.vanity_downloads_hint', { count: filteredCount.value }, filteredCount.value)
            }, [ _hoisted_3, _createElementVNode("span", _hoisted_4, _toDisplayString(_ctx.$n(totalWeeklyDownloads.value)) + " " + _toDisplayString(_ctx.$t('common.per_week')), 1 /* TEXT */) ], 8 /* PROPS */, ["title"]) ]) ]) ]), _createTextVNode("\n\n    " + "\n    "), (_unref(status) === 'pending' && packages.value.length === 0 && !_unref(error)) ? (_openBlock(), _createBlock(_component_LoadingSpinner, {
          key: 0,
          text: _ctx.$t('common.loading_packages')
        }, null, 8 /* PROPS */, ["text"])) : (_unref(error) || _unref(status) === 'error') ? (_openBlock(), _createElementBlock("div", {
            key: 1,
            role: "alert",
            class: "py-12 text-center"
          }, [ _createElementVNode("p", _hoisted_5, _toDisplayString(_unref(error)?.message ?? _ctx.$t('user.page.failed_to_load')), 1 /* TEXT */), _createVNode(_component_LinkBase, {
              variant: "button-secondary",
              to: { name: 'index' }
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('common.go_back_home')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to"]) ])) : (packages.value.length > 0) ? (_openBlock(), _createElementBlock("section", { key: 2 }, [ _createElementVNode("h2", _hoisted_6, _toDisplayString(_ctx.$t('user.page.packages_title')), 1 /* TEXT */), _createTextVNode("\n\n      "), _createTextVNode("\n      "), _createVNode(_component_PackageListControls, {
              placeholder: _ctx.$t('user.page.filter_placeholder', { count: _unref(results)?.total ?? 0 }),
              "total-count": packageCount.value,
              "filtered-count": filteredCount.value,
              filter: filterText.value,
              "onUpdate:filter": _cache[0] || (_cache[0] = ($event: any) => ((filterText).value = $event)),
              sort: sortOption.value,
              "onUpdate:sort": _cache[1] || (_cache[1] = ($event: any) => ((sortOption).value = $event))
            }, null, 8 /* PROPS */, ["placeholder", "total-count", "filtered-count", "filter", "sort"]), _createTextVNode("\n\n      "), _createTextVNode("\n      "), (filteredAndSortedPackages.value.length === 0) ? (_openBlock(), _createElementBlock("p", {
                key: 0,
                class: "text-fg-muted py-8 text-center font-mono"
              }, _toDisplayString(_ctx.$t('user.page.no_match', { query: filterText.value })), 1 /* TEXT */)) : (_openBlock(), _createBlock(_component_PackageList, {
                key: 1,
                results: filteredAndSortedPackages.value,
                "has-more": _unref(hasMore),
                "is-loading": _unref(isLoadingMore),
                "page-size": _unref(pageSize),
                "initial-page": initialPage.value,
                onLoadMore: _cache[2] || (_cache[2] = (...args) => (loadMore && loadMore(...args))),
                onPageChange: handlePageChange
              }, null, 8 /* PROPS */, ["results", "has-more", "is-loading", "page-size", "initial-page"])) ])) : (_unref(status) === 'success') ? (_openBlock(), _createElementBlock("div", {
            key: 3,
            class: "flex-1 flex items-center justify-center"
          }, [ _createElementVNode("div", { class: "text-center" }, [ _createElementVNode("p", { class: "text-fg-muted font-mono" }, [ _createTextVNode(_toDisplayString(_ctx.$t('user.page.no_packages')) + " ", 1 /* TEXT */), _createElementVNode("span", _hoisted_7, "~" + _toDisplayString(username.value), 1 /* TEXT */) ]), _createElementVNode("p", _hoisted_8, _toDisplayString(_ctx.$t('user.page.no_packages_hint')), 1 /* TEXT */) ]) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    " + "\n    " + "\n\n    " + "\n    " + "\n\n    " + "\n    ") ]))
}
}

})
