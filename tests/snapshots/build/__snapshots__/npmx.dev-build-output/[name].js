import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "text-xs" }, "Skeleton")
const _hoisted_2 = { class: "font-mono text-fg-muted text-sm", dir: "ltr" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:arrow-right rtl-flip w-3 h-3", "aria-hidden": "true" })
const _hoisted_4 = { class: "max-sm:sr-only" }
const _hoisted_5 = { class: "max-sm:sr-only" }
const _hoisted_6 = { class: "max-sm:sr-only" }
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("span", { class: "opacity-50" }, "/")
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("li", { class: "basis-full sm:hidden" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:diff w-4 h-4", "aria-hidden": "true" })
const _hoisted_10 = { class: "font-medium mb-2" }
const _hoisted_11 = { class: "text-xs text-fg-subtle uppercase tracking-wider" }
const _hoisted_12 = { class: "text-xs text-fg-subtle uppercase tracking-wider" }
const _hoisted_13 = { class: "text-fg-muted" }
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle" }, "/")
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("span", { class: "i-svg-spinners:ring-resize w-3 h-3", "aria-hidden": "true" })
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle" }, "-")
const _hoisted_17 = { class: "sr-only" }
const _hoisted_18 = { class: "sr-only" }
const _hoisted_19 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:info w-3 h-3", "aria-hidden": "true" })
const _hoisted_20 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle mx-1" }, "/")
const _hoisted_21 = /*#__PURE__*/ _createElementVNode("span", { class: "i-svg-spinners:ring-resize w-3 h-3", "aria-hidden": "true" })
const _hoisted_22 = { class: "text-xs text-fg-subtle uppercase tracking-wider" }
const _hoisted_23 = /*#__PURE__*/ _createElementVNode("span", { class: "i-svg-spinners:ring-resize w-3 h-3", "aria-hidden": "true" })
const _hoisted_24 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:check w-3 h-3", "aria-hidden": "true" })
const _hoisted_25 = { id: "run-heading", class: "text-xs text-fg-subtle uppercase tracking-wider" }
const _hoisted_26 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:circle-alert w-4 h-4 shrink-0", "aria-hidden": "true" })
const _hoisted_27 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:external-link w-3 h-3", "aria-hidden": "true" })
const _hoisted_28 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:external-link w-3 h-3", "aria-hidden": "true" })
const _hoisted_29 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:external-link w-3 h-3", "aria-hidden": "true" })
const _hoisted_30 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:external-link w-3 h-3", "aria-hidden": "true" })
const _hoisted_31 = /*#__PURE__*/ _createElementVNode("span", { class: "i-svg-spinners:ring-resize w-4 h-4", "aria-hidden": "true" })
const _hoisted_32 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:circle-alert w-4 h-4", "aria-hidden": "true" })
const _hoisted_33 = { class: "font-mono text-2xl font-medium mb-4" }
const _hoisted_34 = { class: "text-fg-muted mb-8" }
import type { InstallSizeResult, NpmVersionDist, PackageVersionInfo, PackumentVersion, ProvenanceDetails, ReadmeResponse, ReadmeMarkdownResponse, SkillsListResponse } from '#shared/types'
import type { JsrPackageInfo } from '#shared/types/jsr'
import type { IconClass } from '~/types'
import { assertValidPackageName } from '#shared/utils/npm'
import { joinURL } from 'ufo'
import { areUrlsEquivalent } from '#shared/utils/url'
import { isEditableElement } from '~/utils/input'
import { getDependencyCount } from '~/utils/npm/dependency-count'
import { detectPublishSecurityDowngradeForVersion } from '~/utils/publish-security'
import { useModal } from '~/composables/useModal'
import { useAtproto } from '~/composables/atproto/useAtproto'
import { togglePackageLike } from '~/utils/atproto/likes'
import { useInstallSizeDiff } from '~/composables/useInstallSizeDiff'
import type { RouteLocationRaw } from 'vue-router'

export default /*@__PURE__*/_defineComponent({
  __name: '[name]',
  async setup(__props) {

let __temp: any, __restore: any

defineOgImageComponent('Package', {
  name: () => packageName.value,
  version: () => requestedVersion.value ?? '',
  primaryColor: '#60a5fa',
})
const router = useRouter()
const header = useTemplateRef('header')
const isHeaderPinned = shallowRef(false)
const navExtraOffset = shallowRef(0)
const isMobile = useMediaQuery('(max-width: 639.9px)')
function checkHeaderPosition() {
  const el = header.value
  if (!el) return
  const style = getComputedStyle(el)
  const top = parseFloat(style.top) || 0
  const rect = el.getBoundingClientRect()
  isHeaderPinned.value = Math.abs(rect.top - top) < 1
}
useEventListener('scroll', checkHeaderPosition, { passive: true })
useEventListener('resize', checkHeaderPosition)
const footerTarget = ref<HTMLElement | null>(null)
const footerThresholds = Array.from({ length: 11 }, (_, i) => i / 10)
const { pause: pauseFooterObserver, resume: resumeFooterObserver } = useIntersectionObserver(
  footerTarget,
  ([entry]) => {
    if (!entry) return
    navExtraOffset.value = entry.isIntersecting ? entry.intersectionRect.height : 0
  },
  {
    threshold: footerThresholds,
    immediate: false,
  },
)
function initFooterObserver() {
  footerTarget.value = document.querySelector('footer')
  if (!footerTarget.value) return
  pauseFooterObserver()
  watch(
    isMobile,
    value => {
      if (value) {
        resumeFooterObserver()
      } else {
        pauseFooterObserver()
        navExtraOffset.value = 0
      }
    },
    { immediate: true },
  )
}
onMounted(() => {
  checkHeaderPosition()
  initFooterObserver()
})
const navExtraOffsetStyle = computed(() => ({
  '--package-nav-extra': `${navExtraOffset.value}px`,
}))
const { packageName, requestedVersion, orgName } = usePackageRoute()
if (import.meta.server) {
  assertValidPackageName(packageName.value)
}
// Fetch README for specific version if requested, otherwise latest
const { data: readmeData } = useLazyFetch<ReadmeResponse>(
  () => {
    const base = `/api/registry/readme/${packageName.value}`
    const version = requestedVersion.value
    return version ? `${base}/v/${version}` : base
  },
  {
    default: () => ({
      html: '',
      mdExists: false,
      playgroundLinks: [],
      toc: [],
      defaultValue: true,
    }),
  },
)
const playgroundLinks = computed(() => [
  ...readmeData.value.playgroundLinks,
  // Libraries with a storybook field in package.json contain a link to their deployed playground
  ...(pkg.value?.storybook?.url
    ? [
        {
          url: pkg.value.storybook.url,
          provider: 'storybook',
          providerName: 'Storybook',
          label: 'Storybook',
        },
      ]
    : []),
])
const {
  data: readmeMarkdownData,
  status: readmeMarkdownStatus,
  execute: fetchReadmeMarkdown,
} = useLazyFetch<ReadmeMarkdownResponse>(
  () => {
    const base = `/api/registry/readme/markdown/${packageName.value}`
    const version = requestedVersion.value
    return version ? `${base}/v/${version}` : base
  },
  {
    server: false,
    immediate: false,
    default: () => ({}),
  },
)
//copy README file as Markdown
const { copied: copiedReadme, copy: copyReadme } = useClipboard({
  source: () => '',
  copiedDuring: 2000,
})
function prefetchReadmeMarkdown() {
  if (readmeMarkdownStatus.value === 'idle') {
    fetchReadmeMarkdown()
  }
}
async function copyReadmeHandler() {
;(
  ([__temp,__restore] = _withAsyncContext(() => fetchReadmeMarkdown())),
  await __temp,
  __restore()
)
  const markdown = readmeMarkdownData.value?.markdown
  if (!markdown) return
;(
  ([__temp,__restore] = _withAsyncContext(() => copyReadme(markdown))),
  await __temp,
  __restore()
)
}
// Track active TOC item based on scroll position
const tocItems = computed(() => readmeData.value?.toc ?? [])
const { activeId: activeTocId } = useActiveTocItem(tocItems)
// Check if package exists on JSR (only for scoped packages)
const { data: jsrInfo } = useLazyFetch<JsrPackageInfo>(() => `/api/jsr/${packageName.value}`, {
  default: () => ({ exists: false }),
  // Only fetch for scoped packages (JSR requirement)
  immediate: computed(() => packageName.value.startsWith('@')).value,
})
// Fetch total install size (lazy, can be slow for large dependency trees)
const {
  data: installSize,
  status: installSizeStatus,
  execute: fetchInstallSize,
} = useLazyFetch<InstallSizeResult | null>(
  () => {
    const base = `/api/registry/install-size/${packageName.value}`
    const version = requestedVersion.value
    return version ? `${base}/v/${version}` : base
  },
  {
    server: false,
    immediate: false,
  },
)
onMounted(() => fetchInstallSize())
const { data: skillsData } = useLazyFetch<SkillsListResponse>(
  () => {
    const base = `/skills/${packageName.value}`
    const version = requestedVersion.value
    return version ? `${base}/v/${version}` : base
  },
  { default: () => ({ package: '', version: '', skills: [] }) },
)
const { data: packageAnalysis } = usePackageAnalysis(packageName, requestedVersion)
const { data: moduleReplacement } = useModuleReplacement(packageName)
const { data: resolvedVersion, status: resolvedStatus } = await useResolvedVersion(
  packageName,
  requestedVersion,
)
if (
  import.meta.server &&
  !resolvedVersion.value &&
  ['success', 'error'].includes(resolvedStatus.value)
) {
  throw createError({
    statusCode: 404,
    statusMessage: $t('package.not_found'),
    message: $t('package.not_found_message'),
  })
}
watch(
  [resolvedStatus, resolvedVersion],
  ([status, version]) => {
    if ((!version && status === 'success') || status === 'error') {
      showError({
        statusCode: 404,
        statusMessage: $t('package.not_found'),
        message: $t('package.not_found_message'),
      })
    }
  },
  { immediate: true },
)
const {
  data: pkg,
  status,
  error,
} = usePackage(packageName, () => resolvedVersion.value ?? requestedVersion.value)
const { diff: sizeDiff } = useInstallSizeDiff(packageName, resolvedVersion, pkg, installSize)
// Detect two hydration scenarios where the external _payload.json is missing:
//
// 1. SPA fallback (200.html): No real content was server-rendered.
//    → Show skeleton while data fetches on the client.
//
// 2. SSR-rendered HTML with missing payload: Content was rendered but the external _payload.json
//    returned an ISR fallback.
//    → Preserve the server-rendered DOM, don't flash to skeleton.
const nuxtApp = useNuxtApp()
const route = useRoute()
const hasEmptyPayload =
  import.meta.client &&
  nuxtApp.payload.serverRendered &&
  !Object.keys(nuxtApp.payload.data ?? {}).length
const isSpaFallback = shallowRef(nuxtApp.isHydrating && hasEmptyPayload && !nuxtApp.payload.path)
const isHydratingWithServerContent = shallowRef(
  nuxtApp.isHydrating && hasEmptyPayload && nuxtApp.payload.path === route.path,
)
const hasServerContentOnly = shallowRef(hasEmptyPayload && nuxtApp.payload.path === route.path)
// When we have server-rendered content but no payload data, capture the server
// DOM before Vue's hydration replaces it. This lets us show the server-rendered
// HTML as a static snapshot while data refetches, avoiding any visual flash.
const serverRenderedHtml = shallowRef<string | null>(
  hasServerContentOnly.value
    ? (document.getElementById('package-article')?.innerHTML ?? null)
    : null,
)
if (isSpaFallback.value || isHydratingWithServerContent.value) {
  nuxtApp.hooks.hookOnce('app:suspense:resolve', () => {
    isSpaFallback.value = false
    isHydratingWithServerContent.value = false
  })
}
const displayVersion = computed(() => pkg.value?.requestedVersion ?? null)
const versionSecurityMetadata = computed<PackageVersionInfo[]>(() => {
  if (!pkg.value) return []
  if (pkg.value.securityVersions?.length) return pkg.value.securityVersions

  return Object.entries(pkg.value.versions).map(([version, metadata]) => ({
    version,
    time: pkg.value?.time?.[version],
    hasProvenance: !!metadata.hasProvenance,
    trustLevel: metadata.trustLevel,
    deprecated: metadata.deprecated,
  }))
})
// Process package description
const pkgDescription = useMarkdown(() => ({
  text: pkg.value?.description ?? '',
  packageName: pkg.value?.name,
}))
//copy package name
const { copied: copiedPkgName, copy: copyPkgName } = useClipboard({
  source: packageName,
  copiedDuring: 2000,
})
//copy version name
const { copied: copiedVersion, copy: copyVersion } = useClipboard({
  source: () => resolvedVersion.value ?? '',
  copiedDuring: 2000,
})
const { scrollToTop, isTouchDeviceClient } = useScrollToTop()
const { y: scrollY } = useScroll(window)
const showScrollToTop = computed(
  () => isTouchDeviceClient.value && scrollY.value > SCROLL_TO_TOP_THRESHOLD,
)
// Fetch dependency analysis (lazy, client-side)
// This is the same composable used by PackageVulnerabilityTree and PackageDeprecatedTree
const { data: vulnTree, status: vulnTreeStatus } = useDependencyAnalysis(
  packageName,
  () => resolvedVersion.value ?? '',
)
const {
  data: provenanceData,
  status: provenanceStatus,
  execute: fetchProvenance,
} = useLazyFetch<ProvenanceDetails | null>(
  () => {
    const v = displayVersion.value
    if (!v || !hasProvenance(v)) return ''
    return `/api/registry/provenance/${packageName.value}/v/${v.version}`
  },
  {
    default: () => null,
    server: false,
    immediate: false,
  },
)
if (import.meta.client) {
  watch(
    displayVersion,
    v => {
      if (v && hasProvenance(v) && provenanceStatus.value === 'idle') {
        fetchProvenance()
      }
    },
    { immediate: true },
  )
}
const isMounted = useMounted()
// Keep latestVersion for comparison (to show "(latest)" badge)
const latestVersion = computed(() => {
  if (!pkg.value) return null
  const latestTag = pkg.value['dist-tags']?.latest
  if (!latestTag) return null
  return pkg.value.versions[latestTag] ?? null
})
const deprecationNotice = computed(() => {
  if (!displayVersion.value?.deprecated) return null

  const isLatestDeprecated = !!latestVersion.value?.deprecated

  // If latest is deprecated, show "package deprecated"
  if (isLatestDeprecated) {
    return {
      type: 'package' as const,
      message: displayVersion.value.deprecated,
    }
  }

  // Otherwise show "version deprecated"
  return { type: 'version' as const, message: displayVersion.value.deprecated }
})
const deprecationNoticeMessage = useMarkdown(() => ({
  text: deprecationNotice.value?.message ?? '',
}))
const publishSecurityDowngrade = computed(() => {
  const currentVersion = displayVersion.value?.version
  if (!currentVersion) return null
  return detectPublishSecurityDowngradeForVersion(versionSecurityMetadata.value, currentVersion)
})
const installVersionOverride = computed(
  () => publishSecurityDowngrade.value?.trustedVersion ?? null,
)
const downgradeFallbackInstallText = computed(() => {
  const d = publishSecurityDowngrade.value
  if (!d?.trustedVersion) return null
  if (d.trustedTrustLevel === 'provenance')
    return $t('package.security_downgrade.fallback_install_provenance', {
      version: d.trustedVersion,
    })
  if (d.trustedTrustLevel === 'trustedPublisher')
    return $t('package.security_downgrade.fallback_install_trustedPublisher', {
      version: d.trustedVersion,
    })
  return null
})
const sizeTooltip = computed(() => {
  const chunks = [
    displayVersion.value &&
      displayVersion.value.dist.unpackedSize &&
      $t('package.stats.size_tooltip.unpacked', {
        size: bytesFormatter.format(displayVersion.value.dist.unpackedSize),
      }),
    installSize.value &&
      installSize.value.dependencyCount &&
      $t('package.stats.size_tooltip.total', {
        size: bytesFormatter.format(installSize.value.totalSize),
        count: installSize.value.dependencyCount,
      }),
  ]
  return chunks.filter(Boolean).join('\n')
})
const hasDependencies = computed(() => {
  if (!displayVersion.value) return false
  const deps = displayVersion.value.dependencies
  const peerDeps = displayVersion.value.peerDependencies
  const optionalDeps = displayVersion.value.optionalDependencies
  return (
    (deps && Object.keys(deps).length > 0) ||
    (peerDeps && Object.keys(peerDeps).length > 0) ||
    (optionalDeps && Object.keys(optionalDeps).length > 0)
  )
})
// Vulnerability count for the stats banner
const vulnCount = computed(() => vulnTree.value?.totalCounts.total ?? 0)
const hasVulnerabilities = computed(() => vulnCount.value > 0)
// Total transitive dependencies count (from either vuln tree or install size)
// Subtract 1 to exclude the root package itself
const totalDepsCount = computed(() => {
  if (vulnTree.value) {
    return vulnTree.value.totalPackages ? vulnTree.value.totalPackages - 1 : 0
  }
  if (installSize.value) {
    return installSize.value.dependencyCount
  }
  return null
})
const repositoryUrl = computed(() => {
  const repo = displayVersion.value?.repository
  if (!repo?.url) return null
  let url = normalizeGitUrl(repo.url)
  // append `repository.directory` for monorepo packages
  if (repo.directory) {
    url = joinURL(`${url}/tree/HEAD`, repo.directory)
  }
  return url
})
const { meta: repoMeta, repoRef, stars, starsLink, forks, forksLink } = useRepoMeta(repositoryUrl)
const PROVIDER_ICONS: Record<string, IconClass> = {
  github: 'i-simple-icons:github',
  gitlab: 'i-simple-icons:gitlab',
  bitbucket: 'i-simple-icons:bitbucket',
  codeberg: 'i-simple-icons:codeberg',
  gitea: 'i-simple-icons:gitea',
  forgejo: 'i-simple-icons:forgejo',
  gitee: 'i-simple-icons:gitee',
  sourcehut: 'i-simple-icons:sourcehut',
  tangled: 'i-custom:tangled',
  radicle: 'i-lucide:network', // Radicle is a P2P network, using network icon
}
const repoProviderIcon = computed((): IconClass => {
  const provider = repoRef.value?.provider
  if (!provider) return 'i-simple-icons:github'
  return PROVIDER_ICONS[provider] ?? 'i-lucide:code'
})
const homepageUrl = computed(() => {
  const homepage = displayVersion.value?.homepage
  if (!homepage) return null

  // Don't show homepage if it's the same as the repository URL
  if (repositoryUrl.value && areUrlsEquivalent(homepage, repositoryUrl.value)) {
    return null
  }

  return homepage
})
// Docs URL: use our generated API docs
const docsLink = computed(() => {
  if (!resolvedVersion.value) return null

  return {
    name: 'docs' as const,
    params: {
      path: [pkg.value!.name, 'v', resolvedVersion.value] satisfies [string, string, string],
    },
  }
})
const fundingUrl = computed(() => {
  let funding = displayVersion.value?.funding
  if (Array.isArray(funding)) funding = funding[0]

  if (!funding) return null

  return typeof funding === 'string' ? funding : funding.url
})
function normalizeGitUrl(url: string): string {
  return url
    .replace(/^git\+/, '')
    .replace(/^git:\/\//, 'https://')
    .replace(/\.git$/, '')
    .replace(/^ssh:\/\/git@github\.com/, 'https://github.com')
    .replace(/^git@github\.com:/, 'https://github.com/')
}
// Check if a version has provenance/attestations
// The dist object may have attestations that aren't in the base type
function hasProvenance(version: PackumentVersion | null): boolean {
  if (!version?.dist) return false
  const dist = version.dist as NpmVersionDist
  return !!dist.attestations
}
// Get @types package name if available (non-deprecated)
const typesPackageName = computed(() => {
  if (!packageAnalysis.value) return null
  if (packageAnalysis.value.types.kind !== '@types') return null
  if (packageAnalysis.value.types.deprecated) return null
  return packageAnalysis.value.types.packageName
})
// Executable detection for run command
const executableInfo = computed(() => {
  if (!displayVersion.value || !pkg.value) return null
  return getExecutableInfo(pkg.value.name, displayVersion.value.bin)
})
// Detect if package is binary-only (show only execute commands, no install)
const isBinaryOnly = computed(() => {
  if (!displayVersion.value || !pkg.value) return false
  return isBinaryOnlyPackage({
    name: pkg.value.name,
    bin: displayVersion.value.bin,
    main: displayVersion.value.main,
    module: displayVersion.value.module,
    exports: displayVersion.value.exports,
  })
})
// Detect if package uses create-* naming convention
const isCreatePkg = computed(() => {
  if (!pkg.value) return false
  return isCreatePackage(pkg.value.name)
})
// Get associated create-* package info (e.g., vite -> create-vite)
const createPackageInfo = computed(() => {
  if (!packageAnalysis.value?.createPackage) return null
  // Don't show if deprecated
  if (packageAnalysis.value.createPackage.deprecated) return null
  return packageAnalysis.value.createPackage
})
// Canonical URL for this package page
const canonicalUrl = computed(() => {
  const base = `https://npmx.dev/package/${packageName.value}`
  return requestedVersion.value ? `${base}/v/${requestedVersion.value}` : base
})
//atproto
// TODO: Maybe set this where it's not loaded here every load?
const { user } = useAtproto()
const authModal = useModal('auth-modal')
const { data: likesData, status: likeStatus } = useFetch(
  () => `/api/social/likes/${packageName.value}`,
  {
    default: () => ({ totalLikes: 0, userHasLiked: false }),
    server: false,
  },
)
const isLoadingLikeData = computed(
  () => likeStatus.value === 'pending' || likeStatus.value === 'idle',
)
const isLikeActionPending = shallowRef(false)
const likeAction = async () => {
  if (user.value?.handle == null) {
    authModal.open()
    return
  }

  if (isLikeActionPending.value) return

  const currentlyLiked = likesData.value?.userHasLiked ?? false
  const currentLikes = likesData.value?.totalLikes ?? 0

  // Optimistic update
  likesData.value = {
    totalLikes: currentlyLiked ? currentLikes - 1 : currentLikes + 1,
    userHasLiked: !currentlyLiked,
  }

  isLikeActionPending.value = true

  try {
    const result = await togglePackageLike(packageName.value, currentlyLiked, user.value?.handle)

    isLikeActionPending.value = false

    if (result.success) {
      // Update with server response
      likesData.value = result.data
    } else {
      // Revert on error
      likesData.value = {
        totalLikes: currentLikes,
        userHasLiked: currentlyLiked,
      }
    }
  } catch {
    // Revert on error
    likesData.value = {
      totalLikes: currentLikes,
      userHasLiked: currentlyLiked,
    }
    isLikeActionPending.value = false
  }
}
const dependencyCount = computed(() => getDependencyCount(displayVersion.value))
const numberFormatter = useNumberFormatter()
const compactNumberFormatter = useCompactNumberFormatter()
const bytesFormatter = useBytesFormatter()
useHead({
  link: [{ rel: 'canonical', href: canonicalUrl }],
})
useSeoMeta({
  title: () => (pkg.value?.name ? `${pkg.value.name} - npmx` : 'Package - npmx'),
  ogTitle: () => (pkg.value?.name ? `${pkg.value.name} - npmx` : 'Package - npmx'),
  twitterTitle: () => (pkg.value?.name ? `${pkg.value.name} - npmx` : 'Package - npmx'),
  description: () => pkg.value?.description ?? '',
  ogDescription: () => pkg.value?.description ?? '',
  twitterDescription: () => pkg.value?.description ?? '',
})
const codeLink = computed((): RouteLocationRaw | null => {
  if (pkg.value == null || resolvedVersion.value == null) {
    return null
  }
  const split = pkg.value.name.split('/')
  return {
    name: 'code',
    params: {
      org: split.length === 2 ? split[0] : undefined,
      packageName: split.length === 2 ? split[1]! : split[0]!,
      version: resolvedVersion.value,
      filePath: '',
    },
  }
})
const keyboardShortcuts = useKeyboardShortcuts()
onKeyStroke(
  e => keyboardShortcuts.value && isKeyWithoutModifiers(e, '.') && !isEditableElement(e.target),
  e => {
    if (codeLink.value === null) return
    e.preventDefault()
    navigateTo(codeLink.value)
  },
  { dedupe: true },
)
onKeyStroke(
  e => keyboardShortcuts.value && isKeyWithoutModifiers(e, 'd') && !isEditableElement(e.target),
  e => {
    if (!docsLink.value) return
    e.preventDefault()
    navigateTo(docsLink.value)
  },
  { dedupe: true },
)
onKeyStroke(
  e => keyboardShortcuts.value && isKeyWithoutModifiers(e, 'c') && !isEditableElement(e.target),
  e => {
    if (!pkg.value) return
    e.preventDefault()
    router.push({ name: 'compare', query: { packages: pkg.value.name } })
  },
)
const showSkeleton = shallowRef(false)

return (_ctx: any,_cache: any) => {
  const _component_ButtonBase = _resolveComponent("ButtonBase")
  const _component_DevOnly = _resolveComponent("DevOnly")
  const _component_PackageSkeleton = _resolveComponent("PackageSkeleton")
  const _component_LinkBase = _resolveComponent("LinkBase")
  const _component_CopyToClipboardButton = _resolveComponent("CopyToClipboardButton")
  const _component_TooltipApp = _resolveComponent("TooltipApp")
  const _component_ButtonGroup = _resolveComponent("ButtonGroup")
  const _component_PackageMetricsBadges = _resolveComponent("PackageMetricsBadges")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_LicenseDisplay = _resolveComponent("LicenseDisplay")
  const _component_ClientOnly = _resolveComponent("ClientOnly")
  const _component_DateTime = _resolveComponent("DateTime")
  const _component_PackageSkillsModal = _resolveComponent("PackageSkillsModal")
  const _component_PackageManagerSelect = _resolveComponent("PackageManagerSelect")
  const _component_TerminalExecute = _resolveComponent("TerminalExecute")
  const _component_i18n_t = _resolveComponent("i18n-t")
  const _component_TerminalInstall = _resolveComponent("TerminalInstall")
  const _component_PackageReplacement = _resolveComponent("PackageReplacement")
  const _component_PackageSizeIncrease = _resolveComponent("PackageSizeIncrease")
  const _component_PackageVulnerabilityTree = _resolveComponent("PackageVulnerabilityTree")
  const _component_PackageDeprecatedTree = _resolveComponent("PackageDeprecatedTree")
  const _component_ReadmeTocDropdown = _resolveComponent("ReadmeTocDropdown")
  const _component_Readme = _resolveComponent("Readme")
  const _component_PackageProvenanceSection = _resolveComponent("PackageProvenanceSection")
  const _component_PackageAccessControls = _resolveComponent("PackageAccessControls")
  const _component_PackageSkillsCard = _resolveComponent("PackageSkillsCard")
  const _component_PackageWeeklyDownloadStats = _resolveComponent("PackageWeeklyDownloadStats")
  const _component_PackagePlaygrounds = _resolveComponent("PackagePlaygrounds")
  const _component_PackageCompatibility = _resolveComponent("PackageCompatibility")
  const _component_PackageVersions = _resolveComponent("PackageVersions")
  const _component_PackageInstallScripts = _resolveComponent("PackageInstallScripts")
  const _component_PackageDependencies = _resolveComponent("PackageDependencies")
  const _component_PackageKeywords = _resolveComponent("PackageKeywords")
  const _component_PackageMaintainers = _resolveComponent("PackageMaintainers")
  const _component_PackageSidebar = _resolveComponent("PackageSidebar")

  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createVNode(_component_DevOnly, null, {
        default: _withCtx(() => [
          _createVNode(_component_ButtonBase, {
            class: "fixed bottom-4 inset-is-4 z-50 shadow-lg rounded-full! px-3! py-2!",
            classicon: "i-simple-icons:skeleton",
            variant: "primary",
            title: "Toggle skeleton loader (development only)",
            "aria-pressed": showSkeleton.value,
            onClick: _cache[0] || (_cache[0] = ($event: any) => (showSkeleton.value = !showSkeleton.value))
          }, {
            default: _withCtx(() => [
              _hoisted_1
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["aria-pressed"])
        ]),
        _: 1 /* STABLE */
      }), _createElementVNode("main", { class: "container flex-1 w-full py-8" }, [ (isSpaFallback.value || (!hasServerContentOnly.value && (showSkeleton.value || _unref(status) === 'pending'))) ? (_openBlock(), _createBlock(_component_PackageSkeleton, { key: 0 })) : ( isHydratingWithServerContent.value || (hasServerContentOnly.value && serverRenderedHtml.value && (!_unref(pkg) || _unref(readmeData)?.defaultValue)) ) ? (_openBlock(), _createElementBlock("article", {
              key: 1,
              id: "package-article",
              class: _normalizeClass(_ctx.$style.packagePage),
              innerHTML: serverRenderedHtml.value
            })) : (_unref(pkg)) ? (_openBlock(), _createElementBlock("article", {
              key: 2,
              id: "package-article",
              class: _normalizeClass(_ctx.$style.packagePage)
            }, [ _createElementVNode("header", {
                ref_key: "header", ref: header,
                class: _normalizeClass(["sticky top-14 z-1 bg-[--bg] py-2 border-border", [_ctx.$style.areaHeader, { 'border-b': isHeaderPinned.value }]])
              }, [ _createElementVNode("div", { class: "flex items-baseline gap-x-2 gap-y-1 sm:gap-x-3 flex-wrap min-w-0" }, [ _createVNode(_component_CopyToClipboardButton, {
                    copied: _unref(copiedPkgName),
                    "copy-text": _ctx.$t('package.copy_name'),
                    class: "flex flex-col items-start min-w-0",
                    onClick: _cache[1] || (_cache[1] = ($event: any) => (_unref(copyPkgName)()))
                  }, {
                    default: _withCtx(() => [
                      _createElementVNode("h1", {
                        class: "font-mono text-2xl sm:text-3xl font-medium min-w-0 break-words",
                        title: _unref(pkg).name,
                        dir: "ltr"
                      }, [
                        (_unref(orgName))
                          ? (_openBlock(), _createBlock(_component_LinkBase, {
                            key: 0,
                            to: { name: 'org', params: { org: _unref(orgName) } }
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode("\n                @"),
                              _createTextVNode(_toDisplayString(_unref(orgName)), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["to"]))
                          : _createCommentVNode("v-if", true),
                        (_unref(orgName))
                          ? (_openBlock(), _createElementBlock("span", { key: 0 }, "/"))
                          : _createCommentVNode("v-if", true),
                        _createElementVNode("span", {
                          class: _normalizeClass({ 'text-fg-muted': _unref(orgName) })
                        }, _toDisplayString(_unref(orgName) ? _unref(pkg).name.replace(`@${_unref(orgName)}/`, '') : _unref(pkg).name), 3 /* TEXT, CLASS */)
                      ], 8 /* PROPS */, ["title"])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["copied", "copy-text"]), (_unref(resolvedVersion)) ? (_openBlock(), _createBlock(_component_CopyToClipboardButton, {
                      key: 0,
                      copied: _unref(copiedVersion),
                      "copy-text": _ctx.$t('package.copy_version'),
                      class: "inline-flex items-baseline gap-1.5 font-mono text-base sm:text-lg text-fg-muted shrink-0",
                      onClick: _cache[2] || (_cache[2] = ($event: any) => (_unref(copyVersion)()))
                    }, {
                      default: _withCtx(() => [
                        (_unref(requestedVersion) && _unref(resolvedVersion) !== _unref(requestedVersion))
                          ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                            _createElementVNode("span", _hoisted_2, _toDisplayString(_unref(requestedVersion)), 1 /* TEXT */),
                            _hoisted_3
                          ], 64 /* STABLE_FRAGMENT */))
                          : _createCommentVNode("v-if", true),
                        (_unref(requestedVersion) && _unref(resolvedVersion) !== _unref(requestedVersion))
                          ? (_openBlock(), _createBlock(_component_LinkBase, {
                            key: 0,
                            to: _ctx.packageRoute(_unref(pkg).name, _unref(resolvedVersion)),
                            title: _ctx.$t('package.view_permalink'),
                            dir: "ltr"
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(resolvedVersion)), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["to", "title"]))
                          : (_openBlock(), _createElementBlock("span", {
                            key: 1,
                            dir: "ltr"
                          }, "v" + _toDisplayString(_unref(resolvedVersion)), 1 /* TEXT */)),
                        (hasProvenance(displayVersion.value))
                          ? (_openBlock(), _createBlock(_component_TooltipApp, {
                            key: 0,
                            text: 
                    _unref(provenanceData) && _unref(provenanceStatus) !== 'pending'
                      ? _ctx.$t('package.provenance_section.built_and_signed_on', {
                          provider: _unref(provenanceData).providerLabel,
                        })
                      : _ctx.$t('package.verified_provenance')
                  ,
                            position: "bottom",
                            strategy: "fixed"
                          }, {
                            default: _withCtx(() => [
                              _createVNode(_component_LinkBase, {
                                variant: "button-secondary",
                                size: "small",
                                to: "#provenance",
                                "aria-label": _ctx.$t('package.provenance_section.view_more_details'),
                                classicon: "i-lucide:shield-check"
                              }, null, 8 /* PROPS */, ["aria-label"])
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["text"]))
                          : _createCommentVNode("v-if", true),
                        (_unref(requestedVersion) && latestVersion.value && _unref(resolvedVersion) !== latestVersion.value.version)
                          ? (_openBlock(), _createElementBlock("span", {
                            key: 0,
                            class: "text-fg-subtle text-sm shrink-0"
                          }, _toDisplayString(_ctx.$t('package.not_latest')), 1 /* TEXT */))
                          : _createCommentVNode("v-if", true)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["copied", "copy-text"])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n          " + "\n          "), (_unref(resolvedVersion)) ? (_openBlock(), _createBlock(_component_ButtonGroup, {
                      key: 0,
                      as: "nav",
                      "aria-label": _ctx.$t('package.navigation'),
                      style: _normalizeStyle(navExtraOffsetStyle.value),
                      class: _normalizeClass(["hidden sm:flex max-sm:flex max-sm:fixed max-sm:z-40 max-sm:inset-is-1/2 max-sm:-translate-x-1/2 max-sm:rtl:translate-x-1/2 max-sm:bg-[--bg]/90 max-sm:backdrop-blur-md max-sm:border max-sm:border-border max-sm:rounded-md max-sm:shadow-md ms-auto", _ctx.$style.packageNav])
                    }, {
                      default: _withCtx(() => [
                        (docsLink.value)
                          ? (_openBlock(), _createBlock(_component_LinkBase, {
                            key: 0,
                            variant: "button-secondary",
                            to: docsLink.value,
                            "aria-keyshortcuts": "d",
                            classicon: "i-lucide:file-text",
                            title: _ctx.$t('package.links.docs')
                          }, {
                            default: _withCtx(() => [
                              _createElementVNode("span", _hoisted_4, _toDisplayString(_ctx.$t('package.links.docs')), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["to", "title"]))
                          : _createCommentVNode("v-if", true),
                        (codeLink.value)
                          ? (_openBlock(), _createBlock(_component_LinkBase, {
                            key: 0,
                            variant: "button-secondary",
                            to: codeLink.value,
                            "aria-keyshortcuts": ".",
                            classicon: "i-lucide:code",
                            title: _ctx.$t('package.links.code')
                          }, {
                            default: _withCtx(() => [
                              _createElementVNode("span", _hoisted_5, _toDisplayString(_ctx.$t('package.links.code')), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["to", "title"]))
                          : _createCommentVNode("v-if", true),
                        _createVNode(_component_LinkBase, {
                          variant: "button-secondary",
                          to: { name: 'compare', query: { packages: _unref(pkg).name } },
                          "aria-keyshortcuts": "c",
                          classicon: "i-lucide:git-compare",
                          title: _ctx.$t('package.links.compare')
                        }, {
                          default: _withCtx(() => [
                            _createElementVNode("span", _hoisted_6, _toDisplayString(_ctx.$t('package.links.compare')), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["to", "title"]),
                        (
                  displayVersion.value && latestVersion.value && displayVersion.value.version !== latestVersion.value.version
                )
                          ? (_openBlock(), _createBlock(_component_LinkBase, {
                            key: 0,
                            variant: "button-secondary",
                            to: _ctx.diffRoute(_unref(pkg).name, displayVersion.value.version, latestVersion.value.version),
                            classicon: "i-lucide:diff"
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_ctx.$t('compare.compare_versions')), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["to"]))
                          : _createCommentVNode("v-if", true),
                        (showScrollToTop.value)
                          ? (_openBlock(), _createBlock(_component_ButtonBase, {
                            key: 0,
                            variant: "secondary",
                            title: _ctx.$t('common.scroll_to_top'),
                            "aria-label": _ctx.$t('common.scroll_to_top'),
                            onClick: _cache[3] || (_cache[3] = () => _unref(scrollToTop)()),
                            classicon: "i-lucide:arrow-up"
                          }, null, 8 /* PROPS */, ["title", "aria-label"]))
                          : _createCommentVNode("v-if", true)
                      ]),
                      _: 1 /* STABLE */
                    }, 12 /* STYLE, PROPS */, ["aria-label"])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n          " + "\n          "), _createElementVNode("div", { class: "basis-full flex gap-2 sm:gap-3 flex-wrap items-stretch" }, [ (_unref(resolvedVersion)) ? (_openBlock(), _createBlock(_component_PackageMetricsBadges, {
                        key: 0,
                        "package-name": _unref(pkg).name,
                        version: _unref(resolvedVersion),
                        "is-binary": isBinaryOnly.value,
                        class: "self-baseline"
                      }, null, 8 /* PROPS */, ["package-name", "version", "is-binary"])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n            " + "\n            "), _createVNode(_component_TooltipApp, {
                      text: 
                  isLoadingLikeData.value
                    ? _ctx.$t('common.loading')
                    : _unref(likesData)?.userHasLiked
                      ? _ctx.$t('package.likes.unlike')
                      : _ctx.$t('package.likes.like')
                ,
                      position: "bottom",
                      class: "items-center",
                      strategy: "fixed"
                    }, {
                      default: _withCtx(() => [
                        _createVNode(_component_ButtonBase, {
                          onClick: likeAction,
                          size: "small",
                          "aria-label": 
                    _unref(likesData)?.userHasLiked ? _ctx.$t('package.likes.unlike') : _ctx.$t('package.likes.like')
                  ,
                          "aria-pressed": _unref(likesData)?.userHasLiked,
                          classicon: 
                    _unref(likesData)?.userHasLiked
                      ? 'i-lucide:heart-minus text-red-500'
                      : 'i-lucide:heart-plus'
                
                        }, {
                          default: _withCtx(() => [
                            (isLoadingLikeData.value)
                              ? (_openBlock(), _createElementBlock("span", {
                                key: 0,
                                class: "i-svg-spinners:ring-resize w-3 h-3 my-0.5",
                                "aria-hidden": "true"
                              }))
                              : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(_unref(compactNumberFormatter).format(_unref(likesData)?.totalLikes ?? 0)), 1 /* TEXT */))
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["aria-label", "aria-pressed", "classicon"])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["text"]) ]) ]) ], 2 /* CLASS */), _createTextVNode("\n\n      "), _createTextVNode("\n      "), _createElementVNode("section", {
                class: _normalizeClass(_ctx.$style.areaDetails)
              }, [ _createElementVNode("div", { class: "mb-4" }, [ _createElementVNode("div", { class: "max-w-2xl" }, [ (_unref(pkgDescription)) ? (_openBlock(), _createElementBlock("p", {
                        key: 0,
                        class: "text-fg-muted text-base m-0"
                      }, [ _createElementVNode("span", { innerHTML: _unref(pkgDescription) }, null, 8 /* PROPS */, ["innerHTML"]) ])) : (_openBlock(), _createElementBlock("p", {
                        key: 1,
                        class: "text-fg-subtle text-base m-0 italic"
                      }, _toDisplayString(_ctx.$t('package.no_description')), 1 /* TEXT */)) ]), _createTextVNode("\n\n          " + "\n          "), _createElementVNode("ul", { class: "flex flex-wrap items-center gap-x-3 gap-y-1.5 sm:gap-4 list-none m-0 p-0 mt-3 text-sm" }, [ (repositoryUrl.value) ? (_openBlock(), _createElementBlock("li", { key: 0 }, [ _createVNode(_component_LinkBase, {
                          to: repositoryUrl.value,
                          classicon: repoProviderIcon.value
                        }, {
                          default: _withCtx(() => [
                            (_unref(repoRef))
                              ? (_openBlock(), _createElementBlock("span", { key: 0 }, [
                                _toDisplayString(_unref(repoRef).owner),
                                _hoisted_7,
                                _toDisplayString(_unref(repoRef).repo)
                              ]))
                              : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(_ctx.$t('package.links.repo')), 1 /* TEXT */))
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["to", "classicon"]) ])) : _createCommentVNode("v-if", true), (repositoryUrl.value && _unref(repoMeta) && _unref(starsLink)) ? (_openBlock(), _createElementBlock("li", { key: 0 }, [ _createVNode(_component_LinkBase, {
                          to: _unref(starsLink),
                          classicon: "i-lucide:star"
                        }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(compactNumberFormatter).format(_unref(stars))), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["to"]) ])) : _createCommentVNode("v-if", true), (_unref(forks) && _unref(forksLink)) ? (_openBlock(), _createElementBlock("li", { key: 0 }, [ _createVNode(_component_LinkBase, {
                          to: _unref(forksLink),
                          classicon: "i-lucide:git-fork"
                        }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(compactNumberFormatter).format(_unref(forks))), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["to"]) ])) : _createCommentVNode("v-if", true), _hoisted_8, (homepageUrl.value) ? (_openBlock(), _createElementBlock("li", { key: 0 }, [ _createVNode(_component_LinkBase, {
                          to: homepageUrl.value,
                          classicon: "i-lucide:link"
                        }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_ctx.$t('package.links.homepage')), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["to"]) ])) : _createCommentVNode("v-if", true), (displayVersion.value?.bugs?.url) ? (_openBlock(), _createElementBlock("li", { key: 0 }, [ _createVNode(_component_LinkBase, {
                          to: displayVersion.value.bugs.url,
                          classicon: "i-lucide:circle-alert"
                        }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_ctx.$t('package.links.issues')), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["to"]) ])) : _createCommentVNode("v-if", true), _createElementVNode("li", null, [ _createVNode(_component_LinkBase, {
                        to: `https://www.npmjs.com/package/${_unref(pkg).name}`,
                        title: _ctx.$t('common.view_on_npm'),
                        classicon: "i-simple-icons:npm"
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode("\n                npm\n              ")
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["to", "title"]) ]), (_unref(jsrInfo)?.exists && _unref(jsrInfo).url) ? (_openBlock(), _createElementBlock("li", { key: 0 }, [ _createVNode(_component_LinkBase, {
                          to: _unref(jsrInfo).url,
                          title: _ctx.$t('badges.jsr.title'),
                          classicon: "i-simple-icons:jsr"
                        }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_ctx.$t('package.links.jsr')), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["to", "title"]) ])) : _createCommentVNode("v-if", true), (fundingUrl.value) ? (_openBlock(), _createElementBlock("li", { key: 0 }, [ _createVNode(_component_LinkBase, {
                          to: fundingUrl.value,
                          classicon: "i-lucide:heart"
                        }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_ctx.$t('package.links.fund')), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["to"]) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n            " + "\n            "), (docsLink.value && displayVersion.value) ? (_openBlock(), _createElementBlock("li", {
                        key: 0,
                        class: "sm:hidden"
                      }, [ _createVNode(_component_LinkBase, {
                          to: docsLink.value,
                          classicon: "i-lucide:file-text"
                        }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_ctx.$t('package.links.docs')), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["to"]) ])) : _createCommentVNode("v-if", true), (_unref(resolvedVersion) && codeLink.value) ? (_openBlock(), _createElementBlock("li", {
                        key: 0,
                        class: "sm:hidden"
                      }, [ _createVNode(_component_LinkBase, {
                          to: codeLink.value,
                          classicon: "i-lucide:code"
                        }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_ctx.$t('package.links.code')), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["to"]) ])) : _createCommentVNode("v-if", true), _createElementVNode("li", { class: "sm:hidden" }, [ _createVNode(_component_LinkBase, {
                        to: { name: 'compare', query: { packages: _unref(pkg).name } },
                        classicon: "i-lucide:git-compare"
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_ctx.$t('package.links.compare')), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["to"]) ]), ( displayVersion.value && latestVersion.value && displayVersion.value.version !== latestVersion.value.version ) ? (_openBlock(), _createElementBlock("li", {
                        key: 0,
                        class: "sm:hidden"
                      }, [ _createVNode(_component_NuxtLink, {
                          to: _ctx.diffRoute(_unref(pkg).name, displayVersion.value.version, latestVersion.value.version),
                          class: "link-subtle font-mono text-sm inline-flex items-center gap-1.5"
                        }, {
                          default: _withCtx(() => [
                            _hoisted_9,
                            _createTextVNode("\n                "),
                            _createTextVNode(_toDisplayString(_ctx.$t('compare.compare_versions')), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["to"]) ])) : _createCommentVNode("v-if", true) ]) ]), (deprecationNotice.value) ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: "border border-red-700 dark:border-red-400 bg-red-400/10 rounded-lg px-3 py-2 text-base text-red-700 dark:text-red-400"
                  }, [ _createElementVNode("h2", _hoisted_10, _toDisplayString(deprecationNotice.value.type === 'package' ? _ctx.$t('package.deprecation.package') : _ctx.$t('package.deprecation.version')), 1 /* TEXT */), (_unref(deprecationNoticeMessage)) ? (_openBlock(), _createElementBlock("p", {
                        key: 0,
                        class: "text-base m-0"
                      }, [ _createElementVNode("span", { innerHTML: _unref(deprecationNoticeMessage) }, null, 8 /* PROPS */, ["innerHTML"]) ])) : (_openBlock(), _createElementBlock("p", {
                        key: 1,
                        class: "text-base m-0 italic"
                      }, _toDisplayString(_ctx.$t('package.deprecation.no_reason')), 1 /* TEXT */)) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("dl", { class: "grid grid-cols-2 sm:grid-cols-7 md:grid-cols-11 gap-3 sm:gap-4 py-4 sm:py-6 mt-4 sm:mt-6 border-t border-b border-border" }, [ _createElementVNode("div", { class: "space-y-1 sm:col-span-2" }, [ _createElementVNode("dt", _hoisted_11, _toDisplayString(_ctx.$t('package.stats.license')), 1 /* TEXT */), _createElementVNode("dd", { class: "font-mono text-sm text-fg" }, [ (_unref(pkg).license) ? (_openBlock(), _createBlock(_component_LicenseDisplay, {
                          key: 0,
                          license: _unref(pkg).license
                        }, null, 8 /* PROPS */, ["license"])) : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(_ctx.$t('package.license.none')), 1 /* TEXT */)) ]) ]), _createElementVNode("div", { class: "space-y-1 sm:col-span-2" }, [ _createElementVNode("dt", _hoisted_12, _toDisplayString(_ctx.$t('package.stats.deps')), 1 /* TEXT */), _createElementVNode("dd", { class: "font-mono text-sm text-fg flex items-center justify-start gap-2" }, [ _createElementVNode("span", { class: "flex items-center gap-1" }, [ _createElementVNode("span", _hoisted_13, _toDisplayString(_unref(numberFormatter).format(dependencyCount.value)), 1 /* TEXT */), _createTextVNode("\n\n                " + "\n                "), (dependencyCount.value > 0 && dependencyCount.value !== totalDepsCount.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _hoisted_14, _createVNode(_component_ClientOnly, null, {
                              fallback: _withCtx(() => [
                                _hoisted_16
                              ]),
                              default: _withCtx(() => [
                                (
                          _unref(vulnTreeStatus) === 'pending' ||
                          (_unref(installSizeStatus) === 'pending' && !_unref(vulnTree))
                        )
                                  ? (_openBlock(), _createElementBlock("span", {
                                    key: 0,
                                    class: "inline-flex items-center gap-1 text-fg-subtle"
                                  }, [
                                    _hoisted_15
                                  ]))
                                  : (totalDepsCount.value !== null)
                                    ? (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(_unref(numberFormatter).format(totalDepsCount.value)), 1 /* TEXT */))
                                  : (_openBlock(), _createElementBlock("span", {
                                    key: 2,
                                    class: "text-fg-subtle"
                                  }, "-"))
                              ]),
                              _: 1 /* STABLE */
                            }) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ]), (dependencyCount.value > 0) ? (_openBlock(), _createBlock(_component_ButtonGroup, {
                          key: 0,
                          class: "ms-auto"
                        }, {
                          default: _withCtx(() => [
                            _createVNode(_component_LinkBase, {
                              variant: "button-secondary",
                              size: "small",
                              to: `https://npmgraph.js.org/?q=${_unref(pkg).name}`,
                              title: _ctx.$t('package.stats.view_dependency_graph'),
                              classicon: "i-lucide:network -rotate-90"
                            }, {
                              default: _withCtx(() => [
                                _createElementVNode("span", _hoisted_17, _toDisplayString(_ctx.$t('package.stats.view_dependency_graph')), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            }, 8 /* PROPS */, ["to", "title"]),
                            _createVNode(_component_LinkBase, {
                              variant: "button-secondary",
                              size: "small",
                              to: `https://node-modules.dev/grid/depth#install=${_unref(pkg).name}${_unref(resolvedVersion) ? `@${_unref(resolvedVersion)}` : ''}`,
                              title: _ctx.$t('package.stats.inspect_dependency_tree'),
                              classicon: "i-lucide:table"
                            }, {
                              default: _withCtx(() => [
                                _createElementVNode("span", _hoisted_18, _toDisplayString(_ctx.$t('package.stats.inspect_dependency_tree')), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            }, 8 /* PROPS */, ["to", "title"])
                          ]),
                          _: 1 /* STABLE */
                        })) : _createCommentVNode("v-if", true) ]) ]), _createElementVNode("div", { class: "space-y-1 sm:col-span-3" }, [ _createElementVNode("dt", { class: "text-xs text-fg-subtle uppercase tracking-wider flex items-center gap-1" }, [ _createTextVNode(_toDisplayString(_ctx.$t('package.stats.install_size')) + "\n              ", 1 /* TEXT */), (sizeTooltip.value) ? (_openBlock(), _createBlock(_component_TooltipApp, {
                          key: 0,
                          text: sizeTooltip.value
                        }, {
                          default: _withCtx(() => [
                            _createElementVNode("span", {
                              tabindex: "0",
                              class: "inline-flex items-center justify-center min-w-6 min-h-6 -m-1 p-1 text-fg-subtle cursor-help focus-visible:outline-2 focus-visible:outline-accent/70 rounded"
                            }, [
                              _hoisted_19
                            ])
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["text"])) : _createCommentVNode("v-if", true) ]), _createElementVNode("dd", { class: "font-mono text-sm text-fg" }, [ _createElementVNode("span", {
                        class: "text-fg-muted",
                        dir: "ltr"
                      }, [ (displayVersion.value?.dist?.unpackedSize) ? (_openBlock(), _createElementBlock("span", { key: 0 }, _toDisplayString(_unref(bytesFormatter).format(displayVersion.value.dist.unpackedSize)), 1 /* TEXT */)) : (_openBlock(), _createElementBlock("span", { key: 1 }, "-")) ]), _createTextVNode("\n\n              " + "\n              "), (displayVersion.value?.dist?.unpackedSize !== _unref(installSize)?.totalSize) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _hoisted_20, (_unref(installSizeStatus) === 'pending') ? (_openBlock(), _createElementBlock("span", {
                              key: 0,
                              class: "inline-flex items-center gap-1 text-fg-subtle"
                            }, [ _hoisted_21 ])) : (_unref(installSize)?.totalSize) ? (_openBlock(), _createElementBlock("span", {
                                key: 1,
                                dir: "ltr"
                              }, _toDisplayString(_unref(bytesFormatter).format(_unref(installSize).totalSize)), 1 /* TEXT */)) : (_openBlock(), _createElementBlock("span", {
                              key: 2,
                              class: "text-fg-subtle"
                            }, "-")) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ]) ]), _createTextVNode("\n\n          " + "\n          "), _createElementVNode("div", { class: "space-y-1 sm:col-span-2" }, [ _createElementVNode("dt", _hoisted_22, _toDisplayString(_ctx.$t('package.stats.vulns')), 1 /* TEXT */), _createElementVNode("dd", { class: "font-mono text-sm text-fg" }, [ (_unref(vulnTreeStatus) === 'pending' || _unref(vulnTreeStatus) === 'idle') ? (_openBlock(), _createElementBlock("span", {
                          key: 0,
                          class: "inline-flex items-center gap-1 text-fg-subtle"
                        }, [ _hoisted_23 ])) : (_unref(vulnTreeStatus) === 'success') ? (_openBlock(), _createElementBlock("span", { key: 1 }, [ (hasVulnerabilities.value) ? (_openBlock(), _createElementBlock("span", {
                                key: 0,
                                class: "text-amber-700 dark:text-amber-500"
                              }, _toDisplayString(_unref(numberFormatter).format(vulnCount.value)), 1 /* TEXT */)) : (_openBlock(), _createElementBlock("span", {
                                key: 1,
                                class: "inline-flex items-center gap-1 text-fg-muted"
                              }, [ _hoisted_24, _createTextVNode("\n                  "), _toDisplayString(_unref(numberFormatter).format(0)) ])) ])) : (_openBlock(), _createElementBlock("span", {
                          key: 2,
                          class: "text-fg-subtle"
                        }, "-")) ]) ]), (_unref(resolvedVersion) && _unref(pkg).time?.[_unref(resolvedVersion)]) ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "space-y-1 sm:col-span-2"
                    }, [ _createElementVNode("dt", {
                        class: "text-xs text-fg-subtle uppercase tracking-wider",
                        title: 
                  _ctx.$t('package.stats.published_tooltip', {
                    package: _unref(pkg).name,
                    version: _unref(resolvedVersion),
                  })
              
                      }, _toDisplayString(_ctx.$t('package.stats.published')), 9 /* TEXT, PROPS */, ["title"]), _createElementVNode("dd", { class: "font-mono text-sm text-fg" }, [ _createVNode(_component_DateTime, {
                          datetime: _unref(pkg).time[_unref(resolvedVersion)],
                          "date-style": "medium"
                        }, null, 8 /* PROPS */, ["datetime"]) ]) ])) : _createCommentVNode("v-if", true) ]), _createTextVNode("\n\n        " + "\n        "), _createVNode(_component_ClientOnly, null, {
                  default: _withCtx(() => [
                    _createVNode(_component_PackageSkillsModal, {
                      skills: _unref(skillsData)?.skills ?? [],
                      "package-name": _unref(pkg).name,
                      version: _unref(resolvedVersion) || undefined
                    }, null, 8 /* PROPS */, ["skills", "package-name", "version"])
                  ]),
                  _: 1 /* STABLE */
                }) ]), _createTextVNode("\n\n      "), _createTextVNode("\n      "), (isBinaryOnly.value) ? (_openBlock(), _createElementBlock("section", {
                  key: 0,
                  class: _normalizeClass(["scroll-mt-20", _ctx.$style.areaInstall])
                }, [ _createElementVNode("div", { class: "flex flex-wrap items-center justify-between mb-3" }, [ _createElementVNode("h2", _hoisted_25, _toDisplayString(_ctx.$t('package.run.title')), 1 /* TEXT */), _createTextVNode("\n          " + "\n          "), _createVNode(_component_PackageManagerSelect) ]), _createElementVNode("div", null, [ _createVNode(_component_TerminalExecute, {
                      "package-name": _unref(pkg).name,
                      "jsr-info": _unref(jsrInfo),
                      "is-create-package": isCreatePkg.value
                    }, null, 8 /* PROPS */, ["package-name", "jsr-info", "is-create-package"]) ]) ])) : (_openBlock(), _createElementBlock("section", {
                  key: 1,
                  id: "get-started",
                  class: _normalizeClass(["scroll-mt-20", _ctx.$style.areaInstall])
                }, [ _createElementVNode("div", { class: "flex flex-wrap items-center justify-between mb-3" }, [ _createElementVNode("h2", {
                      id: "get-started-heading",
                      class: "group text-xs text-fg-subtle uppercase tracking-wider"
                    }, [ _createVNode(_component_LinkBase, { to: "#get-started" }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_ctx.$t('package.get_started.title')), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }) ]), _createTextVNode("\n          " + "\n          "), _createVNode(_component_PackageManagerSelect) ]), _createElementVNode("div", null, [ (publishSecurityDowngrade.value) ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        role: "alert",
                        class: "mb-4 rounded-lg border border-amber-600/40 bg-amber-500/10 px-4 py-3 text-amber-700 dark:text-amber-400"
                      }, [ _createElementVNode("h3", { class: "m-0 flex items-center gap-2 font-mono text-sm font-medium" }, [ _hoisted_26, _createTextVNode("\n              " + _toDisplayString(_ctx.$t('package.security_downgrade.title')), 1 /* TEXT */) ]), _createElementVNode("p", { class: "mt-2 mb-0 text-sm" }, [ ( publishSecurityDowngrade.value.downgradedTrustLevel === 'none' && publishSecurityDowngrade.value.trustedTrustLevel === 'provenance' ) ? (_openBlock(), _createBlock(_component_i18n_t, {
                              key: 0,
                              keypath: "package.security_downgrade.description_to_none_provenance",
                              tag: "span",
                              scope: "global"
                            }, {
                              provenance: _withCtx(() => [
                                _createElementVNode("a", {
                                  href: "https://docs.npmjs.com/generating-provenance-statements",
                                  target: "_blank",
                                  rel: "noopener noreferrer",
                                  class: "inline-flex items-center gap-1 rounded-sm underline underline-offset-4 decoration-amber-600/60 dark:decoration-amber-400/50 hover:decoration-fg focus-visible:decoration-fg focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/70 transition-colors"
                                }, [
                                  _createTextVNode(_toDisplayString(_ctx.$t('package.security_downgrade.provenance_link_text')), 1 /* TEXT */),
                                  _hoisted_27
                                ])
                              ]),
                              _: 1 /* STABLE */
                            })) : ( publishSecurityDowngrade.value.downgradedTrustLevel === 'none' && publishSecurityDowngrade.value.trustedTrustLevel === 'trustedPublisher' ) ? (_openBlock(), _createBlock(_component_i18n_t, {
                                key: 1,
                                keypath: "package.security_downgrade.description_to_none_trustedPublisher",
                                tag: "span",
                                scope: "global"
                              }, {
                                trustedPublishing: _withCtx(() => [
                                  _createElementVNode("a", {
                                    href: "https://docs.npmjs.com/trusted-publishers",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    class: "inline-flex items-center gap-1 rounded-sm underline underline-offset-4 decoration-amber-600/60 dark:decoration-amber-400/50 hover:decoration-fg focus-visible:decoration-fg focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/70 transition-colors"
                                  }, [
                                    _createTextVNode(_toDisplayString(_ctx.$t('package.security_downgrade.trusted_publishing_link_text')), 1 /* TEXT */),
                                    _hoisted_28
                                  ])
                                ]),
                                _: 1 /* STABLE */
                              })) : ( publishSecurityDowngrade.value.downgradedTrustLevel === 'provenance' && publishSecurityDowngrade.value.trustedTrustLevel === 'trustedPublisher' ) ? (_openBlock(), _createBlock(_component_i18n_t, {
                                key: 2,
                                keypath: "package.security_downgrade.description_to_provenance_trustedPublisher",
                                tag: "span",
                                scope: "global"
                              }, {
                                provenance: _withCtx(() => [
                                  _createElementVNode("a", {
                                    href: "https://docs.npmjs.com/generating-provenance-statements",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    class: "inline-flex items-center gap-1 rounded-sm underline underline-offset-4 decoration-amber-600/60 dark:decoration-amber-400/50 hover:decoration-fg focus-visible:decoration-fg focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/70 transition-colors"
                                  }, [
                                    _createTextVNode(_toDisplayString(_ctx.$t('package.security_downgrade.provenance_link_text')), 1 /* TEXT */),
                                    _hoisted_29
                                  ])
                                ]),
                                trustedPublishing: _withCtx(() => [
                                  _createElementVNode("a", {
                                    href: "https://docs.npmjs.com/trusted-publishers",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    class: "inline-flex items-center gap-1 rounded-sm underline underline-offset-4 decoration-amber-600/60 dark:decoration-amber-400/50 hover:decoration-fg focus-visible:decoration-fg focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/70 transition-colors"
                                  }, [
                                    _createTextVNode(_toDisplayString(_ctx.$t('package.security_downgrade.trusted_publishing_link_text')), 1 /* TEXT */),
                                    _hoisted_30
                                  ])
                                ]),
                                _: 1 /* STABLE */
                              })) : _createCommentVNode("v-if", true), _createTextVNode("\n              " + _toDisplayString(' ') + "\n              ", 1 /* TEXT */), (downgradeFallbackInstallText.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _toDisplayString(downgradeFallbackInstallText.value) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ]) ])) : _createCommentVNode("v-if", true), _createVNode(_component_TerminalInstall, {
                      "package-name": _unref(pkg).name,
                      "requested-version": _unref(requestedVersion),
                      "install-version-override": installVersionOverride.value,
                      "jsr-info": _unref(jsrInfo),
                      "dev-dependency-suggestion": _unref(packageAnalysis)?.devDependencySuggestion,
                      "types-package-name": typesPackageName.value,
                      "executable-info": executableInfo.value,
                      "create-package-info": createPackageInfo.value
                    }, null, 8 /* PROPS */, ["package-name", "requested-version", "install-version-override", "jsr-info", "dev-dependency-suggestion", "types-package-name", "executable-info", "create-package-info"]) ]) ])), _createTextVNode("\n\n      "), _createTextVNode("\n      "), _createElementVNode("div", {
                class: _normalizeClass(["space-y-6", _ctx.$style.areaVulns])
              }, [ (_unref(moduleReplacement)) ? (_openBlock(), _createBlock(_component_PackageReplacement, {
                    key: 0,
                    replacement: _unref(moduleReplacement)
                  }, null, 8 /* PROPS */, ["replacement"])) : _createCommentVNode("v-if", true), _createTextVNode("\n        " + "\n        "), (_unref(sizeDiff)) ? (_openBlock(), _createBlock(_component_PackageSizeIncrease, {
                    key: 0,
                    diff: _unref(sizeDiff)
                  }, null, 8 /* PROPS */, ["diff"])) : _createCommentVNode("v-if", true), _createTextVNode("\n        " + "\n        "), _createVNode(_component_ClientOnly, null, {
                  default: _withCtx(() => [
                    (_unref(resolvedVersion))
                      ? (_openBlock(), _createBlock(_component_PackageVulnerabilityTree, {
                        key: 0,
                        "package-name": _unref(pkg).name,
                        version: _unref(resolvedVersion)
                      }, null, 8 /* PROPS */, ["package-name", "version"]))
                      : _createCommentVNode("v-if", true),
                    (_unref(resolvedVersion))
                      ? (_openBlock(), _createBlock(_component_PackageDeprecatedTree, {
                        key: 0,
                        "package-name": _unref(pkg).name,
                        version: _unref(resolvedVersion),
                        class: "mt-3"
                      }, null, 8 /* PROPS */, ["package-name", "version"]))
                      : _createCommentVNode("v-if", true)
                  ]),
                  _: 1 /* STABLE */
                }) ]), _createTextVNode("\n\n      "), _createTextVNode("\n      "), _createElementVNode("section", {
                id: "readme",
                class: _normalizeClass(["min-w-0 scroll-mt-20", _ctx.$style.areaReadme])
              }, [ _createElementVNode("div", { class: "flex flex-wrap items-center justify-between mb-3 px-1" }, [ _createElementVNode("h2", {
                    id: "readme-heading",
                    class: "group text-xs text-fg-subtle uppercase tracking-wider"
                  }, [ _createVNode(_component_LinkBase, { to: "#readme" }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_ctx.$t('package.readme.title')), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }) ]), _createElementVNode("div", { class: "flex gap-2" }, [ (_unref(readmeData)?.mdExists) ? (_openBlock(), _createBlock(_component_TooltipApp, {
                        key: 0,
                        text: _ctx.$t('package.readme.copy_as_markdown'),
                        position: "bottom"
                      }, {
                        default: _withCtx(() => [
                          _createVNode(_component_ButtonBase, {
                            onMouseenter: prefetchReadmeMarkdown,
                            onFocus: prefetchReadmeMarkdown,
                            onClick: _cache[4] || (_cache[4] = ($event: any) => (copyReadmeHandler())),
                            "aria-pressed": _unref(copiedReadme),
                            "aria-label": 
                    _unref(copiedReadme) ? _ctx.$t('common.copied') : _ctx.$t('package.readme.copy_as_markdown')
                  ,
                            classicon: _unref(copiedReadme) ? 'i-lucide:check' : 'i-simple-icons:markdown'
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(copiedReadme) ? _ctx.$t('common.copied') : _ctx.$t('common.copy')), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["aria-pressed", "aria-label", "classicon"])
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["text"])) : _createCommentVNode("v-if", true), (_unref(readmeData)?.toc && _unref(readmeData).toc.length > 1) ? (_openBlock(), _createBlock(_component_ReadmeTocDropdown, {
                        key: 0,
                        toc: _unref(readmeData).toc,
                        "active-id": _unref(activeTocId)
                      }, null, 8 /* PROPS */, ["toc", "active-id"])) : _createCommentVNode("v-if", true) ]) ]), _createTextVNode("\n\n        " + "\n        "), (_unref(readmeData)?.html) ? (_openBlock(), _createBlock(_component_Readme, {
                    key: 0,
                    html: _unref(readmeData).html
                  }, null, 8 /* PROPS */, ["html"])) : (_openBlock(), _createElementBlock("p", {
                    key: 1,
                    class: "text-fg-muted italic"
                  }, [ _toDisplayString(_ctx.$t('package.readme.no_readme')), _createTextVNode("\n          "), (repositoryUrl.value) ? (_openBlock(), _createElementBlock("a", {
                        key: 0,
                        href: repositoryUrl.value,
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "link text-fg underline underline-offset-4 decoration-fg-subtle hover:(decoration-fg text-fg) transition-colors duration-200"
                      }, _toDisplayString(_ctx.$t('package.readme.view_on_github')), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ])), (hasProvenance(displayVersion.value) && _unref(isMounted)) ? (_openBlock(), _createElementBlock("section", {
                    key: 0,
                    id: "provenance",
                    class: "scroll-mt-20"
                  }, [ (_unref(provenanceStatus) === 'pending') ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        class: "mt-8 flex items-center gap-2 text-fg-subtle text-sm"
                      }, [ _hoisted_31, _createElementVNode("span", null, _toDisplayString(_ctx.$t('package.provenance_section.title')) + "…", 1 /* TEXT */) ])) : (_unref(provenanceData)) ? (_openBlock(), _createBlock(_component_PackageProvenanceSection, {
                          key: 1,
                          details: _unref(provenanceData),
                          class: "mt-8"
                        }, null, 8 /* PROPS */, ["details"])) : (_unref(provenanceStatus) === 'error') ? (_openBlock(), _createElementBlock("div", {
                          key: 2,
                          class: "mt-8 flex items-center gap-2 text-fg-subtle text-sm"
                        }, [ _hoisted_32, _createElementVNode("span", null, _toDisplayString(_ctx.$t('package.provenance_section.error_loading')), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n          "), _createTextVNode("\n          ") ])) : _createCommentVNode("v-if", true) ]), _createVNode(_component_PackageSidebar, {
                class: _normalizeClass(_ctx.$style.areaSidebar)
              }, {
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "flex flex-col gap-4 sm:gap-6 xl:(pt-2)" }, [
                    _createVNode(_component_ClientOnly, null, {
                      fallback: _withCtx(() => [
                      ]),
                      default: _withCtx(() => [
                        _createVNode(_component_PackageAccessControls, { "package-name": _unref(pkg).name }, null, 8 /* PROPS */, ["package-name"])
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createTextVNode("\n\n          " + "\n          "),
                    _createVNode(_component_ClientOnly, null, {
                      fallback: _withCtx(() => [
                      ]),
                      default: _withCtx(() => [
                        (_unref(skillsData)?.skills?.length)
                          ? (_openBlock(), _createBlock(_component_PackageSkillsCard, {
                            key: 0,
                            skills: _unref(skillsData).skills,
                            "package-name": _unref(pkg).name,
                            version: _unref(resolvedVersion) || undefined
                          }, null, 8 /* PROPS */, ["skills", "package-name", "version"]))
                          : _createCommentVNode("v-if", true)
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createTextVNode("\n\n          " + "\n          "),
                    _createVNode(_component_PackageWeeklyDownloadStats, {
                      packageName: _unref(packageName),
                      createdIso: _unref(pkg)?.time?.created ?? null,
                      repoRef: _unref(repoRef)
                    }, null, 8 /* PROPS */, ["packageName", "createdIso", "repoRef"]),
                    _createTextVNode("\n\n          " + "\n          "),
                    (playgroundLinks.value.length)
                      ? (_openBlock(), _createBlock(_component_PackagePlaygrounds, {
                        key: 0,
                        links: playgroundLinks.value
                      }, null, 8 /* PROPS */, ["links"]))
                      : _createCommentVNode("v-if", true),
                    _createVNode(_component_PackageCompatibility, { engines: displayVersion.value?.engines }, null, 8 /* PROPS */, ["engines"]),
                    _createTextVNode("\n\n          " + "\n          "),
                    (_unref(pkg).versions && Object.keys(_unref(pkg).versions).length > 0)
                      ? (_openBlock(), _createBlock(_component_PackageVersions, {
                        key: 0,
                        "package-name": _unref(pkg).name,
                        versions: _unref(pkg).versions,
                        "dist-tags": _unref(pkg)['dist-tags'] ?? {},
                        time: _unref(pkg).time,
                        "selected-version": _unref(resolvedVersion) ?? _unref(pkg)['dist-tags']?.['latest']
                      }, null, 8 /* PROPS */, ["package-name", "versions", "dist-tags", "time", "selected-version"]))
                      : _createCommentVNode("v-if", true),
                    _createTextVNode("\n\n          " + "\n          "),
                    (displayVersion.value?.installScripts)
                      ? (_openBlock(), _createBlock(_component_PackageInstallScripts, {
                        key: 0,
                        "package-name": _unref(pkg).name,
                        version: displayVersion.value.version,
                        "install-scripts": displayVersion.value.installScripts
                      }, null, 8 /* PROPS */, ["package-name", "version", "install-scripts"]))
                      : _createCommentVNode("v-if", true),
                    _createTextVNode("\n\n          " + "\n          "),
                    (hasDependencies.value && _unref(resolvedVersion) && displayVersion.value)
                      ? (_openBlock(), _createBlock(_component_PackageDependencies, {
                        key: 0,
                        "package-name": _unref(pkg).name,
                        version: _unref(resolvedVersion),
                        dependencies: displayVersion.value.dependencies,
                        "peer-dependencies": displayVersion.value.peerDependencies,
                        "peer-dependencies-meta": displayVersion.value.peerDependenciesMeta,
                        "optional-dependencies": displayVersion.value.optionalDependencies
                      }, null, 8 /* PROPS */, ["package-name", "version", "dependencies", "peer-dependencies", "peer-dependencies-meta", "optional-dependencies"]))
                      : _createCommentVNode("v-if", true),
                    _createTextVNode("\n\n          " + "\n          "),
                    _createVNode(_component_PackageKeywords, { keywords: displayVersion.value?.keywords }, null, 8 /* PROPS */, ["keywords"]),
                    _createTextVNode("\n\n          " + "\n          "),
                    _createVNode(_component_PackageMaintainers, {
                      "package-name": _unref(pkg).name,
                      maintainers: _unref(pkg).maintainers
                    }, null, 8 /* PROPS */, ["package-name", "maintainers"])
                  ])
                ]),
                _: 1 /* STABLE */
              }) ])) : (_unref(status) === 'error') ? (_openBlock(), _createElementBlock("div", {
              key: 3,
              role: "alert",
              class: "flex flex-col items-center py-20 text-center"
            }, [ _createElementVNode("h1", _hoisted_33, _toDisplayString(_ctx.$t('package.not_found')), 1 /* TEXT */), _createElementVNode("p", _hoisted_34, _toDisplayString(_unref(error)?.message ?? _ctx.$t('package.not_found_message')), 1 /* TEXT */), _createVNode(_component_LinkBase, {
                variant: "button-secondary",
                to: { name: 'index' }
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_ctx.$t('common.go_back_home')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to"]) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    " + "\n    " + "\n\n    " + "\n    ") ]) ], 64 /* STABLE_FRAGMENT */))
}
}

})
