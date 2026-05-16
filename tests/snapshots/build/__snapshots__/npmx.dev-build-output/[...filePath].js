import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Teleport as _Teleport, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "/")
const _hoisted_2 = { class: "font-mono text-sm text-fg-muted shrink-0" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle" }, "/")
const _hoisted_4 = { class: "text-fg-muted mb-4" }
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("div", { class: "i-svg-spinners:ring-resize w-8 h-8 mx-auto text-fg-muted" })
const _hoisted_6 = { class: "mt-4 text-fg-muted" }
const _hoisted_7 = { class: "text-fg-muted mb-4" }
const _hoisted_8 = { class: "text-fg-muted", dir: "auto" }
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:arrow-up w-3 h-3" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:external-link w-3 h-3" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("div", { class: "i-lucide:file-text w-12 h-12 mx-auto text-fg-subtle mb-4" })
const _hoisted_12 = { class: "text-fg-muted mb-2" }
const _hoisted_13 = { class: "text-fg-subtle text-sm mb-4" }
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("div", { class: "h-4" })
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("div", { class: "h-4" })
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("div", { class: "h-4" })
const _hoisted_17 = /*#__PURE__*/ _createElementVNode("div", { class: "i-lucide:circle-alert w-8 h-8 mx-auto text-fg-subtle mb-4" })
const _hoisted_18 = { class: "text-fg-muted mb-2" }
const _hoisted_19 = { class: "text-fg-subtle text-sm mb-4" }
import type { PackageFileTree, PackageFileTreeResponse, PackageFileContentResponse } from '#shared/types'

export default /*@__PURE__*/_defineComponent({
  __name: '[...filePath]',
  setup(__props) {

definePageMeta({
  name: 'code',
  path: '/package-code/:org?/:packageName/v/:version/:filePath(.*)?',
  alias: [
    '/package/code/:org?/:packageName/v/:version/:filePath(.*)?',
    '/package/code/:packageName/v/:version/:filePath(.*)?',
    // '/code/@:org?/:packageName/v/:version/:filePath(.*)?',
  ],
  scrollMargin: 160,
})
const route = useRoute('code')
// Parse package name, version, and file path from URL
// Patterns:
//   /code/nuxt/v/4.2.0 → packageName: "nuxt", version: "4.2.0", filePath: null (show tree)
//   /code/nuxt/v/4.2.0/src/index.ts → packageName: "nuxt", version: "4.2.0", filePath: "src/index.ts"
//   /code/@nuxt/kit/v/1.0.0 → packageName: "@nuxt/kit", version: "1.0.0", filePath: null
const parsedRoute = computed(() => {
  const packageName = route.params.org
    ? `${route.params.org}/${route.params.packageName}`
    : route.params.packageName
  const version = route.params.version
  const filePath = route.params.filePath || null

  return { packageName, version, filePath }
})
const packageName = computed(() => parsedRoute.value.packageName)
const version = computed(() => parsedRoute.value.version)
const filePathOrig = computed(() => parsedRoute.value.filePath)
const filePath = computed(() => parsedRoute.value.filePath?.replace(/\/$/, ''))
// Navigation helper - build URL for a path
function getCodeUrl(args: {
  org?: string
  packageName: string
  version: string
  filePath?: string
}): string {
  const base = args.org
    ? `/package-code/${args.org}/${args.packageName}/v/${args.version}`
    : `/package-code/${args.packageName}/v/${args.version}`
  return args.filePath ? `${base}/${args.filePath}` : base
}
// Fetch package data for version list
const { data: pkg } = usePackage(packageName)
// URL pattern for version selector - includes file path if present
const versionUrlPattern = computed(() =>
  getCodeUrl({
    org: route.params.org,
    packageName: route.params.packageName,
    version: '{version}',
    filePath: filePath.value,
  }),
)
// Fetch file tree
const { data: fileTree, status: treeStatus } = useFetch<PackageFileTreeResponse>(
  () => `/api/registry/files/${packageName.value}/v/${version.value}`,
  {
    immediate: !!version.value,
  },
)
// Determine what to show based on the current path
// Note: This needs fileTree to be loaded first
const currentNode = computed(() => {
  if (!fileTree.value?.tree || !filePathOrig.value) return null

  // We use original file path to correctly handle trailing slashes for file tree navigation
  // - /src/index.ts - correct file path
  // - /src/index.ts/ - incorrect file path (but formally can exist as a directory)
  // - /src/index and /src/index/ - correct directory paths
  const parts = filePathOrig.value.split('/')
  let current: PackageFileTree[] | undefined = fileTree.value.tree
  let lastFound: PackageFileTree | null = null
  const partsLength = parts.length

  for (let i = 0; i < partsLength; i++) {
    const part = parts[i]
    const isLast = i === partsLength - 1
    // If the previous part is a directory and the last one is empty (like /lib/) then return the previous directory
    if (!part && isLast && lastFound?.type === 'directory') return lastFound
    const found: PackageFileTree | undefined = current?.find(n => n.name === part)
    if (!found) return null
    lastFound = found
    if (found.type === 'file' && isLast) return found
    current = found.children
  }

  return lastFound
})
const isViewingFile = computed(() => currentNode.value?.type === 'file')
// Maximum file size we'll try to load (500KB) - must match server
const MAX_FILE_SIZE = 500 * 1024
const isFileTooLarge = computed(() => {
  const size = currentNode.value?.size
  return size !== undefined && size > MAX_FILE_SIZE
})
// Fetch file content when a file is selected (and not too large)
const fileContentUrl = computed(() => {
  // Don't fetch if no file path, file tree not loaded, file is too large, or it's a directory
  if (!filePath.value || !fileTree.value || isFileTooLarge.value || !isViewingFile.value) {
    return null
  }
  return `/api/registry/file/${packageName.value}/v/${version.value}/${filePath.value}`
})
const {
  data: fileContent,
  status: fileStatus,
  execute: fetchFileContent,
} = useFetch<PackageFileContentResponse>(() => fileContentUrl.value!, { immediate: false })
watch(
  fileContentUrl,
  url => {
    if (url) fetchFileContent()
  },
  { immediate: true },
)
// Track hash manually since we update it via history API to avoid scroll
const currentHash = shallowRef('')
onMounted(() => {
  currentHash.value = window.location.hash
})
useEventListener('popstate', () => (currentHash.value = window.location.hash))
// Also sync when route changes (e.g., navigating to a different file)
watch(
  () => route.hash,
  hash => {
    currentHash.value = hash
  },
)
// Line number handling from hash
const selectedLines = computed(() => {
  const hash = currentHash.value
  if (!hash) return null

  // Parse #L10 or #L10-L20
  const match = hash.match(/^#L(\d+)(?:-L(\d+))?$/)
  if (!match) return null

  const start = parseInt(match[1] ?? '0', 10)
  const end = match[2] ? parseInt(match[2], 10) : start

  return { start, end }
})
// Scroll to selected line only on initial load or file change (not on click)
const shouldScrollOnHashChange = shallowRef(true)
function scrollToLine() {
  if (!shouldScrollOnHashChange.value) return
  if (!selectedLines.value) return
  const lineEl = document.getElementById(`L${selectedLines.value.start}`)
  if (lineEl) {
    lineEl.scrollIntoView({ behavior: 'smooth', block: 'center' })
  }
}
// Scroll on file content load (initial or file change)
watch(fileContent, () => {
  shouldScrollOnHashChange.value = true
  nextTick(scrollToLine)
})
// Build breadcrumb path segments
const breadcrumbs = computed(() => {
  const parts = filePath.value?.split('/').filter(Boolean) ?? []
  const result: { name: string; path: string }[] = []

  for (let i = 0; i < parts.length; i++) {
    const part = parts[i]
    if (part) {
      result.push({
        name: part,
        path: parts.slice(0, i + 1).join('/'),
      })
    }
  }

  return result
})
// Navigation helper - build URL for a path
function getCurrentCodeUrlWithPath(path?: string): string {
  return getCodeUrl({
    ...route.params,
    filePath: path,
  })
}
// Extract org name from scoped package
const orgName = computed(() => {
  const name = packageName.value
  if (!name.startsWith('@')) return null
  const match = name.match(/^@([^/]+)\//)
  return match ? match[1] : null
})
// Line number click handler - update URL hash without scrolling
function handleLineClick(lineNum: number, event: MouseEvent) {
  let newHash: string
  if (event.shiftKey && selectedLines.value) {
    // Shift+click: select range
    const start = Math.min(selectedLines.value.start, lineNum)
    const end = Math.max(selectedLines.value.end, lineNum)
    newHash = `#L${start}-L${end}`
  } else {
    // Single click: select line
    newHash = `#L${lineNum}`
  }
  // Don't scroll when user clicks - only scroll on initial load
  shouldScrollOnHashChange.value = false
  // Update URL without triggering scroll - use history API directly
  const url = new URL(window.location.href)
  url.hash = newHash
  window.history.replaceState(history.state, '', url.toString())
  // Update our reactive hash tracker
  currentHash.value = newHash
}
// Copy link to current line(s)
const { copied: permalinkCopied, copy: copyPermalink } = useClipboard({ copiedDuring: 2000 })
function copyPermalinkUrl() {
  const url = new URL(window.location.href)
  copyPermalink(url.toString())
}
const { copied: fileContentCopied, copy: copyFileContent } = useClipboard({
  source: () => fileContent.value?.content || '',
  copiedDuring: 2000,
})
// Scroll to top of file content
const contentContainer = useTemplateRef('contentContainer')
function scrollToTop() {
  if (contentContainer.value) {
    contentContainer.value.scrollTo({ top: 0, behavior: 'smooth' })
  }
}
// Canonical URL for this code page
const canonicalUrl = computed(() => `https://npmx.dev${getCodeUrl(route.params)}`)
// Toggle markdown view mode
const markdownViewModes = [
  {
    key: 'preview',
    label: $t('code.markdown_view_mode.preview'),
    icon: 'i-lucide:eye',
  },
  {
    key: 'code',
    label: $t('code.markdown_view_mode.code'),
    icon: 'i-lucide:code',
  },
] as const
const markdownViewMode = shallowRef<(typeof markdownViewModes)[number]['key']>('preview')
const bytesFormatter = useBytesFormatter()
useHead({
  link: [{ rel: 'canonical', href: canonicalUrl }],
})
useSeoMeta({
  title: () => {
    if (filePath.value) {
      return `${filePath.value} - ${packageName.value}@${version.value} - npmx`
    }
    return `Code - ${packageName.value}@${version.value} - npmx`
  },
  ogTitle: () => {
    if (filePath.value) {
      return `${filePath.value} - ${packageName.value}@${version.value} - npmx`
    }
    return `Code - ${packageName.value}@${version.value} - npmx`
  },
  twitterTitle: () => {
    if (filePath.value) {
      return `${filePath.value} - ${packageName.value}@${version.value} - npmx`
    }
    return `Code - ${packageName.value}@${version.value} - npmx`
  },
  description: () => `Browse source code for ${packageName.value}@${version.value}`,
  ogDescription: () => `Browse source code for ${packageName.value}@${version.value}`,
  twitterDescription: () => `Browse source code for ${packageName.value}@${version.value}`,
})
defineOgImageComponent('Default', {
  title: () => `${pkg.value?.name ?? 'Package'} - Code`,
  description: () => pkg.value?.license ?? '',
  primaryColor: '#60a5fa',
})

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_VersionSelector = _resolveComponent("VersionSelector")
  const _component_LinkBase = _resolveComponent("LinkBase")
  const _component_CodeFileTree = _resolveComponent("CodeFileTree")
  const _component_Readme = _resolveComponent("Readme")
  const _component_CodeViewer = _resolveComponent("CodeViewer")
  const _component_SkeletonInline = _resolveComponent("SkeletonInline")
  const _component_SkeletonBlock = _resolveComponent("SkeletonBlock")
  const _component_CodeDirectoryListing = _resolveComponent("CodeDirectoryListing")
  const _component_CodeMobileTreeDrawer = _resolveComponent("CodeMobileTreeDrawer")
  const _component_ClientOnly = _resolveComponent("ClientOnly")

  return (_openBlock(), _createElementBlock("main", { class: "flex-1 flex flex-col" }, [ _createElementVNode("header", { class: "border-b border-border bg-bg sticky top-14 z-20" }, [ _createElementVNode("div", { class: "container py-4" }, [ _createElementVNode("div", { class: "flex items-center gap-2 mb-3 flex-wrap min-w-0" }, [ _createVNode(_component_NuxtLink, {
              to: _ctx.packageRoute(packageName.value, version.value),
              class: "font-mono text-lg font-medium hover:text-fg transition-colors min-w-0 truncate max-w-[60vw] sm:max-w-none",
              title: packageName.value
            }, {
              default: _withCtx(() => [
                (orgName.value)
                  ? (_openBlock(), _createElementBlock("span", {
                    key: 0,
                    class: "text-fg-muted"
                  }, "@" + _toDisplayString(orgName.value) + "/", 1 /* TEXT */))
                  : _createCommentVNode("v-if", true),
                _createTextVNode(_toDisplayString(orgName.value ? packageName.value.replace(`@${orgName.value}/`, '') : packageName.value), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to", "title"]), _createTextVNode("\n          " + "\n          "), (version.value && _unref(pkg)?.versions && _unref(pkg)?.['dist-tags']) ? (_openBlock(), _createBlock(_component_VersionSelector, {
                key: 0,
                "package-name": packageName.value,
                "current-version": version.value,
                versions: _unref(pkg).versions,
                "dist-tags": _unref(pkg)['dist-tags'],
                "url-pattern": versionUrlPattern.value
              }, null, 8 /* PROPS */, ["package-name", "current-version", "versions", "dist-tags", "url-pattern"])) : (version.value) ? (_openBlock(), _createElementBlock("span", {
                  key: 1,
                  class: "px-2 py-0.5 font-mono text-sm bg-bg-muted border border-border rounded truncate max-w-32 sm:max-w-48",
                  title: `v${version.value}`
                }, "\n            v" + _toDisplayString(version.value), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _hoisted_1, _createElementVNode("span", _hoisted_2, _toDisplayString(_ctx.$t('package.links.code')), 1 /* TEXT */) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("nav", {
            "aria-label": _ctx.$t('code.file_path'),
            class: "flex items-center gap-1 font-mono text-sm overflow-x-auto",
            dir: "ltr"
          }, [ (filePath.value) ? (_openBlock(), _createBlock(_component_NuxtLink, {
                key: 0,
                to: getCurrentCodeUrlWithPath(),
                class: "text-fg-muted hover:text-fg transition-colors shrink-0"
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_ctx.$t('code.root')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to"])) : (_openBlock(), _createElementBlock("span", {
                key: 1,
                class: "text-fg shrink-0"
              }, _toDisplayString(_ctx.$t('code.root')), 1 /* TEXT */)), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(breadcrumbs.value, (crumb, i) => {
              return (_openBlock(), _createElementBlock(_Fragment, { key: crumb.path }, [
                _hoisted_3,
                (i < breadcrumbs.value.length - 1)
                  ? (_openBlock(), _createBlock(_component_NuxtLink, {
                    key: 0,
                    to: getCurrentCodeUrlWithPath(crumb.path),
                    class: "text-fg-muted hover:text-fg transition-colors"
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(crumb.name), 1 /* TEXT */)
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 8 /* PROPS */, ["to"]))
                  : (_openBlock(), _createElementBlock("span", {
                    key: 1,
                    class: "text-fg"
                  }, _toDisplayString(crumb.name), 1 /* TEXT */))
              ], 64 /* STABLE_FRAGMENT */))
            }), 128 /* KEYED_FRAGMENT */)) ], 8 /* PROPS */, ["aria-label"]) ]) ]), _createTextVNode("\n\n    " + "\n    "), (!version.value) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "container py-20 text-center"
        }, [ _createElementVNode("p", _hoisted_4, _toDisplayString(_ctx.$t('code.version_required')), 1 /* TEXT */), _createVNode(_component_LinkBase, {
            variant: "button-secondary",
            to: _ctx.packageRoute(packageName.value)
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_ctx.$t('code.go_to_package')), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["to"]) ])) : (_unref(treeStatus) === 'pending') ? (_openBlock(), _createElementBlock("div", {
            key: 1,
            class: "container py-20 text-center"
          }, [ _hoisted_5, _createElementVNode("p", _hoisted_6, _toDisplayString(_ctx.$t('code.loading_tree')), 1 /* TEXT */) ])) : (_unref(treeStatus) === 'error') ? (_openBlock(), _createElementBlock("div", {
            key: 2,
            class: "container py-20 text-center",
            role: "alert"
          }, [ _createElementVNode("p", _hoisted_7, _toDisplayString(_ctx.$t('code.failed_to_load_tree')), 1 /* TEXT */), _createVNode(_component_LinkBase, {
              variant: "button-secondary",
              to: _ctx.packageRoute(packageName.value, version.value)
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('code.back_to_package')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to"]) ])) : (_unref(fileTree)) ? (_openBlock(), _createElementBlock("div", {
            key: 3,
            class: "flex flex-1",
            dir: "ltr"
          }, [ _createElementVNode("aside", { class: "w-64 lg:w-72 border-ie border-border shrink-0 hidden md:block bg-bg-subtle sticky top-28 self-start h-[calc(100vh-7rem)] overflow-y-auto" }, [ _createVNode(_component_CodeFileTree, {
                tree: _unref(fileTree).tree,
                "current-path": filePath.value ?? '',
                "base-url": getCurrentCodeUrlWithPath(),
                "base-route": _unref(route)
              }, null, 8 /* PROPS */, ["tree", "current-path", "base-url", "base-route"]) ]), _createTextVNode("\n\n      "), _createTextVNode("\n      "), _createElementVNode("div", {
              class: "flex-1 min-w-0 overflow-x-hidden sticky top-28 self-start h-[calc(100vh-7rem)] overflow-y-auto",
              ref_key: "contentContainer", ref: contentContainer
            }, [ (isViewingFile.value && _unref(fileContent)) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createElementVNode("div", { class: "sticky z-10 top-0 bg-bg border-b border-border px-4 py-2 flex items-center justify-between" }, [ _createElementVNode("div", { class: "flex items-center gap-2" }, [ (_unref(fileContent).markdownHtml) ? (_openBlock(), _createElementBlock("div", {
                          key: 0,
                          class: "flex items-center gap-1 p-0.5 bg-bg-subtle border border-border-subtle rounded-md overflow-x-auto",
                          role: "tablist",
                          "aria-label": "Markdown view mode selector"
                        }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(markdownViewModes), (mode) => {
                            return (_openBlock(), _createElementBlock("button", {
                              key: mode.key,
                              role: "tab",
                              class: _normalizeClass(["px-2 py-1.5 font-mono text-xs rounded transition-colors duration-150 border border-solid focus-visible:outline-accent/70 inline-flex items-center gap-1.5", 
                      markdownViewMode.value === mode.key
                        ? 'bg-bg shadow text-fg border-border'
                        : 'text-fg-subtle hover:text-fg border-transparent'
                    ]),
                              onClick: _cache[0] || (_cache[0] = ($event: any) => (markdownViewMode.value = mode.key))
                            }, [
                              _createElementVNode("span", {
                                class: _normalizeClass(["inline-block h-3 w-3", mode.icon]),
                                "aria-hidden": "true"
                              }, null, 2 /* CLASS */),
                              _createTextVNode("\n                  " + _toDisplayString(mode.label), 1 /* TEXT */)
                            ], 2 /* CLASS */))
                          }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), _createElementVNode("div", { class: "flex items-center gap-3 text-sm" }, [ _createElementVNode("span", _hoisted_8, _toDisplayString(_ctx.$t('code.lines', { count: _unref(fileContent).lines })), 1 /* TEXT */), (currentNode.value?.size) ? (_openBlock(), _createElementBlock("span", {
                            key: 0,
                            class: "text-fg-subtle"
                          }, _toDisplayString(_unref(bytesFormatter).format(currentNode.value.size)), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]) ]), _createElementVNode("div", { class: "flex items-center gap-2" }, [ _createElementVNode("button", {
                        type: "button",
                        class: "px-2 py-1 font-mono text-xs text-fg-muted bg-bg-subtle border border-border rounded hover:text-fg hover:border-border-hover transition-colors items-center inline-flex gap-1",
                        onClick: scrollToTop
                      }, [ _hoisted_9, _createTextVNode("\n                " + _toDisplayString(_ctx.$t('code.scroll_to_top')), 1 /* TEXT */) ]), (selectedLines.value) ? (_openBlock(), _createElementBlock("button", {
                          key: 0,
                          type: "button",
                          class: "px-2 py-1 font-mono text-xs text-fg-muted bg-bg-subtle border border-border rounded hover:text-fg hover:border-border-hover transition-colors active:scale-95",
                          onClick: copyPermalinkUrl
                        }, _toDisplayString(_unref(permalinkCopied) ? _ctx.$t('common.copied') : _ctx.$t('code.copy_link')), 1 /* TEXT */)) : _createCommentVNode("v-if", true), (!!_unref(fileContent)?.content) ? (_openBlock(), _createElementBlock("button", {
                          key: 0,
                          type: "button",
                          class: "px-2 py-1 font-mono text-xs text-fg-muted bg-bg-subtle border border-border rounded hover:text-fg hover:border-border-hover transition-colors inline-flex items-center gap-1 capitalize",
                          onClick: _cache[1] || (_cache[1] = ($event: any) => (_unref(copyFileContent)()))
                        }, [ _createElementVNode("span", {
                            class: _normalizeClass(["w-3 h-3", _unref(fileContentCopied) ? 'i-lucide:check' : 'i-lucide:file'])
                          }, null, 2 /* CLASS */), _createTextVNode("\n                "), _toDisplayString(_unref(fileContentCopied) ? _ctx.$t('common.copied') : _ctx.$t('common.copy')) ])) : _createCommentVNode("v-if", true), _createElementVNode("a", {
                        href: `https://cdn.jsdelivr.net/npm/${packageName.value}@${version.value}/${filePath.value}`,
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "px-2 py-1 font-mono text-xs text-fg-muted bg-bg-subtle border border-border rounded hover:text-fg hover:border-border-hover transition-colors inline-flex items-center gap-1"
                      }, [ _createTextVNode(_toDisplayString(_ctx.$t('code.raw')) + "\n                ", 1 /* TEXT */), _hoisted_10 ], 8 /* PROPS */, ["href"]) ]) ]), (_unref(fileContent).markdownHtml) ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "flex justify-center p-4"
                    }, [ _createVNode(_component_Readme, { html: _unref(fileContent).markdownHtml.html }, null, 8 /* PROPS */, ["html"]) ])) : _createCommentVNode("v-if", true), _withDirectives(_createVNode(_component_CodeViewer, {
                    html: _unref(fileContent).html,
                    lines: _unref(fileContent).lines,
                    "selected-lines": selectedLines.value,
                    onLineClick: handleLineClick
                  }, null, 8 /* PROPS */, ["html", "lines", "selected-lines"]), [ [_vShow, !_unref(fileContent).markdownHtml || markdownViewMode.value === 'code'] ]) ], 64 /* STABLE_FRAGMENT */)) : (isViewingFile.value && isFileTooLarge.value) ? (_openBlock(), _createElementBlock("div", {
                    key: 1,
                    class: "py-20 text-center"
                  }, [ _hoisted_11, _createElementVNode("p", _hoisted_12, _toDisplayString(_ctx.$t('code.file_too_large')), 1 /* TEXT */), _createElementVNode("p", _hoisted_13, _toDisplayString(_ctx.$t('code.file_size_warning', { size: _unref(bytesFormatter).format(currentNode.value?.size ?? 0) })), 1 /* TEXT */), _createVNode(_component_LinkBase, {
                      variant: "button-secondary",
                      to: `https://cdn.jsdelivr.net/npm/${packageName.value}@${version.value}/${filePath.value}`
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_ctx.$t('code.view_raw')), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["to"]) ])) : (filePath.value && _unref(fileStatus) === 'pending') ? (_openBlock(), _createElementBlock("div", {
                    key: 2,
                    class: "flex min-h-full",
                    "aria-busy": "true",
                    "aria-label": _ctx.$t('common.loading')
                  }, [ _createElementVNode("div", { class: "shrink-0 bg-bg-subtle border-ie border-border w-14 py-0" }, [ (_openBlock(), _createElementBlock(_Fragment, null, _renderList(20, (n) => {
                        return _createElementVNode("div", {
                          key: n,
                          class: "px-3 h-6 flex items-center justify-end"
                        }, [
                          _createVNode(_component_SkeletonInline, { class: "w-4 h-3 rounded-sm" })
                        ])
                      }), 64 /* STABLE_FRAGMENT */)) ]), _createTextVNode("\n          "), _createTextVNode("\n          "), _createElementVNode("div", { class: "flex-1 p-4 space-y-1.5" }, [ _createVNode(_component_SkeletonBlock, { class: "h-4 w-32 rounded-sm" }), _createVNode(_component_SkeletonBlock, { class: "h-4 w-48 rounded-sm" }), _createVNode(_component_SkeletonBlock, { class: "h-4 w-24 rounded-sm" }), _hoisted_14, _createVNode(_component_SkeletonBlock, { class: "h-4 w-64 rounded-sm" }), _createVNode(_component_SkeletonBlock, { class: "h-4 w-56 rounded-sm" }), _createVNode(_component_SkeletonBlock, { class: "h-4 w-40 rounded-sm" }), _createVNode(_component_SkeletonBlock, { class: "h-4 w-72 rounded-sm" }), _hoisted_15, _createVNode(_component_SkeletonBlock, { class: "h-4 w-36 rounded-sm" }), _createVNode(_component_SkeletonBlock, { class: "h-4 w-52 rounded-sm" }), _createVNode(_component_SkeletonBlock, { class: "h-4 w-44 rounded-sm" }), _createVNode(_component_SkeletonBlock, { class: "h-4 w-28 rounded-sm" }), _hoisted_16, _createVNode(_component_SkeletonBlock, { class: "h-4 w-60 rounded-sm" }), _createVNode(_component_SkeletonBlock, { class: "h-4 w-48 rounded-sm" }), _createVNode(_component_SkeletonBlock, { class: "h-4 w-32 rounded-sm" }), _createVNode(_component_SkeletonBlock, { class: "h-4 w-56 rounded-sm" }), _createVNode(_component_SkeletonBlock, { class: "h-4 w-40 rounded-sm" }), _createVNode(_component_SkeletonBlock, { class: "h-4 w-24 rounded-sm" }) ]) ])) : (filePath.value && _unref(fileStatus) === 'error') ? (_openBlock(), _createElementBlock("div", {
                    key: 3,
                    class: "py-20 text-center",
                    role: "alert"
                  }, [ _hoisted_17, _createElementVNode("p", _hoisted_18, _toDisplayString(_ctx.$t('code.failed_to_load')), 1 /* TEXT */), _createElementVNode("p", _hoisted_19, _toDisplayString(_ctx.$t('code.unavailable_hint')), 1 /* TEXT */), _createVNode(_component_LinkBase, {
                      variant: "button-secondary",
                      to: `https://cdn.jsdelivr.net/npm/${packageName.value}@${version.value}/${filePath.value}`
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_ctx.$t('code.view_raw')), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["to"]) ])) : (_openBlock(), _createBlock(_component_CodeDirectoryListing, {
                  key: 4,
                  tree: _unref(fileTree).tree,
                  "current-path": filePath.value ?? '',
                  "base-url": getCurrentCodeUrlWithPath(),
                  "base-route": _unref(route)
                }, null, 8 /* PROPS */, ["tree", "current-path", "base-url", "base-route"])), _createTextVNode("\n\n        " + "\n        " + "\n\n        " + "\n        " + "\n\n        " + "\n        " + "\n\n        " + "\n        ") ], 512 /* NEED_PATCH */) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    " + "\n    " + "\n\n    " + "\n    " + "\n\n    " + "\n    " + "\n\n    " + "\n    "), _createVNode(_component_ClientOnly, null, {
        default: _withCtx(() => [
          _createVNode(_Teleport, { to: "body" }, [
            (_unref(fileTree))
              ? (_openBlock(), _createBlock(_component_CodeMobileTreeDrawer, {
                key: 0,
                tree: _unref(fileTree).tree,
                "current-path": filePath.value ?? '',
                "base-url": getCurrentCodeUrlWithPath(),
                "base-route": _unref(route)
              }, null, 8 /* PROPS */, ["tree", "current-path", "base-url", "base-route"]))
              : _createCommentVNode("v-if", true)
          ])
        ]),
        _: 1 /* STABLE */
      }) ]))
}
}

})
