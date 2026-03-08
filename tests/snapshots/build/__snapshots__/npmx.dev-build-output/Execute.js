import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "w-2.5 h-2.5 rounded-full bg-fg-subtle" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "w-2.5 h-2.5 rounded-full bg-fg-subtle" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "w-2.5 h-2.5 rounded-full bg-fg-subtle" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle font-mono text-sm select-none" }, "$")
import type { JsrPackageInfo } from '#shared/types/jsr'
import type { PackageManagerId } from '~/utils/install-command'

export default /*@__PURE__*/_defineComponent({
  __name: 'Execute',
  props: {
    packageName: { type: String, required: true },
    jsrInfo: { type: null, required: false },
    isCreatePackage: { type: Boolean, required: false }
  },
  setup(__props: any) {

const props = __props
/**
 * A terminal-style execute command display for binary-only packages.
 * Renders all package manager variants with CSS-based visibility.
 */
const selectedPM = useSelectedPackageManager()
// Generate execute command parts for a specific package manager
function getExecutePartsForPM(pmId: PackageManagerId) {
  return getExecuteCommandParts({
    packageName: props.packageName,
    packageManager: pmId,
    jsrInfo: props.jsrInfo,
    isBinaryOnly: true,
    isCreatePackage: props.isCreatePackage,
  })
}
// Full execute command for copying (uses current selected PM)
function getFullExecuteCommand() {
  return getExecuteCommand({
    packageName: props.packageName,
    packageManager: selectedPM.value,
    jsrInfo: props.jsrInfo,
    isBinaryOnly: true,
    isCreatePackage: props.isCreatePackage,
  })
}
// Copy handler
const { copied: executeCopied, copy: copyExecute } = useClipboard({ copiedDuring: 2000 })
const copyExecuteCommand = () => copyExecute(getFullExecuteCommand())

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "relative group" }, [ _createElementVNode("div", { class: "bg-bg-subtle border border-border rounded-lg overflow-hidden" }, [ _createElementVNode("div", { class: "flex gap-1.5 px-3 pt-2 sm:px-4 sm:pt-3" }, [ _hoisted_1, _hoisted_2, _hoisted_3 ]), _createElementVNode("div", { class: "px-3 pt-2 pb-3 sm:px-4 sm:pt-3 sm:pb-4 space-y-1" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.packageManagers, (pm) => {
            return (_openBlock(), _createElementBlock("div", {
              key: `execute-${pm.id}`,
              "data-pm-cmd": pm.id,
              class: "flex items-center gap-2 group/executecmd"
            }, [
              _hoisted_4,
              _createElementVNode("code", { class: "font-mono text-sm" }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(getExecutePartsForPM(pm.id), (part, i) => {
                  return (_openBlock(), _createElementBlock("span", {
                    key: i,
                    class: _normalizeClass(i === 0 ? 'text-fg' : 'text-fg-muted')
                  }, _toDisplayString(i > 0 ? ' ' : '') + _toDisplayString(part), 3 /* TEXT, CLASS */))
                }), 128 /* KEYED_FRAGMENT */))
              ]),
              _createElementVNode("button", {
                type: "button",
                class: "px-2 py-0.5 font-mono text-xs text-fg-muted bg-bg-subtle/80 border border-border rounded transition-colors duration-200 opacity-0 group-hover/executecmd:opacity-100 hover:(text-fg border-border-hover) active:scale-95 focus-visible:opacity-100 focus-visible:outline-accent/70",
                "aria-label": _ctx.$t('package.get_started.copy_command'),
                onClick: _withModifiers(copyExecuteCommand, ["stop"])
              }, _toDisplayString(_unref(executeCopied) ? _ctx.$t('common.copied') : _ctx.$t('common.copy')), 9 /* TEXT, PROPS */, ["aria-label"])
            ], 8 /* PROPS */, ["data-pm-cmd"]))
          }), 128 /* KEYED_FRAGMENT */)) ]) ]) ]))
}
}

})
