import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:circle-alert w-3 h-3", "aria-hidden": "true" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:lightbulb w-3 h-3", "aria-hidden": "true" })
const _hoisted_3 = { class: "sr-only" }
const _hoisted_4 = { class: "sr-only" }
import { SEVERITY_TEXT_COLORS, getHighestSeverity } from '#shared/utils/severity'
import { getOutdatedTooltip, getVersionClass } from '~/utils/npm/outdated-dependencies'

export default /*@__PURE__*/_defineComponent({
  __name: 'Dependencies',
  props: {
    packageName: { type: String, required: true },
    version: { type: String, required: true },
    dependencies: { type: null, required: false },
    peerDependencies: { type: null, required: false },
    peerDependenciesMeta: { type: null, required: false },
    optionalDependencies: { type: null, required: false }
  },
  setup(__props: any) {

const props = __props
const { t } = useI18n()
// Fetch outdated info for dependencies
const outdatedDeps = useOutdatedDependencies(() => props.dependencies)
// Fetch replacement suggestions for dependencies
const replacementDeps = useReplacementDependencies(() => props.dependencies)
// Get vulnerability info from shared cache (already fetched by PackageVulnerabilityTree)
const { data: vulnTree } = useDependencyAnalysis(
  () => props.packageName,
  () => props.version,
)
// Check if a dependency has vulnerabilities (only direct deps)
function getVulnerableDepInfo(depName: string) {
  if (!vulnTree.value) return null
  return vulnTree.value.vulnerablePackages.find(p => p.name === depName && p.depth === 'direct')
}
// Check if a dependency is deprecated (only direct deps)
function getDeprecatedDepInfo(depName: string) {
  if (!vulnTree.value) return null
  return vulnTree.value.deprecatedPackages.find(p => p.name === depName && p.depth === 'direct')
}
// Expanded state for each section
const depsExpanded = shallowRef(false)
const peerDepsExpanded = shallowRef(false)
const optionalDepsExpanded = shallowRef(false)
// Sort dependencies alphabetically
const sortedDependencies = computed(() => {
  if (!props.dependencies) return []
  return Object.entries(props.dependencies).sort(([a], [b]) => a.localeCompare(b))
})
// Sort peer dependencies alphabetically, with required first then optional
const sortedPeerDependencies = computed(() => {
  if (!props.peerDependencies) return []

  return Object.entries(props.peerDependencies)
    .map(([name, version]) => ({
      name,
      version,
      optional: props.peerDependenciesMeta?.[name]?.optional ?? false,
    }))
    .sort((a, b) => {
      // Required first, then optional
      if (a.optional !== b.optional) return a.optional ? 1 : -1
      return a.name.localeCompare(b.name)
    })
})
// Sort optional dependencies alphabetically
const sortedOptionalDependencies = computed(() => {
  if (!props.optionalDependencies) return []
  return Object.entries(props.optionalDependencies).sort(([a], [b]) => a.localeCompare(b))
})
// Get version tooltip
function getDepVersionTooltip(dep: string, version: string) {
  const outdated = outdatedDeps.value[dep]
  if (outdated) return getOutdatedTooltip(outdated, t)
  if (getVulnerableDepInfo(dep) || getDeprecatedDepInfo(dep)) return version
  if (replacementDeps.value[dep]) return t('package.dependencies.has_replacement')
  return version
}
// Get version class
function getDepVersionClass(dep: string) {
  const outdated = outdatedDeps.value[dep]
  if (outdated) return getVersionClass(outdated)
  if (getVulnerableDepInfo(dep) || getDeprecatedDepInfo(dep)) return getVersionClass(undefined)
  if (replacementDeps.value[dep]) return 'text-amber-700 dark:text-amber-500'
  return getVersionClass(undefined)
}
const numberFormatter = useNumberFormatter()

return (_ctx: any,_cache: any) => {
  const _component_LinkBase = _resolveComponent("LinkBase")
  const _component_TooltipApp = _resolveComponent("TooltipApp")
  const _component_CollapsibleSection = _resolveComponent("CollapsibleSection")
  const _component_TagStatic = _resolveComponent("TagStatic")

  return (_openBlock(), _createElementBlock("div", { class: "space-y-8" }, [ (sortedDependencies.value.length > 0) ? (_openBlock(), _createBlock(_component_CollapsibleSection, {
          key: 0,
          id: "dependencies",
          title: 
          _ctx.$t(
            'package.dependencies.title',
            {
              count: _unref(numberFormatter).format(sortedDependencies.value.length),
            },
            sortedDependencies.value.length,
          )
      
        }, {
          default: _withCtx(() => [
            _createElementVNode("ul", {
              class: "px-1 space-y-1 list-none m-0",
              "aria-label": _ctx.$t('package.dependencies.list_label')
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(sortedDependencies.value.slice(0, depsExpanded.value ? undefined : 10), ([dep, version]) => {
                return (_openBlock(), _createElementBlock("li", {
                  key: dep,
                  class: "flex items-center justify-between py-1 text-sm gap-2"
                }, [
                  _createVNode(_component_LinkBase, {
                    to: _ctx.packageRoute(dep),
                    class: "block truncate",
                    dir: "ltr"
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(dep), 1 /* TEXT */)
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 8 /* PROPS */, ["to"]),
                  _createElementVNode("span", {
                    class: "flex items-center gap-1 max-w-[40%]",
                    dir: "ltr"
                  }, [
                    (_unref(outdatedDeps)[dep])
                      ? (_openBlock(), _createBlock(_component_TooltipApp, {
                        key: 0,
                        class: _normalizeClass(["shrink-0", _unref(getVersionClass)(_unref(outdatedDeps)[dep])]),
                        text: _unref(getOutdatedTooltip)(_unref(outdatedDeps)[dep], _ctx.$t)
                      }, {
                        default: _withCtx(() => [
                          _createElementVNode("button", {
                            type: "button",
                            class: "p-2 -m-2",
                            "aria-label": _unref(getOutdatedTooltip)(_unref(outdatedDeps)[dep], _ctx.$t)
                          }, [
                            _hoisted_1
                          ], 8 /* PROPS */, ["aria-label"])
                        ]),
                        _: 2 /* DYNAMIC */
                      }, 10 /* CLASS, PROPS */, ["text"]))
                      : _createCommentVNode("v-if", true),
                    (_unref(replacementDeps)[dep])
                      ? (_openBlock(), _createBlock(_component_TooltipApp, {
                        key: 0,
                        class: "shrink-0 text-amber-700 dark:text-amber-500",
                        text: _ctx.$t('package.dependencies.has_replacement')
                      }, {
                        default: _withCtx(() => [
                          _createElementVNode("button", {
                            type: "button",
                            class: "p-2 -m-2",
                            "aria-label": _ctx.$t('package.dependencies.has_replacement')
                          }, [
                            _hoisted_2
                          ], 8 /* PROPS */, ["aria-label"])
                        ]),
                        _: 2 /* DYNAMIC */
                      }, 8 /* PROPS */, ["text"]))
                      : _createCommentVNode("v-if", true),
                    (getVulnerableDepInfo(dep))
                      ? (_openBlock(), _createBlock(_component_LinkBase, {
                        key: 0,
                        to: _ctx.packageRoute(dep, getVulnerableDepInfo(dep).version),
                        class: _normalizeClass(["shrink-0", _unref(SEVERITY_TEXT_COLORS)[_unref(getHighestSeverity)(getVulnerableDepInfo(dep).counts)]]),
                        title: `${getVulnerableDepInfo(dep).counts.total} vulnerabilities`,
                        classicon: "i-lucide:shield-check"
                      }, {
                        default: _withCtx(() => [
                          _createElementVNode("span", _hoisted_3, _toDisplayString(_ctx.$t('package.dependencies.view_vulnerabilities')), 1 /* TEXT */)
                        ]),
                        _: 2 /* DYNAMIC */
                      }, 10 /* CLASS, PROPS */, ["to", "title"]))
                      : _createCommentVNode("v-if", true),
                    (getDeprecatedDepInfo(dep))
                      ? (_openBlock(), _createBlock(_component_LinkBase, {
                        key: 0,
                        to: _ctx.packageRoute(dep, getDeprecatedDepInfo(dep).version),
                        class: "shrink-0 text-purple-700 dark:text-purple-500",
                        title: getDeprecatedDepInfo(dep).message,
                        classicon: "i-lucide:octagon-alert"
                      }, {
                        default: _withCtx(() => [
                          _createElementVNode("span", _hoisted_4, _toDisplayString(_ctx.$t('package.deprecated.label')), 1 /* TEXT */)
                        ]),
                        _: 2 /* DYNAMIC */
                      }, 8 /* PROPS */, ["to", "title"]))
                      : _createCommentVNode("v-if", true),
                    _createVNode(_component_LinkBase, {
                      to: _ctx.packageRoute(dep, __props.version),
                      class: _normalizeClass(["block truncate", getDepVersionClass(dep)]),
                      title: getDepVersionTooltip(dep, __props.version)
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(__props.version), 1 /* TEXT */)
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 8 /* PROPS */, ["to", "title"]),
                    (_unref(outdatedDeps)[dep])
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: "sr-only"
                      }, "\n              (" + _toDisplayString(_unref(getOutdatedTooltip)(_unref(outdatedDeps)[dep], _ctx.$t)) + ")\n            ", 1 /* TEXT */))
                      : _createCommentVNode("v-if", true),
                    (getVulnerableDepInfo(dep))
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: "sr-only"
                      }, "\n              (" + _toDisplayString(getVulnerableDepInfo(dep).counts.total) + " vulnerabilities)\n            ", 1 /* TEXT */))
                      : _createCommentVNode("v-if", true)
                  ])
                ]))
              }), 128 /* KEYED_FRAGMENT */))
            ], 8 /* PROPS */, ["aria-label"]),
            (sortedDependencies.value.length > 10 && !depsExpanded.value)
              ? (_openBlock(), _createElementBlock("button", {
                key: 0,
                type: "button",
                class: "my-2 ms-1 font-mono text-xs text-fg-muted hover:text-fg transition-colors duration-200 rounded focus-visible:outline-accent/70",
                onClick: _cache[0] || (_cache[0] = ($event: any) => (depsExpanded.value = true))
              }, _toDisplayString(_ctx.$t(
              'package.dependencies.show_all',
              {
                count: _unref(numberFormatter).format(sortedDependencies.value.length),
              },
              sortedDependencies.value.length,
            )), 1 /* TEXT */))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["title"])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    " + "\n    "), (sortedPeerDependencies.value.length > 0) ? (_openBlock(), _createBlock(_component_CollapsibleSection, {
          key: 0,
          id: "peer-dependencies",
          title: 
          _ctx.$t('package.peer_dependencies.title', {
            count: _unref(numberFormatter).format(sortedPeerDependencies.value.length),
          })
      
        }, {
          default: _withCtx(() => [
            _createElementVNode("ul", {
              class: "px-1 space-y-1 list-none m-0",
              "aria-label": _ctx.$t('package.peer_dependencies.list_label')
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(sortedPeerDependencies.value.slice(0, peerDepsExpanded.value ? undefined : 10), (peer) => {
                return (_openBlock(), _createElementBlock("li", {
                  key: peer.name,
                  class: "flex items-center justify-between py-1 text-sm gap-1 min-w-0"
                }, [
                  _createElementVNode("div", { class: "flex items-center gap-1 min-w-0 flex-1" }, [
                    _createVNode(_component_LinkBase, {
                      to: _ctx.packageRoute(peer.name),
                      class: "block truncate",
                      dir: "ltr"
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(peer.name), 1 /* TEXT */)
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 8 /* PROPS */, ["to"]),
                    (peer.optional)
                      ? (_openBlock(), _createBlock(_component_TagStatic, {
                        key: 0,
                        title: _ctx.$t('package.dependencies.optional')
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_ctx.$t('package.dependencies.optional')), 1 /* TEXT */)
                        ]),
                        _: 2 /* DYNAMIC */
                      }, 8 /* PROPS */, ["title"]))
                      : _createCommentVNode("v-if", true)
                  ]),
                  _createVNode(_component_LinkBase, {
                    to: _ctx.packageRoute(peer.name, peer.version),
                    class: "block truncate max-w-[40%]",
                    title: peer.version,
                    dir: "ltr"
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(peer.version), 1 /* TEXT */)
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 8 /* PROPS */, ["to", "title"])
                ]))
              }), 128 /* KEYED_FRAGMENT */))
            ], 8 /* PROPS */, ["aria-label"]),
            (sortedPeerDependencies.value.length > 10 && !peerDepsExpanded.value)
              ? (_openBlock(), _createElementBlock("button", {
                key: 0,
                type: "button",
                class: "mt-2 font-mono text-xs text-fg-muted hover:text-fg transition-colors duration-200 rounded focus-visible:outline-accent/70",
                onClick: _cache[1] || (_cache[1] = ($event: any) => (peerDepsExpanded.value = true))
              }, _toDisplayString(_ctx.$t(
              'package.peer_dependencies.show_all',
              {
                count: _unref(numberFormatter).format(sortedPeerDependencies.value.length),
              },
              sortedPeerDependencies.value.length,
            )), 1 /* TEXT */))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["title"])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    " + "\n    "), (sortedOptionalDependencies.value.length > 0) ? (_openBlock(), _createBlock(_component_CollapsibleSection, {
          key: 0,
          id: "optional-dependencies",
          title: 
          _ctx.$t(
            'package.optional_dependencies.title',
            {
              count: _unref(numberFormatter).format(sortedOptionalDependencies.value.length),
            },
            sortedOptionalDependencies.value.length,
          )
      
        }, {
          default: _withCtx(() => [
            _createElementVNode("ul", {
              class: "px-1 space-y-1 list-none m-0",
              "aria-label": _ctx.$t('package.optional_dependencies.list_label')
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(sortedOptionalDependencies.value.slice(
              0,
              optionalDepsExpanded.value ? undefined : 10,
            ), ([dep, version]) => {
                return (_openBlock(), _createElementBlock("li", {
                  key: dep,
                  class: "flex items-center justify-between py-1 text-sm gap-2"
                }, [
                  _createVNode(_component_LinkBase, {
                    to: _ctx.packageRoute(dep),
                    class: "block truncate",
                    dir: "ltr"
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(dep), 1 /* TEXT */)
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 8 /* PROPS */, ["to"]),
                  _createVNode(_component_LinkBase, {
                    to: _ctx.packageRoute(dep, __props.version),
                    class: "block truncate",
                    title: __props.version,
                    dir: "ltr"
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(__props.version), 1 /* TEXT */)
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 8 /* PROPS */, ["to", "title"])
                ]))
              }), 128 /* KEYED_FRAGMENT */))
            ], 8 /* PROPS */, ["aria-label"]),
            (sortedOptionalDependencies.value.length > 10 && !optionalDepsExpanded.value)
              ? (_openBlock(), _createElementBlock("button", {
                key: 0,
                type: "button",
                class: "mt-2 truncate",
                onClick: _cache[2] || (_cache[2] = ($event: any) => (optionalDepsExpanded.value = true))
              }, _toDisplayString(_ctx.$t(
              'package.optional_dependencies.show_all',
              {
                count: _unref(numberFormatter).format(sortedOptionalDependencies.value.length),
              },
              sortedOptionalDependencies.value.length,
            )), 1 /* TEXT */))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["title"])) : _createCommentVNode("v-if", true) ]))
}
}

})
