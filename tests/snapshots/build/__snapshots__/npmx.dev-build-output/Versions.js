import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "sr-only" }
const _hoisted_2 = { dir: "ltr", class: "block truncate" }
const _hoisted_3 = { dir: "ltr", class: "block truncate" }
const _hoisted_4 = { dir: "ltr", class: "block truncate" }
const _hoisted_5 = { dir: "ltr", class: "block truncate" }
const _hoisted_6 = { dir: "ltr", class: "block truncate" }
const _hoisted_7 = { dir: "ltr", class: "block truncate" }
import type { PackageVersionInfo, SlimVersion } from '#shared/types'
import { compare, validRange } from 'semver'
import type { RouteLocationRaw } from 'vue-router'
import { fetchAllPackageVersions } from '~/utils/npm/api'
import { NPMX_DOCS_SITE } from '#shared/utils/constants'
import { buildVersionToTagsMap, filterExcludedTags, filterVersions, getPrereleaseChannel, getVersionGroupKey, getVersionGroupLabel, isSameVersionGroup } from '~/utils/versions'

interface VersionDisplay {
  version: string
  time?: string
  tags?: string[]
  hasProvenance: boolean
  deprecated?: string
}
const QUERY_MODAL_VALUE = 'versions'
const MAX_VISIBLE_TAGS = 10

export default /*@__PURE__*/_defineComponent({
  __name: 'Versions',
  props: {
    packageName: { type: String, required: true },
    versions: { type: null, required: true },
    distTags: { type: null, required: true },
    time: { type: null, required: true },
    selectedVersion: { type: String, required: false }
  },
  setup(__props: any) {

const props = __props
const chartModal = useModal('chart-modal')
const hasDistributionModalTransitioned = shallowRef(false)
const isDistributionModalOpen = shallowRef(false)
let distributionModalFallbackTimer: ReturnType<typeof setTimeout> | null = null
function clearDistributionModalFallbackTimer() {
  if (distributionModalFallbackTimer) {
    clearTimeout(distributionModalFallbackTimer)
    distributionModalFallbackTimer = null
  }
}
const router = useRouter()
const route = useRoute()
async function openDistributionModal() {
  isDistributionModalOpen.value = true
  hasDistributionModalTransitioned.value = false
  // ensure the component renders before opening the dialog
  await nextTick()
  chartModal.open()
  await router.replace({
    query: {
      ...route.query,
      modal: QUERY_MODAL_VALUE,
    },
  })
  // Fallback: Force mount if transition event doesn't fire
  clearDistributionModalFallbackTimer()
  distributionModalFallbackTimer = setTimeout(() => {
    if (!hasDistributionModalTransitioned.value) {
      hasDistributionModalTransitioned.value = true
    }
  }, 500)
}
function closeDistributionModal() {
  isDistributionModalOpen.value = false
  router.replace({
    query: {
      ...route.query,
      modal: undefined,
      grouping: undefined,
      recent: undefined,
      lowUsage: undefined,
    },
  })
  hasDistributionModalTransitioned.value = false
  clearDistributionModalFallbackTimer()
}
onMounted(() => {
  if (route.query.modal === QUERY_MODAL_VALUE) {
    openDistributionModal()
  }
})
function handleDistributionModalTransitioned() {
  hasDistributionModalTransitioned.value = true
  clearDistributionModalFallbackTimer()
}
/** Maximum number of dist-tag rows to show before collapsing into "Other versions" */
/** A version with its metadata */
// Build route object for package version link
function versionRoute(version: string): RouteLocationRaw {
  return packageRoute(props.packageName, version)
}
// Version to tags lookup (supports multiple tags per version)
const versionToTags = computed(() => buildVersionToTagsMap(props.distTags))
const effectiveCurrentVersion = computed(
  () => props.selectedVersion ?? props.distTags.latest ?? undefined,
)
// Semver range filter
const semverFilter = ref('')
// Collect all known versions: initial props + dynamically loaded ones
const allKnownVersions = computed(() => {
  const versions = new Set(Object.keys(props.versions))
  for (const versionList of tagVersions.value.values()) {
    for (const v of versionList) {
      versions.add(v.version)
    }
  }
  for (const group of otherMajorGroups.value) {
    for (const v of group.versions) {
      versions.add(v.version)
    }
  }
  return [...versions]
})
const filteredVersionSet = computed(() =>
  filterVersions(allKnownVersions.value, semverFilter.value),
)
const isFilterActive = computed(() => semverFilter.value.trim() !== '')
const isInvalidRange = computed(
  () => isFilterActive.value && validRange(semverFilter.value.trim()) === null,
)
// All tag rows derived from props (SSR-safe)
// Deduplicates so each version appears only once, with all its tags
const allTagRows = computed(() => {
  // Group tags by version with their metadata
  const versionMap = new Map<string, { tags: string[]; versionData: SlimVersion | undefined }>()
  for (const [tag, version] of Object.entries(props.distTags)) {
    const existing = versionMap.get(version)
    if (existing) {
      existing.tags.push(tag)
    } else {
      versionMap.set(version, {
        tags: [tag],
        versionData: props.versions[version],
      })
    }
  }

  // Sort tags within each version: 'latest' first, then alphabetically
  for (const entry of versionMap.values()) {
    entry.tags.sort((a, b) => {
      if (a === 'latest') return -1
      if (b === 'latest') return 1
      return a.localeCompare(b)
    })
  }

  // Convert to rows, using the first (most important) tag as the primary
  return Array.from(versionMap.entries())
    .map(([version, { tags, versionData }]) => ({
      id: `version:${version}`,
      tag: tags[0]!, // Primary tag for expand/collapse logic
      tags, // All tags for this version
      primaryVersion: {
        version,
        time: props.time[version],
        tags,
        hasProvenance: versionData?.hasProvenance,
        deprecated: versionData?.deprecated,
      } as VersionDisplay,
    }))
    .sort((a, b) => compare(b.primaryVersion.version, a.primaryVersion.version))
})
// Check if the whole package is deprecated (latest version is deprecated)
const isPackageDeprecated = computed(() => {
  const latestVersion = props.distTags.latest
  if (!latestVersion) return false
  return !!props.versions[latestVersion]?.deprecated
})
// Visible tag rows: limited to MAX_VISIBLE_TAGS
// If package is NOT deprecated, filter out deprecated tags from visible list
// When semver filter is active, also filter by matching version
const visibleTagRows = computed(() => {
  const rowsMaybeFilteredForDeprecation = isPackageDeprecated.value
    ? allTagRows.value
    : allTagRows.value.filter(row => !row.primaryVersion.deprecated)
  const rows = isFilterActive.value
    ? rowsMaybeFilteredForDeprecation.filter(row =>
        filteredVersionSet.value.has(row.primaryVersion.version),
      )
    : rowsMaybeFilteredForDeprecation
  const first = rows.slice(0, MAX_VISIBLE_TAGS)
  const latestTagRow = rows.find(row => row.tag === 'latest')
  // Ensure 'latest' tag is always included (at the end) if not already present
  if (latestTagRow && !first.includes(latestTagRow)) {
    first.pop()
    first.push(latestTagRow)
  }
  return first
})
// Hidden tag rows (all other tags) - shown in "Other versions"
// When semver filter is active, also filter by matching version
const hiddenTagRows = computed(() => {
  const hiddenRows = allTagRows.value.filter(row => !visibleTagRows.value.includes(row))
  const rows = isFilterActive.value
    ? hiddenRows.filter(row => filteredVersionSet.value.has(row.primaryVersion.version))
    : hiddenRows
  return rows
})
// Client-side state for expansion and loaded versions
const expandedTags = ref<Set<string>>(new Set())
const tagVersions = ref<Map<string, VersionDisplay[]>>(new Map())
const loadingTags = ref<Set<string>>(new Set())
const otherVersionsExpanded = shallowRef(false)
const expandedMajorGroups = ref<Set<string>>(new Set())
const otherMajorGroups = shallowRef<
  Array<{ groupKey: string; label: string; versions: VersionDisplay[] }>
>([])
const otherVersionsLoading = shallowRef(false)
// Filtered major groups (applies semver filter when active)
const filteredOtherMajorGroups = computed(() => {
  if (!isFilterActive.value) return otherMajorGroups.value
  return otherMajorGroups.value
    .map(group => ({
      ...group,
      versions: group.versions.filter(v => filteredVersionSet.value.has(v.version)),
    }))
    .filter(group => group.versions.length > 0)
})
// Whether the filter is active but nothing matches anywhere
const hasNoFilterMatches = computed(() => {
  if (!isFilterActive.value) return false
  return (
    visibleTagRows.value.length === 0 &&
    hiddenTagRows.value.length === 0 &&
    filteredOtherMajorGroups.value.length === 0
  )
})
// Cached full version list (local to component instance)
const allVersionsCache = shallowRef<PackageVersionInfo[] | null>(null)
const loadingVersions = shallowRef(false)
const hasLoadedAll = shallowRef(false)
// Load all versions using shared function
async function loadAllVersions(): Promise<PackageVersionInfo[]> {
  if (allVersionsCache.value) return allVersionsCache.value
  if (loadingVersions.value) {
    await new Promise<void>(resolve => {
      const unwatch = watch(allVersionsCache, val => {
        if (val) {
          unwatch()
          resolve()
        }
      })
    })
    return allVersionsCache.value!
  }
  loadingVersions.value = true
  try {
    const versions = await fetchAllPackageVersions(props.packageName)
    allVersionsCache.value = versions
    hasLoadedAll.value = true
    return versions
  } finally {
    loadingVersions.value = false
  }
}
// Process loaded versions
function processLoadedVersions(allVersions: PackageVersionInfo[]) {
  const distTags = props.distTags
  // For each tag, find versions in its channel (same major + same prerelease channel)
  const claimedVersions = new Set<string>()
  for (const row of allTagRows.value) {
    const tagVersion = distTags[row.tag]
    if (!tagVersion) continue
    const tagChannel = getPrereleaseChannel(tagVersion)
    // Find all versions in the same version group + prerelease channel
    // For 0.x versions, this means same major.minor; for 1.x+, same major
    const channelVersions = allVersions
      .filter(v => {
        const vChannel = getPrereleaseChannel(v.version)
        return isSameVersionGroup(v.version, tagVersion) && vChannel === tagChannel
      })
      .sort((a, b) => compare(b.version, a.version))
      .map(v => ({
        version: v.version,
        time: v.time,
        tags: versionToTags.value.get(v.version),
        hasProvenance: v.hasProvenance,
        deprecated: v.deprecated,
      }))
    tagVersions.value.set(row.tag, channelVersions)
    for (const v of channelVersions) {
      claimedVersions.add(v.version)
    }
  }
  // Group unclaimed versions by version group key
  // For 0.x versions, group by major.minor (e.g., "0.9", "0.10")
  // For 1.x+, group by major (e.g., "1", "2")
  const byGroupKey = new Map<string, VersionDisplay[]>()
  for (const v of allVersions) {
    if (claimedVersions.has(v.version)) continue
    const groupKey = getVersionGroupKey(v.version)
    if (!byGroupKey.has(groupKey)) {
      byGroupKey.set(groupKey, [])
    }
    byGroupKey.get(groupKey)!.push({
      version: v.version,
      time: v.time,
      tags: versionToTags.value.get(v.version),
      hasProvenance: v.hasProvenance,
      deprecated: v.deprecated,
    })
  }
  // Sort within each group
  for (const versions of byGroupKey.values()) {
    versions.sort((a, b) => compare(b.version, a.version))
  }
  // Build groups sorted by group key descending
  // Sort: "2", "1", "0.10", "0.9" (numerically descending)
  const sortedGroupKeys = Array.from(byGroupKey.keys()).sort((a, b) => {
    const [aMajor, aMinor] = a.split('.').map(Number)
    const [bMajor, bMinor] = b.split('.').map(Number)
    if (aMajor !== bMajor) return (bMajor ?? 0) - (aMajor ?? 0)
    return (bMinor ?? -1) - (aMinor ?? -1)
  })
  otherMajorGroups.value = sortedGroupKeys.map(groupKey => ({
    groupKey,
    label: getVersionGroupLabel(groupKey),
    versions: byGroupKey.get(groupKey)!,
  }))
  expandedMajorGroups.value.clear()
}
// Expand a tag row
async function expandTagRow(tag: string) {
  if (expandedTags.value.has(tag)) {
    expandedTags.value.delete(tag)
    expandedTags.value = new Set(expandedTags.value)
    return
  }
  if (!hasLoadedAll.value) {
    loadingTags.value.add(tag)
    loadingTags.value = new Set(loadingTags.value)
    try {
      const allVersions = await loadAllVersions()
      processLoadedVersions(allVersions)
    } catch (error) {
      // oxlint-disable-next-line no-console -- error logging
      console.error('Failed to load versions:', error)
    } finally {
      loadingTags.value.delete(tag)
      loadingTags.value = new Set(loadingTags.value)
    }
  }
  expandedTags.value.add(tag)
  expandedTags.value = new Set(expandedTags.value)
}
// Expand "Other versions" section
async function expandOtherVersions() {
  if (otherVersionsExpanded.value) {
    otherVersionsExpanded.value = false
    return
  }
  if (!hasLoadedAll.value) {
    otherVersionsLoading.value = true
    try {
      const allVersions = await loadAllVersions()
      processLoadedVersions(allVersions)
    } catch (error) {
      // oxlint-disable-next-line no-console -- error logging
      console.error('Failed to load versions:', error)
    } finally {
      otherVersionsLoading.value = false
    }
  }
  otherVersionsExpanded.value = true
}
// Toggle a version group
function toggleMajorGroup(groupKey: string) {
  if (expandedMajorGroups.value.has(groupKey)) {
    expandedMajorGroups.value.delete(groupKey)
  } else {
    expandedMajorGroups.value.add(groupKey)
  }
}
// Get versions for a tag (from loaded data or empty)
function getTagVersions(tag: string): VersionDisplay[] {
  return tagVersions.value.get(tag) ?? []
}
// Get filtered versions for a tag (applies semver filter when active)
function getFilteredTagVersions(tag: string): VersionDisplay[] {
  const versions = getTagVersions(tag)
  if (!isFilterActive.value) return versions
  return versions.filter(v => filteredVersionSet.value.has(v.version))
}
function findClaimingTag(version: string): string | null {
  const versionChannel = getPrereleaseChannel(version)
  // First matching tag claims the version
  for (const row of allTagRows.value) {
    const tagVersion = props.distTags[row.tag]
    if (!tagVersion) continue
    const tagChannel = getPrereleaseChannel(tagVersion)
    if (isSameVersionGroup(version, tagVersion) && versionChannel === tagChannel) {
      return row.tag
    }
  }
  return null
}
// Whether this row should be highlighted for the current version
function rowContainsCurrentVersion(row: (typeof visibleTagRows.value)[0]): boolean {
  if (!effectiveCurrentVersion.value) return false
  if (row.primaryVersion.version === effectiveCurrentVersion.value) return true
  if (getTagVersions(row.tag).some(v => v.version === effectiveCurrentVersion.value)) return true
  const claimingTag = findClaimingTag(effectiveCurrentVersion.value)
  return claimingTag === row.tag
}
function otherVersionsContainsCurrent(): boolean {
  if (!effectiveCurrentVersion.value) return false
  const claimingTag = findClaimingTag(effectiveCurrentVersion.value)
  // If a tag claims it, check if that tag is in visibleTagRows
  if (claimingTag) {
    const isInVisibleTags = visibleTagRows.value.some(row => row.tag === claimingTag)
    if (!isInVisibleTags) return true
    return false
  }
  // No tag claims it - it would be in otherMajorGroups
  return true
}
function hiddenRowContainsCurrent(row: (typeof hiddenTagRows.value)[0]): boolean {
  if (!effectiveCurrentVersion.value) return false
  if (row.primaryVersion.version === effectiveCurrentVersion.value) return true
  const claimingTag = findClaimingTag(effectiveCurrentVersion.value)
  return claimingTag === row.tag
}
function majorGroupContainsCurrent(group: (typeof otherMajorGroups.value)[0]): boolean {
  if (!effectiveCurrentVersion.value) return false
  return group.versions.some(v => v.version === effectiveCurrentVersion.value)
}

return (_ctx: any,_cache: any) => {
  const _component_ButtonBase = _resolveComponent("ButtonBase")
  const _component_InputBase = _resolveComponent("InputBase")
  const _component_LinkBase = _resolveComponent("LinkBase")
  const _component_i18n_t = _resolveComponent("i18n-t")
  const _component_TooltipApp = _resolveComponent("TooltipApp")
  const _component_DateTime = _resolveComponent("DateTime")
  const _component_ProvenanceBadge = _resolveComponent("ProvenanceBadge")
  const _component_CollapsibleSection = _resolveComponent("CollapsibleSection")
  const _component_PackageVersionDistribution = _resolveComponent("PackageVersionDistribution")
  const _component_PackageChartModal = _resolveComponent("PackageChartModal")

  return (_openBlock(), _createElementBlock(_Fragment, null, [ (allTagRows.value.length > 0) ? (_openBlock(), _createBlock(_component_CollapsibleSection, {
          key: 0,
          title: _ctx.$t('package.versions.title'),
          id: "versions"
        }, {
          actions: _withCtx(() => [
            _createVNode(_component_ButtonBase, {
              variant: "secondary",
              class: "text-fg-subtle hover:text-fg transition-colors min-w-6 min-h-6 -m-1 p-1 rounded",
              title: _ctx.$t('package.downloads.community_distribution'),
              classicon: "i-lucide:file-stack",
              onClick: openDistributionModal
            }, {
              default: _withCtx(() => [
                _createElementVNode("span", _hoisted_1, _toDisplayString(_ctx.$t('package.downloads.community_distribution')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["title"])
          ]),
          default: _withCtx(() => [
            _createElementVNode("div", { class: "space-y-0.5 min-w-0" }, [
              _createElementVNode("div", null, [
                _createElementVNode("div", { class: "flex items-center gap-2 p-1" }, [
                  _createVNode(_component_InputBase, {
                    type: "text",
                    placeholder: _ctx.$t('package.versions.filter_placeholder'),
                    "aria-label": _ctx.$t('package.versions.filter_placeholder'),
                    "aria-invalid": isInvalidRange.value ? 'true' : undefined,
                    "aria-describedby": isInvalidRange.value ? 'semver-filter-error' : undefined,
                    autocomplete: "off",
                    class: _normalizeClass(["flex-1 min-w-0", isInvalidRange.value ? '!border-red-500' : '']),
                    size: "small",
                    modelValue: semverFilter.value,
                    "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((semverFilter).value = $event))
                  }, null, 10 /* CLASS, PROPS */, ["placeholder", "aria-label", "aria-invalid", "aria-describedby", "modelValue"]),
                  _createVNode(_component_TooltipApp, {
                    interactive: "",
                    position: "top"
                  }, {
                    content: _withCtx(() => [
                      _createElementVNode("p", { class: "text-xs text-fg-muted" }, [
                        _createVNode(_component_i18n_t, {
                          keypath: "package.versions.filter_tooltip",
                          tag: "span"
                        }, {
                          link: _withCtx(() => [
                            _createVNode(_component_LinkBase, { to: `${_unref(NPMX_DOCS_SITE)}/guide/semver-ranges` }, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_ctx.$t('package.versions.filter_tooltip_link')), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            }, 8 /* PROPS */, ["to"])
                          ]),
                          _: 1 /* STABLE */
                        })
                      ])
                    ]),
                    default: _withCtx(() => [
                      _createElementVNode("span", {
                        tabindex: "0",
                        class: "block cursor-help shrink-0 -m-2 p-2 -me-1 focus-visible:outline-2 focus-visible:outline-accent/70 rounded"
                      }, [
                        _createElementVNode("span", {
                          class: "block i-lucide:info w-3.5 h-3.5 text-fg-subtle",
                          role: "img",
                          "aria-label": _ctx.$t('package.versions.filter_help')
                        }, null, 8 /* PROPS */, ["aria-label"])
                      ])
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                (isInvalidRange.value)
                  ? (_openBlock(), _createElementBlock("p", {
                    key: 0,
                    id: "semver-filter-error",
                    class: "text-red-500 text-3xs mt-1",
                    role: "alert"
                  }, _toDisplayString(_ctx.$t('package.versions.filter_invalid')), 1 /* TEXT */))
                  : _createCommentVNode("v-if", true)
              ]),
              _createTextVNode("\n\n      " + "\n      "),
              (hasNoFilterMatches.value)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: "px-1 py-2 text-xs text-fg-subtle",
                  role: "status",
                  "aria-live": "polite"
                }, _toDisplayString(_ctx.$t('package.versions.no_matches')), 1 /* TEXT */))
                : _createCommentVNode("v-if", true),
              _createTextVNode("\n\n      " + "\n      "),
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(visibleTagRows.value, (row) => {
                return (_openBlock(), _createElementBlock("div", { key: row.id }, [
                  _createElementVNode("div", {
                    class: _normalizeClass(["flex items-center gap-2 pe-2 px-1 relative group/version-row hover:bg-bg-subtle focus-within:bg-bg-subtle transition-colors duration-200 rounded-lg", rowContainsCurrentVersion(row) ? 'bg-bg-subtle' : ''])
                  }, [
                    (getTagVersions(row.tag).length > 1 || !hasLoadedAll.value)
                      ? (_openBlock(), _createElementBlock("button", {
                        key: 0,
                        type: "button",
                        class: "size-5 -me-1 flex items-center justify-center text-fg-subtle hover:text-fg transition-colors rounded-sm relative z-10",
                        "aria-expanded": expandedTags.value.has(row.tag),
                        "aria-label": 
                expandedTags.value.has(row.tag)
                  ? _ctx.$t('package.versions.collapse', { tag: row.tag })
                  : _ctx.$t('package.versions.expand', { tag: row.tag })
              ,
                        "data-testid": "tag-expand-button",
                        onClick: _cache[1] || (_cache[1] = ($event: any) => (expandTagRow(row.tag)))
                      }, [
                        (loadingTags.value.has(row.tag))
                          ? (_openBlock(), _createElementBlock("span", {
                            key: 0,
                            class: "i-svg-spinners:ring-resize w-3 h-3",
                            "data-testid": "loading-spinner",
                            "aria-hidden": "true"
                          }))
                          : (_openBlock(), _createElementBlock("span", {
                            key: 1,
                            class: _normalizeClass(["size-3 transition-transform duration-200 rtl-flip", 
                  expandedTags.value.has(row.tag) ? 'i-lucide:chevron-down' : 'i-lucide:chevron-right'
                ]),
                            "aria-hidden": "true"
                          }))
                      ]))
                      : (_openBlock(), _createElementBlock("span", {
                        key: 1,
                        class: "w-5"
                      })),
                    _createTextVNode("\n\n          " + "\n          "),
                    _createElementVNode("div", { class: "flex-1 py-1.5 min-w-0 flex gap-2 justify-between items-center" }, [
                      _createElementVNode("div", { class: "overflow-hidden" }, [
                        _createVNode(_component_LinkBase, {
                          to: versionRoute(row.primaryVersion.version),
                          block: "",
                          class: _normalizeClass(["text-sm after:absolute after:inset-0 after:content-['']", 
                    row.primaryVersion.deprecated
                      ? 'text-red-800 group-hover/version-row:text-red-700 dark:text-red-400 dark:group-hover/version-row:text-red-300'
                      : undefined
                  ]),
                          title: 
                    row.primaryVersion.deprecated
                      ? _ctx.$t('package.versions.deprecated_title', {
                          version: row.primaryVersion.version,
                        })
                      : row.primaryVersion.version
                  ,
                          classicon: row.primaryVersion.deprecated ? 'i-lucide:octagon-alert' : undefined
                        }, {
                          default: _withCtx(() => [
                            _createElementVNode("span", _hoisted_2, _toDisplayString(row.primaryVersion.version), 1 /* TEXT */)
                          ]),
                          _: 2 /* DYNAMIC */
                        }, 10 /* CLASS, PROPS */, ["to", "title", "classicon"]),
                        (row.tags.length)
                          ? (_openBlock(), _createElementBlock("div", {
                            key: 0,
                            class: "flex items-center gap-1 mt-0.5 flex-wrap relative z-10"
                          }, [
                            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(row.tags, (tag) => {
                              return (_openBlock(), _createElementBlock("span", {
                                key: tag,
                                class: _normalizeClass(["text-4xs font-semibold uppercase tracking-wide truncate", tag === 'latest' ? 'text-accent' : 'text-fg-subtle']),
                                title: tag
                              }, _toDisplayString(tag), 11 /* TEXT, CLASS, PROPS */, ["title"]))
                            }), 128 /* KEYED_FRAGMENT */))
                          ]))
                          : _createCommentVNode("v-if", true)
                      ]),
                      _createElementVNode("div", { class: "flex items-center gap-2 shrink-0 relative z-10" }, [
                        (row.primaryVersion.time)
                          ? (_openBlock(), _createBlock(_component_DateTime, {
                            key: 0,
                            datetime: row.primaryVersion.time,
                            year: "numeric",
                            month: "short",
                            day: "numeric",
                            class: "text-xs text-fg-subtle"
                          }, null, 8 /* PROPS */, ["datetime"]))
                          : _createCommentVNode("v-if", true),
                        (row.primaryVersion.hasProvenance)
                          ? (_openBlock(), _createBlock(_component_ProvenanceBadge, {
                            key: 0,
                            "package-name": __props.packageName,
                            version: row.primaryVersion.version,
                            compact: "",
                            linked: false
                          }, null, 8 /* PROPS */, ["package-name", "version", "linked"]))
                          : _createCommentVNode("v-if", true)
                      ])
                    ])
                  ], 2 /* CLASS */),
                  _createTextVNode("\n\n        " + "\n        "),
                  (expandedTags.value.has(row.tag) && getFilteredTagVersions(row.tag).length > 1)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "ms-4 ps-2 border-is border-border space-y-0.5 pe-2"
                    }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(getFilteredTagVersions(row.tag).slice(1), (v) => {
                        return (_openBlock(), _createElementBlock("div", {
                          key: v.version,
                          class: _normalizeClass(["py-1 relative group/version-row hover:bg-bg-elevated/20 focus-within:bg-bg-elevated/20 transition-colors duration-200", v.version === effectiveCurrentVersion.value ? 'bg-bg-elevated/20 rounded' : ''])
                        }, [
                          _createElementVNode("div", { class: "flex items-center justify-between gap-2" }, [
                            _createVNode(_component_LinkBase, {
                              to: versionRoute(v.version),
                              block: "",
                              class: _normalizeClass(["text-xs after:absolute after:inset-0 after:content-['']", 
                    v.deprecated
                      ? 'text-red-800 group-hover/version-row:text-red-700 dark:text-red-400 dark:group-hover/version-row:text-red-300'
                      : undefined
                  ]),
                              title: 
                    v.deprecated
                      ? _ctx.$t('package.versions.deprecated_title', {
                          version: v.version,
                        })
                      : v.version
                  ,
                              classicon: v.deprecated ? 'i-lucide:octagon-alert' : undefined
                            }, {
                              default: _withCtx(() => [
                                _createElementVNode("span", _hoisted_3, _toDisplayString(v.version), 1 /* TEXT */)
                              ]),
                              _: 2 /* DYNAMIC */
                            }, 10 /* CLASS, PROPS */, ["to", "title", "classicon"]),
                            _createElementVNode("div", { class: "flex items-center gap-2 shrink-0 relative z-10" }, [
                              (v.time)
                                ? (_openBlock(), _createBlock(_component_DateTime, {
                                  key: 0,
                                  datetime: v.time,
                                  class: "text-3xs text-fg-subtle",
                                  year: "numeric",
                                  month: "short",
                                  day: "numeric"
                                }, null, 8 /* PROPS */, ["datetime"]))
                                : _createCommentVNode("v-if", true),
                              (v.hasProvenance)
                                ? (_openBlock(), _createBlock(_component_ProvenanceBadge, {
                                  key: 0,
                                  "package-name": __props.packageName,
                                  version: v.version,
                                  compact: "",
                                  linked: false
                                }, null, 8 /* PROPS */, ["package-name", "version", "linked"]))
                                : _createCommentVNode("v-if", true)
                            ])
                          ]),
                          (v.tags?.length && _unref(filterExcludedTags)(v.tags, row.tags).length)
                            ? (_openBlock(), _createElementBlock("div", {
                              key: 0,
                              class: "flex items-center gap-1 mt-0.5 relative z-10"
                            }, [
                              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(filterExcludedTags)(v.tags, row.tags), (tag) => {
                                return (_openBlock(), _createElementBlock("span", {
                                  key: tag,
                                  class: _normalizeClass(["text-5xs font-semibold uppercase tracking-wide truncate max-w-[120px]", tag === 'latest' ? 'text-accent' : 'text-fg-subtle']),
                                  title: tag
                                }, _toDisplayString(tag), 11 /* TEXT, CLASS, PROPS */, ["title"]))
                              }), 128 /* KEYED_FRAGMENT */))
                            ]))
                            : _createCommentVNode("v-if", true)
                        ], 2 /* CLASS */))
                      }), 128 /* KEYED_FRAGMENT */))
                    ]))
                    : _createCommentVNode("v-if", true)
                ]))
              }), 128 /* KEYED_FRAGMENT */)),
              _createTextVNode("\n\n      " + "\n      "),
              _createElementVNode("div", { class: "p-1" }, [
                _createElementVNode("button", {
                  type: "button",
                  class: _normalizeClass(["flex items-center gap-2 text-start rounded-sm w-full", otherVersionsContainsCurrent() ? 'bg-bg-subtle' : '']),
                  "aria-expanded": otherVersionsExpanded.value,
                  "aria-label": 
              otherVersionsExpanded.value
                ? _ctx.$t('package.versions.collapse_other')
                : _ctx.$t('package.versions.expand_other')
            ,
                  onClick: expandOtherVersions
                }, [
                  _createElementVNode("span", { class: "size-5 -me-1 flex items-center justify-center text-fg-subtle hover:text-fg transition-colors" }, [
                    (otherVersionsLoading.value)
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: "i-svg-spinners:ring-resize w-3 h-3",
                        "data-testid": "loading-spinner",
                        "aria-hidden": "true"
                      }))
                      : (_openBlock(), _createElementBlock("span", {
                        key: 1,
                        class: _normalizeClass(["w-3 h-3 transition-transform duration-200 rtl-flip", otherVersionsExpanded.value ? 'i-lucide:chevron-down' : 'i-lucide:chevron-right']),
                        "aria-hidden": "true"
                      }))
                  ]),
                  _createElementVNode("span", { class: "text-xs text-fg-muted py-1.5" }, [
                    _createTextVNode(_toDisplayString(_ctx.$t('package.versions.other_versions')) + "\n            ", 1 /* TEXT */),
                    (hiddenTagRows.value.length > 0)
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: "text-fg-subtle"
                      }, "\n              (" + _toDisplayString(_ctx.$t(
                    'package.versions.more_tagged',
                    { count: hiddenTagRows.value.length },
                    hiddenTagRows.value.length,
                  )) + ")\n            ", 1 /* TEXT */))
                      : _createCommentVNode("v-if", true)
                  ])
                ], 8 /* PROPS */, ["aria-expanded", "aria-label"]),
                _createTextVNode("\n\n        " + "\n        "),
                (otherVersionsExpanded.value)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: "ms-4 ps-2 border-is border-border space-y-0.5"
                  }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(hiddenTagRows.value, (row) => {
                      return (_openBlock(), _createElementBlock("div", {
                        key: row.id,
                        class: _normalizeClass(["py-1 relative group/version-row hover:bg-bg-subtle focus-within:bg-bg-subtle transition-colors duration-200 rounded-lg", hiddenRowContainsCurrent(row) ? 'bg-bg-subtle' : ''])
                      }, [
                        _createElementVNode("div", { class: "flex items-center justify-between gap-2" }, [
                          _createVNode(_component_LinkBase, {
                            to: versionRoute(row.primaryVersion.version),
                            block: "",
                            class: _normalizeClass(["text-xs after:absolute after:inset-0 after:content-['']", 
                    row.primaryVersion.deprecated
                      ? 'text-red-800 group-hover/version-row:text-red-700 dark:text-red-400 dark:group-hover/version-row:text-red-300'
                      : undefined
                  ]),
                            title: 
                    row.primaryVersion.deprecated
                      ? _ctx.$t('package.versions.deprecated_title', {
                          version: row.primaryVersion.version,
                        })
                      : row.primaryVersion.version
                  ,
                            classicon: row.primaryVersion.deprecated ? 'i-lucide:octagon-alert' : undefined
                          }, {
                            default: _withCtx(() => [
                              _createElementVNode("span", _hoisted_4, _toDisplayString(row.primaryVersion.version), 1 /* TEXT */)
                            ]),
                            _: 2 /* DYNAMIC */
                          }, 10 /* CLASS, PROPS */, ["to", "title", "classicon"]),
                          _createElementVNode("div", { class: "flex items-center gap-2 shrink-0 pe-2 relative z-10" }, [
                            (row.primaryVersion.time)
                              ? (_openBlock(), _createBlock(_component_DateTime, {
                                key: 0,
                                datetime: row.primaryVersion.time,
                                class: "text-3xs text-fg-subtle",
                                year: "numeric",
                                month: "short",
                                day: "numeric"
                              }, null, 8 /* PROPS */, ["datetime"]))
                              : _createCommentVNode("v-if", true)
                          ])
                        ]),
                        (row.tags.length)
                          ? (_openBlock(), _createElementBlock("div", {
                            key: 0,
                            class: "flex items-center gap-1 mt-0.5 flex-wrap relative z-10"
                          }, [
                            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(row.tags, (tag) => {
                              return (_openBlock(), _createElementBlock("span", {
                                key: tag,
                                class: _normalizeClass(["text-5xs font-semibold uppercase tracking-wide truncate max-w-[120px]", tag === 'latest' ? 'text-accent' : 'text-fg-subtle']),
                                title: tag
                              }, _toDisplayString(tag), 11 /* TEXT, CLASS, PROPS */, ["title"]))
                            }), 128 /* KEYED_FRAGMENT */))
                          ]))
                          : _createCommentVNode("v-if", true)
                      ], 2 /* CLASS */))
                    }), 128 /* KEYED_FRAGMENT */)),
                    _createTextVNode("\n\n          "),
                    _createTextVNode("\n          "),
                    (filteredOtherMajorGroups.value.length > 0)
                      ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(filteredOtherMajorGroups.value, (group) => {
                          return (_openBlock(), _createElementBlock("div", { key: group.groupKey }, [
                            (group.versions.length > 1)
                              ? (_openBlock(), _createElementBlock("div", {
                                key: 0,
                                class: _normalizeClass(["py-1 relative group/version-row hover:bg-bg-subtle focus-within:bg-bg-subtle transition-colors duration-200 rounded-lg", majorGroupContainsCurrent(group) ? 'bg-bg-subtle' : ''])
                              }, [
                                _createElementVNode("div", { class: "flex items-center justify-between gap-2" }, [
                                  _createElementVNode("div", { class: "flex items-center gap-2 min-w-0" }, [
                                    _createElementVNode("button", {
                                      type: "button",
                                      class: "w-4 h-4 flex items-center justify-center text-fg-subtle hover:text-fg transition-colors shrink-0 rounded-sm relative z-10",
                                      "aria-expanded": expandedMajorGroups.value.has(group.groupKey),
                                      "aria-label": 
                          expandedMajorGroups.value.has(group.groupKey)
                            ? _ctx.$t('package.versions.collapse_major', {
                                major: group.label,
                              })
                            : _ctx.$t('package.versions.expand_major', {
                                major: group.label,
                              })
                        ,
                                      "data-testid": "major-group-expand-button",
                                      onClick: _cache[2] || (_cache[2] = ($event: any) => (toggleMajorGroup(group.groupKey)))
                                    }, [
                                      _createElementVNode("span", {
                                        class: _normalizeClass(["w-3 h-3 transition-transform duration-200 rtl-flip", 
                            expandedMajorGroups.value.has(group.groupKey)
                              ? 'i-lucide:chevron-down'
                              : 'i-lucide:chevron-right'
                          ]),
                                        "aria-hidden": "true"
                                      }, null, 2 /* CLASS */)
                                    ], 8 /* PROPS */, ["aria-expanded", "aria-label"]),
                                    (group.versions[0]?.version)
                                      ? (_openBlock(), _createBlock(_component_LinkBase, {
                                        key: 0,
                                        to: versionRoute(group.versions[0]?.version),
                                        block: "",
                                        class: _normalizeClass(["text-xs after:absolute after:inset-0 after:content-['']", 
                          group.versions[0]?.deprecated
                            ? 'text-red-800 group-hover/version-row:text-red-700 dark:text-red-400 dark:group-hover/version-row:text-red-300'
                            : undefined
                        ]),
                                        title: 
                          group.versions[0]?.deprecated
                            ? _ctx.$t('package.versions.deprecated_title', {
                                version: group.versions[0]?.version,
                              })
                            : group.versions[0]?.version
                        ,
                                        classicon: 
                          group.versions[0]?.deprecated ? 'i-lucide:octagon-alert' : undefined
                      
                                      }, {
                                        default: _withCtx(() => [
                                          _createElementVNode("span", _hoisted_5, _toDisplayString(group.versions[0]?.version), 1 /* TEXT */)
                                        ]),
                                        _: 2 /* DYNAMIC */
                                      }, 10 /* CLASS, PROPS */, ["to", "title", "classicon"]))
                                      : _createCommentVNode("v-if", true)
                                  ]),
                                  _createElementVNode("div", { class: "flex items-center gap-2 shrink-0 pe-2 relative z-10" }, [
                                    (group.versions[0]?.time)
                                      ? (_openBlock(), _createBlock(_component_DateTime, {
                                        key: 0,
                                        datetime: group.versions[0]?.time,
                                        class: "text-3xs text-fg-subtle",
                                        year: "numeric",
                                        month: "short",
                                        day: "numeric"
                                      }, null, 8 /* PROPS */, ["datetime"]))
                                      : _createCommentVNode("v-if", true),
                                    (group.versions[0]?.hasProvenance)
                                      ? (_openBlock(), _createBlock(_component_ProvenanceBadge, {
                                        key: 0,
                                        "package-name": __props.packageName,
                                        version: group.versions[0]?.version,
                                        compact: "",
                                        linked: false
                                      }, null, 8 /* PROPS */, ["package-name", "version", "linked"]))
                                      : _createCommentVNode("v-if", true)
                                  ])
                                ]),
                                (group.versions[0]?.tags?.length)
                                  ? (_openBlock(), _createElementBlock("div", {
                                    key: 0,
                                    class: "flex items-center gap-1 ms-5 flex-wrap relative z-10"
                                  }, [
                                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(group.versions[0].tags, (tag) => {
                                      return (_openBlock(), _createElementBlock("span", {
                                        key: tag,
                                        class: _normalizeClass(["text-5xs font-semibold uppercase tracking-wide truncate max-w-[120px]", tag === 'latest' ? 'text-accent' : 'text-fg-subtle']),
                                        title: tag
                                      }, _toDisplayString(tag), 11 /* TEXT, CLASS, PROPS */, ["title"]))
                                    }), 128 /* KEYED_FRAGMENT */))
                                  ]))
                                  : _createCommentVNode("v-if", true)
                              ]))
                              : (_openBlock(), _createElementBlock("div", {
                                key: 1,
                                class: _normalizeClass(["py-1 relative group/version-row hover:bg-bg-subtle focus-within:bg-bg-subtle transition-colors duration-200 rounded-lg", majorGroupContainsCurrent(group) ? 'bg-bg-subtle' : ''])
                              }, [
                                _createElementVNode("div", { class: "flex items-center justify-between gap-2" }, [
                                  _createElementVNode("div", { class: "flex items-center gap-2 min-w-0" }, [
                                    (group.versions[0]?.version)
                                      ? (_openBlock(), _createBlock(_component_LinkBase, {
                                        key: 0,
                                        to: versionRoute(group.versions[0]?.version),
                                        block: "",
                                        class: _normalizeClass(["text-xs ms-6 after:absolute after:inset-0 after:content-['']", 
                          group.versions[0]?.deprecated
                            ? 'text-red-800 group-hover/version-row:text-red-700 dark:text-red-400 dark:group-hover/version-row:text-red-300'
                            : undefined
                        ]),
                                        title: 
                          group.versions[0]?.deprecated
                            ? _ctx.$t('package.versions.deprecated_title', {
                                version: group.versions[0]?.version,
                              })
                            : group.versions[0]?.version
                        ,
                                        classicon: 
                          group.versions[0]?.deprecated ? 'i-lucide:octagon-alert' : undefined
                      
                                      }, {
                                        default: _withCtx(() => [
                                          _createElementVNode("span", _hoisted_6, _toDisplayString(group.versions[0]?.version), 1 /* TEXT */)
                                        ]),
                                        _: 2 /* DYNAMIC */
                                      }, 10 /* CLASS, PROPS */, ["to", "title", "classicon"]))
                                      : _createCommentVNode("v-if", true)
                                  ]),
                                  _createElementVNode("div", { class: "flex items-center gap-2 shrink-0 pe-2 relative z-10" }, [
                                    (group.versions[0]?.time)
                                      ? (_openBlock(), _createBlock(_component_DateTime, {
                                        key: 0,
                                        datetime: group.versions[0]?.time,
                                        class: "text-3xs text-fg-subtle",
                                        year: "numeric",
                                        month: "short",
                                        day: "numeric"
                                      }, null, 8 /* PROPS */, ["datetime"]))
                                      : _createCommentVNode("v-if", true),
                                    (group.versions[0]?.hasProvenance)
                                      ? (_openBlock(), _createBlock(_component_ProvenanceBadge, {
                                        key: 0,
                                        "package-name": __props.packageName,
                                        version: group.versions[0]?.version,
                                        compact: "",
                                        linked: false
                                      }, null, 8 /* PROPS */, ["package-name", "version", "linked"]))
                                      : _createCommentVNode("v-if", true)
                                  ])
                                ]),
                                (group.versions[0]?.tags?.length)
                                  ? (_openBlock(), _createElementBlock("div", {
                                    key: 0,
                                    class: "flex items-center gap-1 ms-5 relative z-10"
                                  }, [
                                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(group.versions[0].tags, (tag) => {
                                      return (_openBlock(), _createElementBlock("span", {
                                        key: tag,
                                        class: _normalizeClass(["text-5xs font-semibold uppercase tracking-wide", tag === 'latest' ? 'text-accent' : 'text-fg-subtle'])
                                      }, _toDisplayString(tag), 3 /* TEXT, CLASS */))
                                    }), 128 /* KEYED_FRAGMENT */))
                                  ]))
                                  : _createCommentVNode("v-if", true)
                              ])),
                            _createTextVNode("\n              " + "\n              " + "\n\n              " + "\n              "),
                            (expandedMajorGroups.value.has(group.groupKey) && group.versions.length > 1)
                              ? (_openBlock(), _createElementBlock("div", {
                                key: 0,
                                class: "ms-6 space-y-0.5"
                              }, [
                                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(group.versions.slice(1), (v) => {
                                  return (_openBlock(), _createElementBlock("div", {
                                    key: v.version,
                                    class: _normalizeClass(["py-1 relative group/version-row hover:bg-bg-elevated/20 focus-within:bg-bg-elevated/20 transition-colors duration-200", v.version === effectiveCurrentVersion.value ? 'bg-bg-elevated/20 rounded' : ''])
                                  }, [
                                    _createElementVNode("div", { class: "flex items-center justify-between gap-2" }, [
                                      _createVNode(_component_LinkBase, {
                                        to: versionRoute(v.version),
                                        block: "",
                                        class: _normalizeClass(["text-xs after:absolute after:inset-0 after:content-['']", 
                          v.deprecated
                            ? 'text-red-800 group-hover/version-row:text-red-700 dark:text-red-400 dark:group-hover/version-row:text-red-300'
                            : undefined
                        ]),
                                        title: 
                          v.deprecated
                            ? _ctx.$t('package.versions.deprecated_title', {
                                version: v.version,
                              })
                            : v.version
                        ,
                                        classicon: v.deprecated ? 'i-lucide:octagon-alert' : undefined
                                      }, {
                                        default: _withCtx(() => [
                                          _createElementVNode("span", _hoisted_7, _toDisplayString(v.version), 1 /* TEXT */)
                                        ]),
                                        _: 2 /* DYNAMIC */
                                      }, 10 /* CLASS, PROPS */, ["to", "title", "classicon"]),
                                      _createElementVNode("div", { class: "flex items-center gap-2 shrink-0 pe-2 relative z-10" }, [
                                        (v.time)
                                          ? (_openBlock(), _createBlock(_component_DateTime, {
                                            key: 0,
                                            datetime: v.time,
                                            class: "text-3xs text-fg-subtle",
                                            year: "numeric",
                                            month: "short",
                                            day: "numeric"
                                          }, null, 8 /* PROPS */, ["datetime"]))
                                          : _createCommentVNode("v-if", true),
                                        (v.hasProvenance)
                                          ? (_openBlock(), _createBlock(_component_ProvenanceBadge, {
                                            key: 0,
                                            "package-name": __props.packageName,
                                            version: v.version,
                                            compact: "",
                                            linked: false
                                          }, null, 8 /* PROPS */, ["package-name", "version", "linked"]))
                                          : _createCommentVNode("v-if", true)
                                      ])
                                    ]),
                                    (v.tags?.length)
                                      ? (_openBlock(), _createElementBlock("div", {
                                        key: 0,
                                        class: "flex items-center gap-1 mt-0.5 relative z-10"
                                      }, [
                                        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(v.tags, (tag) => {
                                          return (_openBlock(), _createElementBlock("span", {
                                            key: tag,
                                            class: _normalizeClass(["text-5xs font-semibold uppercase tracking-wide", tag === 'latest' ? 'text-accent' : 'text-fg-subtle'])
                                          }, _toDisplayString(tag), 3 /* TEXT, CLASS */))
                                        }), 128 /* KEYED_FRAGMENT */))
                                      ]))
                                      : _createCommentVNode("v-if", true)
                                  ], 2 /* CLASS */))
                                }), 128 /* KEYED_FRAGMENT */))
                              ]))
                              : _createCommentVNode("v-if", true)
                          ]))
                        }), 128 /* KEYED_FRAGMENT */))
                      ], 64 /* STABLE_FRAGMENT */))
                      : (hasLoadedAll.value && hiddenTagRows.value.length === 0)
                        ? (_openBlock(), _createElementBlock("div", {
                          key: 1,
                          class: "py-1 text-xs text-fg-subtle"
                        }, _toDisplayString(_ctx.$t('package.versions.all_covered')), 1 /* TEXT */))
                      : _createCommentVNode("v-if", true)
                  ]))
                  : _createCommentVNode("v-if", true)
              ])
            ])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["title"])) : _createCommentVNode("v-if", true), (isDistributionModalOpen.value) ? (_openBlock(), _createBlock(_component_PackageChartModal, {
          key: 0,
          "modal-title": _ctx.$t('package.versions.distribution_modal_title'),
          onClose: closeDistributionModal,
          onTransitioned: handleDistributionModalTransitioned
        }, {
          default: _withCtx(() => [
            _createVNode(_Transition, {
              name: "opacity",
              mode: "out-in"
            }, {
              default: _withCtx(() => [
                (hasDistributionModalTransitioned.value)
                  ? (_openBlock(), _createBlock(_component_PackageVersionDistribution, {
                    key: 0,
                    "package-name": __props.packageName,
                    "in-modal": true
                  }, null, 8 /* PROPS */, ["package-name", "in-modal"]))
                  : _createCommentVNode("v-if", true)
              ]),
              _: 1 /* STABLE */
            }),
            _createTextVNode("\n\n    "),
            _createTextVNode("\n    "),
            _createTextVNode("\n    "),
            (!hasDistributionModalTransitioned.value)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: "w-full aspect-[272/609] sm:aspect-[718/592.67]"
              }))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["modal-title"])) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */))
}
}

})
