import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = { dir: "ltr" }
const _hoisted_2 = { dir: "ltr" }
const _hoisted_3 = { class: "truncate", dir: "ltr" }
import type { PackageVersionInfo } from '#shared/types'
import { onClickOutside } from '@vueuse/core'
import { compare } from 'semver'
import { buildVersionToTagsMap, getPrereleaseChannel, getVersionGroupKey, getVersionGroupLabel, isSameVersionGroup } from '~/utils/versions'
import { fetchAllPackageVersions } from '~/utils/npm/api'

interface VersionDisplay {
  version: string
  tags?: string[]
  isCurrent?: boolean
}
interface VersionGroup {
  id: string
  label: string
  primaryVersion: VersionDisplay
  versions: VersionDisplay[]
  isExpanded: boolean
  isLoading: boolean
}

export default /*@__PURE__*/_defineComponent({
  __name: 'VersionSelector',
  props: {
    packageName: { type: String, required: true },
    currentVersion: { type: String, required: true },
    versions: { type: null, required: true },
    distTags: { type: null, required: true },
    urlPattern: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const isOpen = shallowRef(false)
const dropdownRef = useTemplateRef('dropdownRef')
const listboxRef = useTemplateRef('listboxRef')
const focusedIndex = shallowRef(-1)
onClickOutside(dropdownRef, () => {
  isOpen.value = false
})
// ============================================================================
// Version Display Types
// ============================================================================
// ============================================================================
// State
// ============================================================================
/** All version groups (dist-tags + major versions) */
const versionGroups = ref<VersionGroup[]>([])
/** Whether we've loaded all versions from the API */
const hasLoadedAll = shallowRef(false)
/** Loading state for initial all-versions fetch */
const isLoadingAll = shallowRef(false)
/** Cached full version list */
const allVersionsCache = shallowRef<PackageVersionInfo[] | null>(null)
// ============================================================================
// Computed
// ============================================================================
const latestVersion = computed(() => props.distTags.latest)
const versionToTags = computed(() => buildVersionToTagsMap(props.distTags))
/** Get URL for a specific version */
function getVersionUrl(version: string): string {
  return props.urlPattern.replace('{version}', version)
}
/** Safe semver comparison with fallback */
function safeCompareVersions(a: string, b: string): number {
  try {
    return compare(a, b)
  } catch {
    return a.localeCompare(b)
  }
}
// ============================================================================
// Initial Groups (SSR-safe, from props only)
// ============================================================================
/** Build initial version groups from dist-tags only */
function buildInitialGroups(): VersionGroup[] {
  const groups: VersionGroup[] = []
  const seenVersions = new Set<string>()
  // Group tags by version (multiple tags can point to same version)
  const versionMap = new Map<string, { tags: string[] }>()
  for (const [tag, version] of Object.entries(props.distTags)) {
    const existing = versionMap.get(version)
    if (existing) {
      existing.tags.push(tag)
    } else {
      versionMap.set(version, { tags: [tag] })
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
  // Build groups from tagged versions, sorted by version descending
  const sortedEntries = Array.from(versionMap.entries()).sort((a, b) =>
    safeCompareVersions(b[0], a[0]),
  )
  for (const [version, { tags }] of sortedEntries) {
    seenVersions.add(version)
    const primaryTag = tags[0]!
    groups.push({
      id: `tag:${primaryTag}`,
      label: primaryTag,
      primaryVersion: {
        version,
        tags,
        isCurrent: version === props.currentVersion,
      },
      versions: [], // Will be populated when expanded
      isExpanded: false,
      isLoading: false,
    })
  }
  return groups
}
// Initialize groups
versionGroups.value = buildInitialGroups()
// ============================================================================
// Load All Versions
// ============================================================================
async function loadAllVersions(): Promise<PackageVersionInfo[]> {
  if (allVersionsCache.value) return allVersionsCache.value
  isLoadingAll.value = true
  try {
    const versions = await fetchAllPackageVersions(props.packageName)
    allVersionsCache.value = versions
    hasLoadedAll.value = true
    return versions
  } finally {
    isLoadingAll.value = false
  }
}
/** Process loaded versions and populate groups */
function processLoadedVersions(allVersions: PackageVersionInfo[]) {
  const groups: VersionGroup[] = []
  const claimedVersions = new Set<string>()
  // Process each dist-tag and find its channel versions
  for (const [tag, tagVersion] of Object.entries(props.distTags)) {
    // Skip if we already have a group for this version
    const existingGroup = groups.find(g => g.primaryVersion.version === tagVersion)
    if (existingGroup) {
      // Add tag to existing group
      if (!existingGroup.primaryVersion.tags?.includes(tag)) {
        existingGroup.primaryVersion.tags = [...(existingGroup.primaryVersion.tags ?? []), tag]
        existingGroup.primaryVersion.tags.sort((a, b) => {
          if (a === 'latest') return -1
          if (b === 'latest') return 1
          return a.localeCompare(b)
        })
        // Update label to primary tag
        existingGroup.label = existingGroup.primaryVersion.tags[0]!
        existingGroup.id = `tag:${existingGroup.label}`
      }
      continue
    }
    const tagChannel = getPrereleaseChannel(tagVersion)
    // Find all versions in the same version group + prerelease channel
    // For 0.x versions, this means same major.minor; for 1.x+, same major
    const channelVersions = allVersions
      .filter(v => {
        const vChannel = getPrereleaseChannel(v.version)
        return isSameVersionGroup(v.version, tagVersion) && vChannel === tagChannel
      })
      .sort((a, b) => safeCompareVersions(b.version, a.version))
      .map(v => ({
        version: v.version,
        tags: versionToTags.value.get(v.version),
        isCurrent: v.version === props.currentVersion,
      }))
    // Mark these versions as claimed
    for (const v of channelVersions) {
      claimedVersions.add(v.version)
    }
    groups.push({
      id: `tag:${tag}`,
      label: tag,
      primaryVersion: {
        version: tagVersion,
        tags: versionToTags.value.get(tagVersion),
        isCurrent: tagVersion === props.currentVersion,
      },
      versions: channelVersions,
      isExpanded: false,
      isLoading: false,
    })
  }
  // Sort groups by primary version descending
  groups.sort((a, b) => safeCompareVersions(b.primaryVersion.version, a.primaryVersion.version))
  // Deduplicate groups with same version (merge their tags)
  const deduped: VersionGroup[] = []
  for (const group of groups) {
    const existing = deduped.find(g => g.primaryVersion.version === group.primaryVersion.version)
    if (existing) {
      // Merge tags
      const allTags = [
        ...(existing.primaryVersion.tags ?? []),
        ...(group.primaryVersion.tags ?? []),
      ]
      const uniqueTags = [...new Set(allTags)].sort((a, b) => {
        if (a === 'latest') return -1
        if (b === 'latest') return 1
        return a.localeCompare(b)
      })
      existing.primaryVersion.tags = uniqueTags
      existing.label = uniqueTags[0]!
      existing.id = `tag:${existing.label}`
    } else {
      deduped.push(group)
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
      tags: versionToTags.value.get(v.version),
      isCurrent: v.version === props.currentVersion,
    })
  }
  // Sort within each group and create groups
  // Sort group keys: "2", "1", "0.10", "0.9" (descending)
  const sortedGroupKeys = Array.from(byGroupKey.keys()).sort((a, b) => {
    // Parse as numbers for proper sorting
    const [aMajor, aMinor] = a.split('.').map(Number)
    const [bMajor, bMinor] = b.split('.').map(Number)
    if (aMajor !== bMajor) return (bMajor ?? 0) - (aMajor ?? 0)
    return (bMinor ?? -1) - (aMinor ?? -1)
  })
  for (const groupKey of sortedGroupKeys) {
    const versions = byGroupKey.get(groupKey)!
    versions.sort((a, b) => safeCompareVersions(b.version, a.version))
    const primaryVersion = versions[0]
    if (primaryVersion) {
      deduped.push({
        id: `group:${groupKey}`,
        label: getVersionGroupLabel(groupKey),
        primaryVersion,
        versions,
        isExpanded: false,
        isLoading: false,
      })
    }
  }
  versionGroups.value = deduped
}
// ============================================================================
// Expand/Collapse
// ============================================================================
async function toggleGroup(groupId: string) {
  const group = versionGroups.value.find(g => g.id === groupId)
  if (!group) return
  if (group.isExpanded) {
    group.isExpanded = false
    return
  }
  // Load all versions if not yet loaded
  if (!hasLoadedAll.value) {
    group.isLoading = true
    try {
      const allVersions = await loadAllVersions()
      processLoadedVersions(allVersions)
      // Find the group again after processing (it may have moved)
      const updatedGroup = versionGroups.value.find(g => g.id === groupId)
      if (updatedGroup) {
        updatedGroup.isExpanded = true
      }
    } catch (error) {
      // eslint-disable-next-line no-console
      console.error('Failed to load versions:', error)
    } finally {
      group.isLoading = false
    }
  } else {
    group.isExpanded = true
  }
}
// ============================================================================
// Keyboard Navigation
// ============================================================================
/** Flat list of navigable items for keyboard navigation */
const flatItems = computed(() => {
  const items: Array<{ type: 'group' | 'version'; groupId: string; version?: VersionDisplay }> = []

  for (const group of versionGroups.value) {
    items.push({ type: 'group', groupId: group.id, version: group.primaryVersion })

    if (group.isExpanded && group.versions.length > 1) {
      // Skip first version (it's the primary)
      for (const v of group.versions.slice(1)) {
        items.push({ type: 'version', groupId: group.id, version: v })
      }
    }
  }

  return items
})
function handleButtonKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    isOpen.value = false
  } else if (event.key === 'ArrowDown' && !isOpen.value) {
    event.preventDefault()
    isOpen.value = true
    focusedIndex.value = 0
  }
}
function handleListboxKeydown(event: KeyboardEvent) {
  const items = flatItems.value
  switch (event.key) {
    case 'Escape':
      isOpen.value = false
      break
    case 'ArrowDown':
      event.preventDefault()
      focusedIndex.value = Math.min(focusedIndex.value + 1, items.length - 1)
      scrollToFocused()
      break
    case 'ArrowUp':
      event.preventDefault()
      focusedIndex.value = Math.max(focusedIndex.value - 1, 0)
      scrollToFocused()
      break
    case 'Home':
      event.preventDefault()
      focusedIndex.value = 0
      scrollToFocused()
      break
    case 'End':
      event.preventDefault()
      focusedIndex.value = items.length - 1
      scrollToFocused()
      break
    case 'ArrowRight': {
      event.preventDefault()
      const item = items[focusedIndex.value]
      if (item?.type === 'group') {
        const group = versionGroups.value.find(g => g.id === item.groupId)
        if (group && !group.isExpanded && group.versions.length > 1) {
          toggleGroup(item.groupId)
        }
      }
      break
    }
    case 'ArrowLeft': {
      event.preventDefault()
      const item = items[focusedIndex.value]
      if (item?.type === 'group') {
        const group = versionGroups.value.find(g => g.id === item.groupId)
        if (group?.isExpanded) {
          group.isExpanded = false
        }
      } else if (item?.type === 'version') {
        // Jump to parent group
        const groupIndex = items.findIndex(i => i.type === 'group' && i.groupId === item.groupId)
        if (groupIndex >= 0) {
          focusedIndex.value = groupIndex
          scrollToFocused()
        }
      }
      break
    }
    case 'Enter':
    case ' ':
      event.preventDefault()
      if (focusedIndex.value >= 0 && focusedIndex.value < items.length) {
        const item = items[focusedIndex.value]
        if (item?.version) {
          navigateToVersion(item.version.version)
        }
      }
      break
  }
}
function scrollToFocused() {
  nextTick(() => {
    const focused = listboxRef.value?.querySelector('[data-focused="true"]')
    focused?.scrollIntoView({ block: 'nearest' })
  })
}
function navigateToVersion(version: string) {
  isOpen.value = false
  navigateTo(getVersionUrl(version))
}
// Reset focused index when dropdown opens
watch(isOpen, open => {
  if (open) {
    // Find current version in flat list
    const currentIdx = flatItems.value.findIndex(item => item.version?.isCurrent)
    focusedIndex.value = currentIdx >= 0 ? currentIdx : 0
  }
})
// Rebuild groups when props change
watch(
  () => [props.distTags, props.versions, props.currentVersion],
  () => {
    if (hasLoadedAll.value && allVersionsCache.value) {
      processLoadedVersions(allVersionsCache.value)
    } else {
      versionGroups.value = buildInitialGroups()
    }
  },
)

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (_openBlock(), _createElementBlock("div", {
      ref_key: "dropdownRef", ref: dropdownRef,
      class: "relative"
    }, [ _createElementVNode("button", {
        type: "button",
        "aria-haspopup": "listbox",
        "aria-expanded": isOpen.value,
        class: "flex items-center gap-1.5 text-fg-subtle font-mono text-sm hover:text-fg transition-[color] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-bg rounded",
        onClick: _cache[0] || (_cache[0] = ($event: any) => (isOpen.value = !isOpen.value)),
        onKeydown: handleButtonKeydown
      }, [ _createElementVNode("span", _hoisted_1, _toDisplayString(__props.currentVersion), 1 /* TEXT */), (__props.currentVersion === latestVersion.value) ? (_openBlock(), _createElementBlock("span", {
            key: 0,
            class: "text-xs px-1.5 py-0.5 rounded badge-green font-sans font-medium"
          }, "\n        latest\n      ")) : _createCommentVNode("v-if", true), _createElementVNode("span", {
          class: _normalizeClass(["i-lucide:chevron-down w-3.5 h-3.5 transition-[transform] duration-200 motion-reduce:transition-none", { 'rotate-180': isOpen.value }]),
          "aria-hidden": "true"
        }, null, 2 /* CLASS */) ], 40 /* PROPS, NEED_HYDRATION */, ["aria-expanded"]), _createVNode(_Transition, {
        "enter-active-class": "transition-[opacity,transform] duration-150 ease-out motion-reduce:transition-none",
        "enter-from-class": "opacity-0 scale-95",
        "enter-to-class": "opacity-100 scale-100",
        "leave-active-class": "transition-[opacity,transform] duration-100 ease-in motion-reduce:transition-none",
        "leave-from-class": "opacity-100 scale-100",
        "leave-to-class": "opacity-0 scale-95"
      }, {
        default: _withCtx(() => [
          (isOpen.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              ref: "listboxRef",
              role: "listbox",
              tabindex: "0",
              "aria-activedescendant": 
            focusedIndex.value >= 0 ? `version-${flatItems.value[focusedIndex.value]?.version?.version}` : undefined
          ,
              class: "absolute top-full inset-is-0 mt-2 min-w-[220px] bg-bg-subtle/80 backdrop-blur-sm border border-border-subtle rounded-lg shadow-lg shadow-fg-subtle/10 z-50 py-1 max-h-[400px] overflow-y-auto overscroll-contain focus-visible:outline-none",
              onKeydown: handleListboxKeydown
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(versionGroups.value, (group) => {
                return (_openBlock(), _createElementBlock("div", { key: group.id }, [
                  _createElementVNode("div", {
                    id: `version-${group.primaryVersion.version}`,
                    role: "option",
                    "aria-selected": group.primaryVersion.isCurrent,
                    "data-focused": 
                flatItems.value[focusedIndex.value]?.groupId === group.id &&
                flatItems.value[focusedIndex.value]?.type === 'group'
              ,
                    class: _normalizeClass(["flex items-center gap-2 px-3 py-2 text-sm font-mono hover:bg-bg-muted transition-[color,background-color] focus-visible:outline-none", [
                group.primaryVersion.isCurrent ? 'text-fg bg-bg-muted' : 'text-fg-muted',
                flatItems.value[focusedIndex.value]?.groupId === group.id &&
                flatItems.value[focusedIndex.value]?.type === 'group'
                  ? 'bg-bg-muted'
                  : '',
              ]])
                  }, [
                    (group.versions.length > 1 || !hasLoadedAll.value)
                      ? (_openBlock(), _createElementBlock("button", {
                        key: 0,
                        type: "button",
                        class: "w-4 h-4 flex items-center justify-center text-fg-subtle hover:text-fg transition-colors shrink-0",
                        "aria-expanded": group.isExpanded,
                        "aria-label": group.isExpanded ? 'Collapse' : 'Expand',
                        onClick: _cache[1] || (_cache[1] = _withModifiers(($event: any) => (toggleGroup(group.id)), ["stop"]))
                      }, [
                        (group.isLoading)
                          ? (_openBlock(), _createElementBlock("span", {
                            key: 0,
                            class: "i-svg-spinners:ring-resize w-3 h-3",
                            "aria-hidden": "true"
                          }))
                          : (_openBlock(), _createElementBlock("span", {
                            key: 1,
                            class: _normalizeClass(["w-3 h-3 transition-transform duration-200 rtl-flip", group.isExpanded ? 'i-lucide:chevron-down' : 'i-lucide:chevron-right']),
                            "aria-hidden": "true"
                          }))
                      ]))
                      : (_openBlock(), _createElementBlock("span", {
                        key: 1,
                        class: "w-4"
                      })),
                    _createTextVNode("\n\n            " + "\n            "),
                    _createVNode(_component_NuxtLink, {
                      to: getVersionUrl(group.primaryVersion.version),
                      class: "flex-1 truncate hover:text-fg transition-colors",
                      onClick: _cache[2] || (_cache[2] = ($event: any) => (isOpen.value = false))
                    }, {
                      default: _withCtx(() => [
                        _createElementVNode("span", _hoisted_2, _toDisplayString(group.primaryVersion.version), 1 /* TEXT */)
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 8 /* PROPS */, ["to"]),
                    _createTextVNode("\n\n            " + "\n            "),
                    (group.primaryVersion.tags?.length)
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: "flex items-center gap-1 shrink-0"
                      }, [
                        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(group.primaryVersion.tags, (tag) => {
                          return (_openBlock(), _createElementBlock("span", {
                            key: tag,
                            class: _normalizeClass(["text-xs px-1.5 py-0.5 rounded font-sans font-medium", tag === 'latest' ? 'badge-green' : 'badge-subtle'])
                          }, _toDisplayString(tag), 3 /* TEXT, CLASS */))
                        }), 128 /* KEYED_FRAGMENT */))
                      ]))
                      : _createCommentVNode("v-if", true)
                  ], 10 /* CLASS, PROPS */, ["id", "aria-selected", "data-focused"]),
                  _createTextVNode("\n\n          " + "\n          "),
                  (group.isExpanded && group.versions.length > 1)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "ms-6 border-is border-border"
                    }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(group.versions.slice(1), (v) => {
                        return (_openBlock(), _createElementBlock(_Fragment, { key: v.version }, [
                          _createVNode(_component_NuxtLink, {
                            id: `version-${v.version}`,
                            to: getVersionUrl(v.version),
                            role: "option",
                            "aria-selected": v.isCurrent,
                            "data-focused": 
                    flatItems.value[focusedIndex.value]?.groupId === group.id &&
                    flatItems.value[focusedIndex.value]?.type === 'version' &&
                    flatItems.value[focusedIndex.value]?.version?.version === v.version
                  ,
                            class: _normalizeClass(["flex items-center justify-between gap-2 ps-4 pe-3 py-1.5 text-xs font-mono hover:bg-bg-muted transition-[color,background-color] focus-visible:outline-none", [
                    v.isCurrent ? 'text-fg bg-bg-muted' : 'text-fg-subtle',
                    flatItems.value[focusedIndex.value]?.version?.version === v.version ? 'bg-bg-muted' : '',
                  ]]),
                            onClick: _cache[3] || (_cache[3] = ($event: any) => (isOpen.value = false))
                          }, {
                            default: _withCtx(() => [
                              _createElementVNode("span", _hoisted_3, _toDisplayString(v.version), 1 /* TEXT */),
                              (v.tags?.length)
                                ? (_openBlock(), _createElementBlock("span", {
                                  key: 0,
                                  class: "flex items-center gap-1 shrink-0"
                                }, [
                                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(v.tags, (tag) => {
                                    return (_openBlock(), _createElementBlock("span", {
                                      key: tag,
                                      class: _normalizeClass(["text-4xs px-1 py-0.5 rounded font-sans font-medium", 
                        tag === 'latest'
                          ? 'bg-emerald-500/10 text-emerald-400'
                          : 'bg-bg-muted text-fg-subtle'
                      ])
                                    }, _toDisplayString(tag), 3 /* TEXT, CLASS */))
                                  }), 128 /* KEYED_FRAGMENT */))
                                ]))
                                : _createCommentVNode("v-if", true)
                            ]),
                            _: 2 /* DYNAMIC */
                          }, 10 /* CLASS, PROPS */, ["id", "to", "aria-selected", "data-focused"])
                        ], 64 /* STABLE_FRAGMENT */))
                      }), 128 /* KEYED_FRAGMENT */))
                    ]))
                    : _createCommentVNode("v-if", true)
                ]))
              }), 128 /* KEYED_FRAGMENT */)),
              _createTextVNode("\n\n        "),
              _createTextVNode("\n        "),
              _createElementVNode("div", { class: "border-t border-border mt-1 pt-1 px-3 py-2" }, [
                _createVNode(_component_NuxtLink, {
                  to: _ctx.packageRoute(__props.packageName),
                  class: "text-xs text-fg-subtle hover:text-fg transition-[color] focus-visible:outline-none focus-visible:text-fg",
                  onClick: _cache[4] || (_cache[4] = ($event: any) => (isOpen.value = false))
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_ctx.$t(
                  'package.versions.view_all',
                  { count: Object.keys(__props.versions).length },
                  Object.keys(__props.versions).length,
                )), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["to"])
              ])
            ]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }) ], 512 /* NEED_PATCH */))
}
}

})
