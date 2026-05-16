import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "w-2.5 h-2.5 rounded-full bg-fg-subtle" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "w-2.5 h-2.5 rounded-full bg-fg-subtle" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "w-2.5 h-2.5 rounded-full bg-fg-subtle" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "self-start text-fg-subtle font-mono text-sm select-none shrink-0" }, "$")
const _hoisted_5 = { "aria-live": "polite" }
const _hoisted_6 = { class: "text-fg-subtle font-mono text-sm" }
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle font-mono text-sm select-none shrink-0" }, "$")
const _hoisted_8 = { "aria-live": "polite" }
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("span", { class: "self-start text-fg-subtle font-mono text-sm select-none shrink-0" }, "$")
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:arrow-right rtl-flip w-3 h-3 align-middle", "aria-hidden": "true" })
const _hoisted_11 = { class: "sr-only" }
const _hoisted_12 = { class: "text-fg-subtle font-mono text-sm select-none" }
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("span", { class: "self-start text-fg-subtle font-mono text-sm select-none" }, "$")
const _hoisted_14 = { class: "text-fg-subtle font-mono text-sm" }
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:info w-3 h-3", "aria-hidden": "true" })
const _hoisted_16 = { class: "sr-only" }
const _hoisted_17 = /*#__PURE__*/ _createElementVNode("span", { class: "self-start text-fg-subtle font-mono text-sm select-none" }, "$")
const _hoisted_18 = { "aria-live": "polite" }
import type { JsrPackageInfo } from '#shared/types/jsr'
import type { DevDependencySuggestion } from '#shared/utils/dev-dependency'
import type { PackageManagerId } from '~/utils/install-command'

export default /*@__PURE__*/_defineComponent({
  __name: 'Install',
  props: {
    packageName: { type: String, required: true },
    requestedVersion: { type: String, required: false },
    installVersionOverride: { type: String, required: false },
    jsrInfo: { type: null, required: false },
    devDependencySuggestion: { type: null, required: false },
    typesPackageName: { type: String, required: false },
    executableInfo: { type: Object, required: false },
    createPackageInfo: { type: Object, required: false }
  },
  setup(__props: any) {

const props = __props
const { selectedPM, showTypesInInstall, copied, copyInstallCommand } = useInstallCommand(
  () => props.packageName,
  () => props.requestedVersion ?? null,
  () => props.jsrInfo ?? null,
  () => props.typesPackageName ?? null,
  () => props.installVersionOverride ?? null,
)
// Generate install command parts for a specific package manager
function getInstallPartsForPM(pmId: PackageManagerId) {
  return getInstallCommandParts({
    packageName: props.packageName,
    packageManager: pmId,
    version: props.installVersionOverride ?? props.requestedVersion,
    jsrInfo: props.jsrInfo,
  })
}
const devDependencySuggestion = computed(
  () => props.devDependencySuggestion ?? { recommended: false as const },
)
function getDevInstallPartsForPM(pmId: PackageManagerId) {
  return getInstallCommandParts({
    packageName: props.packageName,
    packageManager: pmId,
    version: props.requestedVersion,
    jsrInfo: props.jsrInfo,
    dev: true,
  })
}
// Generate run command parts for a specific package manager
function getRunPartsForPM(pmId: PackageManagerId, command?: string) {
  return getRunCommandParts({
    packageName: props.packageName,
    packageManager: pmId,
    jsrInfo: props.jsrInfo,
    command,
    isBinaryOnly: false,
  })
}
// Generate create command parts for a specific package manager
function getCreatePartsForPM(pmId: PackageManagerId) {
  if (!props.createPackageInfo) return []
  const pm = packageManagers.find(p => p.id === pmId)
  if (!pm) return []
  const createPkgName = props.createPackageInfo.packageName
  let shortName: string
  if (createPkgName.startsWith('@')) {
    const slashIndex = createPkgName.indexOf('/')
    const name = createPkgName.slice(slashIndex + 1)
    shortName = name.startsWith('create-') ? name.slice('create-'.length) : name
  } else {
    shortName = createPkgName.startsWith('create-')
      ? createPkgName.slice('create-'.length)
      : createPkgName
  }
  return [...pm.create.split(' '), shortName]
}
// Generate @types install command parts for a specific package manager
function getTypesInstallPartsForPM(pmId: PackageManagerId) {
  if (!props.typesPackageName) return []
  const pm = packageManagers.find(p => p.id === pmId)
  if (!pm) return []
  const devFlag = getDevDependencyFlag(pmId)
  const pkgSpec = pmId === 'deno' ? `npm:${props.typesPackageName}` : props.typesPackageName
  return [pm.label, pm.action, devFlag, pkgSpec]
}
// Full run command for copying (uses current selected PM)
function getFullRunCommand(command?: string) {
  return getRunCommand({
    packageName: props.packageName,
    packageManager: selectedPM.value,
    jsrInfo: props.jsrInfo,
    command,
  })
}
// Full create command for copying (uses current selected PM)
function getFullCreateCommand() {
  return getCreatePartsForPM(selectedPM.value).join(' ')
}
// Copy handlers
const { copied: runCopied, copy: copyRun } = useClipboard({ copiedDuring: 2000 })
const copyRunCommand = (command?: string) => copyRun(getFullRunCommand(command))
const { copied: createCopied, copy: copyCreate } = useClipboard({ copiedDuring: 2000 })
const copyCreateCommand = () => copyCreate(getFullCreateCommand())
const { copied: devInstallCopied, copy: copyDevInstall } = useClipboard({ copiedDuring: 2000 })
const copyDevInstallCommand = () =>
  copyDevInstall(
    getInstallCommand({
      packageName: props.packageName,
      packageManager: selectedPM.value,
      version: props.requestedVersion,
      jsrInfo: props.jsrInfo,
      dev: true,
    }),
  )

return (_ctx: any,_cache: any) => {
  const _component_ButtonBase = _resolveComponent("ButtonBase")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_TooltipApp = _resolveComponent("TooltipApp")

  return (_openBlock(), _createElementBlock("div", { class: "relative group" }, [ _createElementVNode("div", { class: "bg-bg-subtle border border-border rounded-lg overflow-hidden" }, [ _createElementVNode("div", { class: "flex gap-1.5 px-3 pt-2 sm:px-4 sm:pt-3" }, [ _hoisted_1, _hoisted_2, _hoisted_3 ]), _createElementVNode("div", {
          class: "px-3 pt-2 pb-3 sm:px-4 sm:pt-3 sm:pb-4 space-y-1 overflow-x-auto",
          dir: "ltr"
        }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.packageManagers, (pm) => {
            return (_openBlock(), _createElementBlock("div", {
              key: `install-${pm.id}`,
              "data-pm-cmd": pm.id,
              class: "flex items-center gap-2 group/installcmd min-w-0"
            }, [
              _hoisted_4,
              _createElementVNode("code", { class: "font-mono text-sm min-w-0" }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(getInstallPartsForPM(pm.id), (part, i) => {
                  return (_openBlock(), _createElementBlock("span", {
                    key: i,
                    class: _normalizeClass(i === 0 ? 'text-fg' : 'text-fg-muted')
                  }, _toDisplayString(i > 0 ? ' ' : '') + _toDisplayString(part), 3 /* TEXT, CLASS */))
                }), 128 /* KEYED_FRAGMENT */))
              ]),
              _createElementVNode("button", {
                type: "button",
                class: "px-2 py-0.5 font-mono text-xs text-fg-muted bg-bg-subtle/80 border border-border rounded transition-colors duration-200 opacity-0 group-hover/installcmd:opacity-100 hover:(text-fg border-border-hover) active:scale-95 focus-visible:opacity-100 focus-visible:outline-accent/70 select-none",
                "aria-label": _ctx.$t('package.get_started.copy_command'),
                onClick: _cache[0] || (_cache[0] = _withModifiers((...args) => (copyInstallCommand && copyInstallCommand(...args)), ["stop"]))
              }, [
                _createElementVNode("span", _hoisted_5, _toDisplayString(_unref(copied) ? _ctx.$t('common.copied') : _ctx.$t('common.copy')), 1 /* TEXT */)
              ], 8 /* PROPS */, ["aria-label"])
            ], 8 /* PROPS */, ["data-pm-cmd"]))
          }), 128 /* KEYED_FRAGMENT */)), _createTextVNode("\n\n        " + "\n        "), (devDependencySuggestion.value.recommended) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createElementVNode("div", { class: "flex items-center gap-2 pt-1 select-none" }, [ _createElementVNode("span", _hoisted_6, "# " + _toDisplayString(_ctx.$t('package.get_started.dev_dependency_hint')), 1 /* TEXT */) ]), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.packageManagers, (pm) => {
                return (_openBlock(), _createElementBlock("div", {
                  key: `install-dev-${pm.id}`,
                  "data-pm-cmd": pm.id,
                  class: "flex items-center gap-2 group/devinstallcmd min-w-0"
                }, [
                  _hoisted_7,
                  _createElementVNode("code", { class: "font-mono text-sm min-w-0" }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(getDevInstallPartsForPM(pm.id), (part, i) => {
                      return (_openBlock(), _createElementBlock("span", {
                        key: i,
                        class: _normalizeClass(i === 0 ? 'text-fg' : 'text-fg-muted')
                      }, _toDisplayString(i > 0 ? ' ' : '') + _toDisplayString(part), 3 /* TEXT, CLASS */))
                    }), 128 /* KEYED_FRAGMENT */))
                  ]),
                  _createVNode(_component_ButtonBase, {
                    type: "button",
                    size: "small",
                    class: "text-fg-muted bg-bg-subtle/80 border-border opacity-0 group-hover/devinstallcmd:opacity-100 active:scale-95 focus-visible:opacity-100 select-none",
                    "aria-label": _ctx.$t('package.get_started.copy_dev_command'),
                    onClick: _withModifiers(copyDevInstallCommand, ["stop"])
                  }, {
                    default: _withCtx(() => [
                      _createElementVNode("span", _hoisted_8, _toDisplayString(_unref(devInstallCopied) ? _ctx.$t('common.copied') : _ctx.$t('common.copy')), 1 /* TEXT */)
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 8 /* PROPS */, ["aria-label"])
                ], 8 /* PROPS */, ["data-pm-cmd"]))
              }), 128 /* KEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true), _createTextVNode("\n\n        " + "\n        "), (__props.typesPackageName && _unref(showTypesInInstall)) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.packageManagers, (pm) => {
                return (_openBlock(), _createElementBlock("div", {
                  key: `types-${pm.id}`,
                  "data-pm-cmd": pm.id,
                  class: "flex items-center gap-2 min-w-0"
                }, [
                  _hoisted_9,
                  _createElementVNode("code", { class: "font-mono text-sm min-w-0" }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(getTypesInstallPartsForPM(pm.id), (part, i) => {
                      return (_openBlock(), _createElementBlock("span", {
                        key: i,
                        class: _normalizeClass(i === 0 ? 'text-fg' : 'text-fg-muted')
                      }, _toDisplayString(i > 0 ? ' ' : '') + _toDisplayString(part), 3 /* TEXT, CLASS */))
                    }), 128 /* KEYED_FRAGMENT */))
                  ]),
                  _createVNode(_component_NuxtLink, {
                    to: _ctx.packageRoute(__props.typesPackageName),
                    class: "text-fg-subtle hover:text-fg-muted text-xs transition-colors focus-visible:outline-accent/70 rounded select-none",
                    title: _ctx.$t('package.get_started.view_types', { package: __props.typesPackageName })
                  }, {
                    default: _withCtx(() => [
                      _hoisted_10,
                      _createElementVNode("span", _hoisted_11, "View " + _toDisplayString(__props.typesPackageName), 1 /* TEXT */)
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 8 /* PROPS */, ["to", "title"])
                ], 8 /* PROPS */, ["data-pm-cmd"]))
              }), 128 /* KEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true), _createTextVNode("\n\n        " + "\n        "), (__props.executableInfo?.hasExecutable) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createElementVNode("div", {
                class: "flex items-center gap-2 pt-1",
                dir: "auto"
              }, [ _createElementVNode("span", _hoisted_12, "# " + _toDisplayString(_ctx.$t('package.run.locally')), 1 /* TEXT */) ]), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.packageManagers, (pm) => {
                return (_openBlock(), _createElementBlock("div", {
                  key: `run-${pm.id}`,
                  "data-pm-cmd": pm.id,
                  class: "flex items-center gap-2 group/runcmd"
                }, [
                  _hoisted_13,
                  _createElementVNode("code", { class: "font-mono text-sm" }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(getRunPartsForPM(pm.id, __props.executableInfo?.primaryCommand), (part, i) => {
                      return (_openBlock(), _createElementBlock("span", {
                        key: i,
                        class: _normalizeClass(i === 0 ? 'text-fg' : 'text-fg-muted')
                      }, _toDisplayString(i > 0 ? ' ' : '') + _toDisplayString(part), 3 /* TEXT, CLASS */))
                    }), 128 /* KEYED_FRAGMENT */))
                  ]),
                  _createElementVNode("button", {
                    type: "button",
                    class: "px-2 py-0.5 font-mono text-xs text-fg-muted bg-bg-subtle/80 border border-border rounded transition-colors duration-200 opacity-0 group-hover/runcmd:opacity-100 hover:(text-fg border-border-hover) active:scale-95 focus-visible:opacity-100 focus-visible:outline-accent/70 select-none",
                    onClick: _cache[1] || (_cache[1] = _withModifiers(($event: any) => (copyRunCommand(__props.executableInfo?.primaryCommand)), ["stop"]))
                  }, _toDisplayString(_unref(runCopied) ? _ctx.$t('common.copied') : _ctx.$t('common.copy')), 1 /* TEXT */)
                ], 8 /* PROPS */, ["data-pm-cmd"]))
              }), 128 /* KEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true), _createTextVNode("\n\n        " + "\n        "), (__props.createPackageInfo) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createElementVNode("div", {
                class: "flex items-center gap-2 pt-1 select-none",
                dir: "auto"
              }, [ _createElementVNode("span", _hoisted_14, "# " + _toDisplayString(_ctx.$t('package.create.title')), 1 /* TEXT */), _createVNode(_component_TooltipApp, { text: _ctx.$t('package.create.view', { packageName: __props.createPackageInfo.packageName }) }, {
                  default: _withCtx(() => [
                    _createVNode(_component_NuxtLink, {
                      to: _ctx.packageRoute(__props.createPackageInfo.packageName),
                      class: "inline-flex items-center justify-center min-w-6 min-h-6 -m-1 p-1 text-fg-muted hover:text-fg text-xs transition-colors focus-visible:outline-2 focus-visible:outline-accent/70 rounded"
                    }, {
                      default: _withCtx(() => [
                        _hoisted_15,
                        _createElementVNode("span", _hoisted_16, _toDisplayString(_ctx.$t('package.create.view', { packageName: __props.createPackageInfo.packageName })), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["to"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["text"]) ]), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.packageManagers, (pm) => {
                return (_openBlock(), _createElementBlock("div", {
                  key: `create-${pm.id}`,
                  "data-pm-cmd": pm.id,
                  class: "flex items-center gap-2 group/createcmd"
                }, [
                  _hoisted_17,
                  _createElementVNode("code", { class: "font-mono text-sm" }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(getCreatePartsForPM(pm.id), (part, i) => {
                      return (_openBlock(), _createElementBlock("span", {
                        key: i,
                        class: _normalizeClass(i === 0 ? 'text-fg' : 'text-fg-muted')
                      }, _toDisplayString(i > 0 ? ' ' : '') + _toDisplayString(part), 3 /* TEXT, CLASS */))
                    }), 128 /* KEYED_FRAGMENT */))
                  ]),
                  _createElementVNode("button", {
                    type: "button",
                    class: "px-2 py-0.5 font-mono text-xs text-fg-muted bg-bg-subtle/80 border border-border rounded transition-colors duration-200 opacity-0 group-hover/createcmd:opacity-100 hover:(text-fg border-border-hover) active:scale-95 focus-visible:opacity-100 focus-visible:outline-accent/70 select-none",
                    "aria-label": _ctx.$t('package.create.copy_command'),
                    onClick: _withModifiers(copyCreateCommand, ["stop"])
                  }, [
                    _createElementVNode("span", _hoisted_18, _toDisplayString(_unref(createCopied) ? _ctx.$t('common.copied') : _ctx.$t('common.copy')), 1 /* TEXT */)
                  ], 8 /* PROPS */, ["aria-label"])
                ], 8 /* PROPS */, ["data-pm-cmd"]))
              }), 128 /* KEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ]) ]) ]))
}
}

})
