import { defineComponent as _defineComponent } from 'vue'
import { Teleport as _Teleport, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle" }, "/")
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "font-mono text-sm text-fg-muted" }, "compare")
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "text-xs text-fg-subtle uppercase tracking-wide" }, "From")
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:arrow-right w-4 h-4 text-fg-subtle" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("span", { class: "text-xs text-fg-subtle uppercase tracking-wide" }, "To")
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("p", { class: "text-fg-muted mb-4" }, "\n        Invalid comparison URL. Use format: /diff/package/v/from...to\n      ")
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("div", { class: "i-svg-spinners-ring-resize w-8 h-8 mx-auto text-fg-muted" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("p", { class: "mt-4 text-fg-muted" }, "Comparing versions...")
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("p", { class: "text-fg-muted mb-4" }, "Failed to compare versions")
const _hoisted_10 = { class: "text-green-500" }
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle" }, "/")
const _hoisted_12 = { class: "text-red-500" }
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle" }, "/")
const _hoisted_14 = { class: "text-yellow-500" }
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle" }, "•")
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:file-text w-3.5 h-3.5" })
const _hoisted_17 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:file-text w-16 h-16 mx-auto text-fg-subtle/50 block mb-4" })
const _hoisted_18 = { class: "text-fg-muted" }
import type { CompareResponse, FileChange } from '#shared/types'
import { diffRoute, packageRoute } from '~/utils/router'

export default /*@__PURE__*/_defineComponent({
  __name: '[versionRange]',
  setup(__props) {

definePageMeta({
  name: 'diff',
  path: '/diff/:org?/:packageName/v/:versionRange',
  alias: ['/diff/:packageName/v/:versionRange'],
})
const route = useRoute('diff')
// Derive package name from typed route params
// /diff/nuxt/v/4.0.0...4.2.0           → org: undefined, packageName: "nuxt"
// /diff/@nuxt/kit/v/1.0.0...2.0.0      → org: "@nuxt", packageName: "kit"
const packageName = computed(() =>
  route.params.org ? `${route.params.org}/${route.params.packageName}` : route.params.packageName,
)
// Parse version range from the typed param (from...to)
const versionRange = computed(() => {
  const parts = route.params.versionRange.split('...')
  if (parts.length !== 2) return null
  return { from: parts[0]!, to: parts[1]! }
})
const fromVersion = computed(() => versionRange.value?.from ?? '')
const toVersion = computed(() => versionRange.value?.to ?? '')
const router = useRouter()
const { data: pkg } = usePackage(packageName)
const { data: compare, status: compareStatus } = useFetch<CompareResponse>(
  () => `/api/registry/compare/${packageName.value}/v/${fromVersion.value}...${toVersion.value}`,
  {
    immediate: !!versionRange.value,
    timeout: 15000,
  },
)
const manualSelection = ref<FileChange | null>(null)
const fileFilter = ref<'all' | 'added' | 'removed' | 'modified'>('all')
const mobileDrawerOpen = ref(false)
const allChanges = computed(() => {
  if (!compare.value) return []
  return [
    ...compare.value.files.added,
    ...compare.value.files.removed,
    ...compare.value.files.modified,
  ].sort((a, b) => a.path.localeCompare(b.path))
})
// Derive selected file: manual selection takes priority, then ?file= query param.
// Using a computed ensures the query-param file is resolved during SSR without
// needing a watcher (which may not re-run before SSR rendering completes).
const selectedFile = computed<FileChange | null>({
  get: () => {
    if (manualSelection.value) return manualSelection.value
    const filePath = route.query.file
    if (!filePath || !compare.value) return null
    return allChanges.value.find(f => f.path === filePath) ?? null
  },
  set: file => {
    manualSelection.value = file
  },
})
if (import.meta.client) {
  watch(
    selectedFile,
    file => {
      const query = { ...route.query }
      if (file?.path) query.file = file.path
      else delete query.file
      router.replace({ query })
    },
    { deep: false },
  )
}
const groupedDeps = computed(() => {
  if (!compare.value?.dependencyChanges) return new Map()

  const groups = new Map<string, typeof compare.value.dependencyChanges>()
  for (const change of compare.value.dependencyChanges) {
    const existing = groups.get(change.section) ?? []
    existing.push(change)
    groups.set(change.section, existing)
  }
  return groups
})
const fromVersionUrlPattern = computed(() => {
  return router.resolve(diffRoute(packageName.value, '{version}', toVersion.value)).href
})
const toVersionUrlPattern = computed(() => {
  return router.resolve(diffRoute(packageName.value, fromVersion.value, '{version}')).href
})
useSeoMeta({
  title: () => {
    if (fromVersion.value && toVersion.value) {
      return `Compare ${packageName.value} ${fromVersion.value}...${toVersion.value} - npmx`
    }
    return `Compare - ${packageName.value} - npmx`
  },
  description: () =>
    `Compare changes between ${packageName.value} versions ${fromVersion.value} and ${toVersion.value}`,
})

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_VersionSelector = _resolveComponent("VersionSelector")
  const _component_DiffSidebarPanel = _resolveComponent("DiffSidebarPanel")
  const _component_DiffViewerPanel = _resolveComponent("DiffViewerPanel")
  const _component_DiffMobileSidebarDrawer = _resolveComponent("DiffMobileSidebarDrawer")
  const _component_ClientOnly = _resolveComponent("ClientOnly")

  return (_openBlock(), _createElementBlock("main", { class: "flex-1 flex flex-col min-h-0" }, [ _createElementVNode("header", { class: "border-b border-border bg-bg sticky top-14 z-20" }, [ _createElementVNode("div", { class: "container py-4" }, [ _createElementVNode("div", { class: "flex items-center gap-2 mb-3 flex-wrap min-w-0" }, [ _createVNode(_component_NuxtLink, {
              to: _unref(packageRoute)(packageName.value),
              class: "font-mono text-lg font-medium hover:text-fg transition-colors min-w-0 truncate"
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(packageName.value), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to"]), _hoisted_1, _hoisted_2 ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", { class: "flex items-center gap-3 flex-wrap" }, [ _createElementVNode("div", { class: "flex items-center gap-2" }, [ _hoisted_3, (_unref(pkg)?.versions && _unref(pkg)?.['dist-tags']) ? (_openBlock(), _createBlock(_component_VersionSelector, {
                  key: 0,
                  "package-name": packageName.value,
                  "current-version": fromVersion.value,
                  versions: _unref(pkg).versions,
                  "dist-tags": _unref(pkg)['dist-tags'],
                  "url-pattern": fromVersionUrlPattern.value
                }, null, 8 /* PROPS */, ["package-name", "current-version", "versions", "dist-tags", "url-pattern"])) : (_openBlock(), _createElementBlock("span", {
                  key: 1,
                  class: "font-mono text-sm text-fg-muted"
                }, _toDisplayString(fromVersion.value), 1 /* TEXT */)) ]), _hoisted_4, _createElementVNode("div", { class: "flex items-center gap-2" }, [ _hoisted_5, (_unref(pkg)?.versions && _unref(pkg)?.['dist-tags']) ? (_openBlock(), _createBlock(_component_VersionSelector, {
                  key: 0,
                  "package-name": packageName.value,
                  "current-version": toVersion.value,
                  versions: _unref(pkg).versions,
                  "dist-tags": _unref(pkg)['dist-tags'],
                  "url-pattern": toVersionUrlPattern.value
                }, null, 8 /* PROPS */, ["package-name", "current-version", "versions", "dist-tags", "url-pattern"])) : (_openBlock(), _createElementBlock("span", {
                  key: 1,
                  class: "font-mono text-sm text-fg-muted"
                }, _toDisplayString(toVersion.value), 1 /* TEXT */)) ]) ]) ]) ]), _createTextVNode("\n\n    " + "\n    "), (!versionRange.value) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "container py-20 text-center"
        }, [ _hoisted_6, _createVNode(_component_NuxtLink, {
            to: _unref(packageRoute)(packageName.value),
            class: "btn"
          }, {
            default: _withCtx(() => [
              _createTextVNode("Go to package")
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["to"]) ])) : (_unref(compareStatus) === 'pending') ? (_openBlock(), _createElementBlock("div", {
            key: 1,
            class: "container py-20 text-center"
          }, [ _hoisted_7, _hoisted_8 ])) : (_unref(compareStatus) === 'error') ? (_openBlock(), _createElementBlock("div", {
            key: 2,
            class: "container py-20 text-center",
            role: "alert"
          }, [ _hoisted_9, _createVNode(_component_NuxtLink, {
              to: _unref(packageRoute)(packageName.value),
              class: "btn"
            }, {
              default: _withCtx(() => [
                _createTextVNode("Back to package")
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to"]) ])) : (_unref(compare)) ? (_openBlock(), _createElementBlock("div", {
            key: 3,
            class: "flex-1 flex flex-col md:flex-row min-h-0 overflow-hidden"
          }, [ _createElementVNode("aside", { class: "hidden md:flex w-80 border-ie border-border bg-bg-subtle flex-col shrink-0 min-h-0" }, [ _createVNode(_component_DiffSidebarPanel, {
                compare: _unref(compare),
                "grouped-deps": groupedDeps.value,
                "all-changes": allChanges.value,
                onFileSelect: _cache[0] || (_cache[0] = ($event: any) => (selectedFile.value = $event)),
                "selected-file": selectedFile.value,
                "onUpdate:selected-file": _cache[1] || (_cache[1] = ($event: any) => ((selectedFile).value = $event)),
                "file-filter": fileFilter.value,
                "onUpdate:file-filter": _cache[2] || (_cache[2] = ($event: any) => ((fileFilter).value = $event))
              }, null, 8 /* PROPS */, ["compare", "grouped-deps", "all-changes", "selected-file", "file-filter"]) ]), _createTextVNode("\n\n      "), _createTextVNode("\n      "), _createElementVNode("div", { class: "flex-1 flex flex-col min-w-0 overflow-hidden" }, [ _createElementVNode("div", { class: "md:hidden border-b border-border bg-bg-subtle px-4 py-3 flex items-center justify-between gap-3" }, [ _createElementVNode("div", { class: "flex items-center gap-2 text-2xs font-mono text-fg-muted" }, [ _createElementVNode("span", { class: "flex items-center gap-1" }, [ _createElementVNode("span", _hoisted_10, "+" + _toDisplayString(_unref(compare).stats.filesAdded), 1 /* TEXT */), _hoisted_11, _createElementVNode("span", _hoisted_12, "-" + _toDisplayString(_unref(compare).stats.filesRemoved), 1 /* TEXT */), _hoisted_13, _createElementVNode("span", _hoisted_14, "~" + _toDisplayString(_unref(compare).stats.filesModified), 1 /* TEXT */) ]), _hoisted_15, _createElementVNode("span", null, _toDisplayString(_ctx.$t('compare.files_count', { count: allChanges.value.length })), 1 /* TEXT */) ]), _createElementVNode("button", {
                  type: "button",
                  class: "px-2 py-1 inline-flex items-center gap-1.5 font-mono text-xs bg-bg-muted border border-border rounded text-fg-muted hover:text-fg hover:border-border-hover transition-colors",
                  onClick: _cache[3] || (_cache[3] = ($event: any) => (mobileDrawerOpen.value = true))
                }, [ _hoisted_16, _createTextVNode("\n            " + _toDisplayString(_ctx.$t('compare.files_button')), 1 /* TEXT */) ]) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", { class: "flex-1 overflow-hidden bg-bg-subtle" }, [ (selectedFile.value) ? (_openBlock(), _createBlock(_component_DiffViewerPanel, {
                    key: 0,
                    "package-name": packageName.value,
                    "from-version": fromVersion.value,
                    "to-version": toVersion.value,
                    file: selectedFile.value
                  }, null, 8 /* PROPS */, ["package-name", "from-version", "to-version", "file"])) : (_openBlock(), _createElementBlock("div", {
                    key: 1,
                    class: "h-full flex items-center justify-center text-center p-8"
                  }, [ _createElementVNode("div", null, [ _hoisted_17, _createElementVNode("p", _hoisted_18, _toDisplayString(_ctx.$t('compare.select_file_prompt')), 1 /* TEXT */) ]) ])) ]) ]) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    " + "\n    " + "\n\n    " + "\n    " + "\n\n    " + "\n    " + "\n\n    " + "\n    "), _createVNode(_component_ClientOnly, null, {
        default: _withCtx(() => [
          _createVNode(_Teleport, { to: "body" }, [
            (_unref(compare))
              ? (_openBlock(), _createBlock(_component_DiffMobileSidebarDrawer, {
                key: 0,
                compare: _unref(compare),
                "grouped-deps": groupedDeps.value,
                "all-changes": allChanges.value,
                "selected-file": selectedFile.value,
                "onUpdate:selected-file": _cache[4] || (_cache[4] = ($event: any) => ((selectedFile).value = $event)),
                "file-filter": fileFilter.value,
                "onUpdate:file-filter": _cache[5] || (_cache[5] = ($event: any) => ((fileFilter).value = $event)),
                open: mobileDrawerOpen.value,
                "onUpdate:open": _cache[6] || (_cache[6] = ($event: any) => ((mobileDrawerOpen).value = $event))
              }, null, 8 /* PROPS */, ["compare", "grouped-deps", "all-changes", "selected-file", "file-filter", "open"]))
              : _createCommentVNode("v-if", true)
          ])
        ]),
        _: 1 /* STABLE */
      }) ]))
}
}

})
