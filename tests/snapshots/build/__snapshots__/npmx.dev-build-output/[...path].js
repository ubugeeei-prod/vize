import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "sr-only" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { class: "absolute inset-0 bg-bg/90 backdrop-blur" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "text-xs px-2 py-1 rounded badge-green border border-badge-green/50" }, "\n              API Docs\n            ")
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("h2", { class: "text-xs font-semibold text-fg-subtle uppercase tracking-wider mb-4" }, "\n            Contents\n          ")
const _hoisted_5 = { class: "font-mono text-lg mb-2" }
const _hoisted_6 = { class: "text-fg-subtle text-sm" }
import { setResponseHeader } from 'h3'
import type { DocsResponse } from '#shared/types'
import { assertValidPackageName, fetchLatestVersion } from '#shared/utils/npm'

export default /*@__PURE__*/_defineComponent({
  __name: '[...path]',
  async setup(__props) {

let __temp: any, __restore: any

definePageMeta({
  name: 'docs',
  path: '/package-docs/:path+',
  alias: ['/package/docs/:path+', '/docs/:path+'],
  scrollMargin: 180,
})
const route = useRoute('docs')
const router = useRouter()
const parsedRoute = computed(() => {
  const segments = route.params.path?.filter(Boolean)
  const vIndex = segments.indexOf('v')

  if (vIndex === -1 || vIndex >= segments.length - 1) {
    return {
      packageName: segments.join('/'),
      version: null as string | null,
    }
  }

  return {
    packageName: segments.slice(0, vIndex).join('/'),
    version: segments.slice(vIndex + 1).join('/'),
  }
})
const packageName = computed(() => parsedRoute.value.packageName)
const requestedVersion = computed(() => parsedRoute.value.version)
// Validate package name on server-side for early error detection
if (import.meta.server && packageName.value) {
  assertValidPackageName(packageName.value)
}
const { data: pkg } = usePackage(packageName)
const latestVersion = computed(() => pkg.value?.['dist-tags']?.latest ?? null)
if (import.meta.server && !requestedVersion.value && packageName.value) {
  const app = useNuxtApp()
  const version = await fetchLatestVersion(packageName.value)
  if (version) {
    setResponseHeader(useRequestEvent()!, 'Cache-Control', 'no-cache')
    const pathSegments = [...packageName.value.split('/'), 'v', version]
    app.runWithContext(() =>
      navigateTo(
        { name: 'docs', params: { path: pathSegments as [string, ...string[]] } },
        { redirectCode: 302 },
      ),
    )
  }
}
watch(
  [requestedVersion, latestVersion, packageName],
  ([version, latest, name]) => {
    if (!version && latest && name) {
      const pathSegments = [...name.split('/'), 'v', latest]
      router.replace({ name: 'docs', params: { path: pathSegments as [string, ...string[]] } })
    }
  },
  { immediate: true },
)
const resolvedVersion = computed(() => requestedVersion.value ?? latestVersion.value)
const docsUrl = computed(() => {
  if (!packageName.value || !resolvedVersion.value) return null
  return `/api/registry/docs/${packageName.value}/v/${resolvedVersion.value}`
})
const shouldFetch = computed(() => !!docsUrl.value)
const { data: docsData, status: docsStatus } = useLazyFetch<DocsResponse>(
  () => docsUrl.value ?? '',
  {
    watch: [docsUrl],
    immediate: shouldFetch.value,
    server: false,
    default: () => ({
      package: packageName.value,
      version: resolvedVersion.value ?? '',
      html: '',
      toc: null,
      status: 'missing' as const,
      message: 'Docs are not available for this version.',
    }),
  },
)
const pageTitle = computed(() => {
  if (!packageName.value) return 'API Docs - npmx'
  if (!resolvedVersion.value) return `${packageName.value} docs - npmx`
  return `${packageName.value}@${resolvedVersion.value} docs - npmx`
})
useSeoMeta({
  title: () => pageTitle.value,
  ogTitle: () => pageTitle.value,
  twitterTitle: () => pageTitle.value,
  description: () => pkg.value?.license ?? '',
  ogDescription: () => pkg.value?.license ?? '',
  twitterDescription: () => pkg.value?.license ?? '',
})
defineOgImageComponent('Default', {
  title: () => `${pkg.value?.name ?? 'Package'} - Docs`,
  description: () => pkg.value?.license ?? '',
  primaryColor: '#60a5fa',
})
const showLoading = computed(
  () => docsStatus.value === 'pending' || (docsStatus.value === 'idle' && docsUrl.value !== null),
)
const showEmptyState = computed(() => docsData.value?.status !== 'ok')

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_VersionSelector = _resolveComponent("VersionSelector")
  const _component_SkeletonBlock = _resolveComponent("SkeletonBlock")

  return (_openBlock(), _createElementBlock("div", { class: "docs-page flex-1 flex flex-col" }, [ _createElementVNode("h1", _hoisted_1, _toDisplayString(packageName.value) + " API Documentation", 1 /* TEXT */), _createTextVNode("\n\n    " + "\n    "), _createElementVNode("header", {
        "aria-label": "Package documentation header",
        class: "docs-header sticky z-10 border-b border-border"
      }, [ _hoisted_2, _createElementVNode("div", { class: "relative px-4 sm:px-6 lg:px-8 py-4 z-1" }, [ _createElementVNode("div", { class: "flex items-center justify-between gap-4" }, [ _createElementVNode("div", { class: "flex items-center gap-3 min-w-0" }, [ (packageName.value) ? (_openBlock(), _createBlock(_component_NuxtLink, {
                  key: 0,
                  to: _ctx.packageRoute(packageName.value),
                  class: "font-mono text-lg sm:text-xl font-semibold text-fg hover:text-fg-muted transition-colors truncate"
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(packageName.value), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true), (resolvedVersion.value && _unref(pkg)?.versions && _unref(pkg)?.['dist-tags']) ? (_openBlock(), _createBlock(_component_VersionSelector, {
                  key: 0,
                  "package-name": packageName.value,
                  "current-version": resolvedVersion.value,
                  versions: _unref(pkg).versions,
                  "dist-tags": _unref(pkg)['dist-tags'],
                  "url-pattern": `/package-docs/${packageName.value}/v/{version}`
                }, null, 8 /* PROPS */, ["package-name", "current-version", "versions", "dist-tags", "url-pattern"])) : (resolvedVersion.value) ? (_openBlock(), _createElementBlock("span", {
                    key: 1,
                    class: "text-fg-subtle font-mono text-sm shrink-0"
                  }, _toDisplayString(resolvedVersion.value), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", { class: "flex items-center gap-3 shrink-0" }, [ _hoisted_3 ]) ]) ]) ]), _createElementVNode("div", {
        class: "flex",
        dir: "ltr"
      }, [ (_unref(docsData)?.toc && !showEmptyState.value) ? (_openBlock(), _createElementBlock("aside", {
            key: 0,
            class: "hidden lg:block w-64 xl:w-72 shrink-0 border-ie border-border"
          }, [ _createElementVNode("div", { class: "docs-sidebar sticky overflow-y-auto p-4" }, [ _hoisted_4, _createTextVNode("\n          " + "\n          "), _createElementVNode("div", {
                class: "toc-content",
                innerHTML: _unref(docsData).toc
              }, null, 8 /* PROPS */, ["innerHTML"]) ]) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n      " + "\n      "), _createElementVNode("main", { class: "flex-1 min-w-0" }, [ (showLoading.value) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "p-6 sm:p-8 lg:p-12 space-y-4"
            }, [ _createVNode(_component_SkeletonBlock, { class: "h-8 w-64 rounded" }), _createVNode(_component_SkeletonBlock, { class: "h-4 w-full max-w-2xl rounded" }), _createVNode(_component_SkeletonBlock, { class: "h-4 w-5/6 max-w-2xl rounded" }), _createVNode(_component_SkeletonBlock, { class: "h-4 w-3/4 max-w-2xl rounded" }) ])) : (showEmptyState.value) ? (_openBlock(), _createElementBlock("div", {
                key: 1,
                class: "p-6 sm:p-8 lg:p-12"
              }, [ _createElementVNode("div", { class: "max-w-xl rounded-lg border border-border bg-bg-muted p-6" }, [ _createElementVNode("h2", _hoisted_5, _toDisplayString(_ctx.$t('package.docs.not_available')), 1 /* TEXT */), _createElementVNode("p", _hoisted_6, _toDisplayString(_unref(docsData)?.message ?? _ctx.$t('package.docs.not_available_detail')), 1 /* TEXT */), _createElementVNode("div", { class: "flex gap-4 mt-4" }, [ (packageName.value) ? (_openBlock(), _createBlock(_component_NuxtLink, {
                        key: 0,
                        to: _ctx.packageRoute(packageName.value),
                        class: "link-subtle font-mono text-sm"
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode("\n                View package\n              ")
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true) ]) ]) ])) : (_openBlock(), _createElementBlock("div", {
              key: 2,
              class: "docs-content p-6 sm:p-8 lg:p-12",
              innerHTML: _unref(docsData)?.html
            })), _createTextVNode("\n\n        " + "\n        ") ]) ]) ]))
}
}

})
