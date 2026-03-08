import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:octagon-alert w-4 h-4 shrink-0", "aria-hidden": "true" })
const _hoisted_2 = { class: "font-mono text-sm font-medium truncate" }
const _hoisted_3 = { class: "text-xs text-fg-muted m-0 line-clamp-2" }
import type { DependencyDepth } from '#shared/types'
const bannerColor = 'border-purple-600/40 bg-purple-500/10 text-purple-700 dark:text-purple-400'

export default /*@__PURE__*/_defineComponent({
  __name: 'DeprecatedTree',
  props: {
    packageName: { type: String, required: true },
    version: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const { data: analysisData, status } = useDependencyAnalysis(
  () => props.packageName,
  () => props.version,
)
const isExpanded = shallowRef(false)
const showAll = shallowRef(false)
const hasDeprecated = computed(
  () => analysisData.value?.deprecatedPackages && analysisData.value.deprecatedPackages.length > 0,
)
// Banner color - purple for deprecated
// Styling for each depth level
const depthStyles = {
  root: {
    bg: 'bg-purple-500/5 border-is-2 border-is-purple-600',
    text: 'text-fg',
  },
  direct: {
    bg: 'bg-purple-500/5 border-is-2 border-is-purple-500',
    text: 'text-fg-muted',
  },
  transitive: {
    bg: 'bg-purple-500/5 border-is-2 border-is-purple-400',
    text: 'text-fg-muted',
  },
} as const
function getDepthStyle(depth: DependencyDepth) {
  return depthStyles[depth] || depthStyles.transitive
}

return (_ctx: any,_cache: any) => {
  const _component_DependencyPathPopup = _resolveComponent("DependencyPathPopup")
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (_unref(status) === 'success' && hasDeprecated.value)
      ? (_openBlock(), _createElementBlock("section", {
        key: 0,
        class: "relative"
      }, [ _createElementVNode("div", {
          class: _normalizeClass(["rounded-lg border overflow-hidden", bannerColor])
        }, [ _createElementVNode("button", {
            type: "button",
            class: "w-full flex items-center justify-between gap-3 px-4 py-3 text-start transition-colors duration-200 hover:bg-white/5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-accent/70",
            "aria-expanded": isExpanded.value,
            "aria-controls": "deprecated-tree-details",
            onClick: _cache[0] || (_cache[0] = ($event: any) => (isExpanded.value = !isExpanded.value))
          }, [ _createElementVNode("span", { class: "flex items-center gap-2 min-w-0" }, [ _hoisted_1, _createElementVNode("span", _hoisted_2, _toDisplayString(_ctx.$t("package.deprecated.tree_found", _unref(analysisData).deprecatedPackages.length)), 1 /* TEXT */) ]), _createElementVNode("span", {
              class: _normalizeClass(["i-lucide:chevron-down w-4 h-4 transition-transform duration-200 shrink-0", { 'rotate-180': isExpanded.value }]),
              "aria-hidden": "true"
            }, null, 2 /* CLASS */) ], 8 /* PROPS */, ["aria-expanded"]), _createTextVNode("\n\n      " + "\n      "), _withDirectives(_createElementVNode("div", {
            id: "deprecated-tree-details",
            class: "border-t border-border bg-bg-subtle"
          }, [ _createElementVNode("ul", { class: "divide-y divide-border list-none m-0 p-0" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(analysisData).deprecatedPackages.slice(0, showAll.value ? undefined : 5), (pkg) => {
                return (_openBlock(), _createElementBlock("li", {
                  key: `${pkg.name}@${pkg.version}`,
                  class: _normalizeClass(["px-4 py-3", getDepthStyle(pkg.depth).bg])
                }, [
                  _createElementVNode("div", { class: "flex items-center gap-2 mb-1" }, [
                    (pkg.path && pkg.path.length > 1)
                      ? (_openBlock(), _createBlock(_component_DependencyPathPopup, {
                        key: 0,
                        path: pkg.path
                      }, null, 8 /* PROPS */, ["path"]))
                      : _createCommentVNode("v-if", true),
                    _createVNode(_component_NuxtLink, {
                      to: _ctx.packageRoute(pkg.name, pkg.version),
                      class: _normalizeClass(["font-mono text-sm font-medium hover:underline truncate py-4", getDepthStyle(pkg.depth).text])
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(pkg.name), 1 /* TEXT */),
                        _createTextVNode("@"),
                        _createTextVNode(_toDisplayString(pkg.version), 1 /* TEXT */)
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 10 /* CLASS, PROPS */, ["to"])
                  ]),
                  _createElementVNode("p", _hoisted_3, _toDisplayString(pkg.message), 1 /* TEXT */)
                ], 2 /* CLASS */))
              }), 128 /* KEYED_FRAGMENT */)) ]), (_unref(analysisData).deprecatedPackages.length > 5 && !showAll.value) ? (_openBlock(), _createElementBlock("button", {
                key: 0,
                type: "button",
                class: "w-full px-4 py-2 text-xs font-mono text-fg-muted hover:text-fg border-t border-border transition-colors duration-200",
                onClick: _cache[1] || (_cache[1] = ($event: any) => (showAll.value = true))
              }, _toDisplayString(_ctx.$t("package.deprecated.show_all", { count: _unref(analysisData).deprecatedPackages.length })), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 512 /* NEED_PATCH */), [ [_vShow, isExpanded.value] ]) ]) ]))
      : _createCommentVNode("v-if", true)
}
}

})
