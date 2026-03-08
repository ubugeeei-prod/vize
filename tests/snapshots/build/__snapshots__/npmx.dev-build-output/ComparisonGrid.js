import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { class: "comparison-label" })
const _hoisted_2 = { class: "text-sm font-medium text-fg mb-1" }
import type { ModuleReplacement } from 'module-replacements'

export interface ComparisonGridColumn {
  name: string
  version?: string
  /** Module replacement data for this package (if available) */
  replacement?: ModuleReplacement | null
}

export default /*@__PURE__*/_defineComponent({
  __name: 'ComparisonGrid',
  props: {
    columns: { type: Array, required: true },
    showNoDependency: { type: Boolean, required: false }
  },
  setup(__props: any) {

const props = __props
/** Total column count including the optional no-dep column */
const totalColumns = computed(() => props.columns.length + (props.showNoDependency ? 1 : 0))
/** Compute plain-text tooltip for a replacement column */
function getReplacementTooltip(col: ComparisonGridColumn): string {
  if (!col.replacement) return ''
  return [$t('package.replacement.title'), $t('package.replacement.learn_more_above')].join(' ')
}

return (_ctx: any,_cache: any) => {
  const _component_LinkBase = _resolveComponent("LinkBase")
  const _component_TooltipApp = _resolveComponent("TooltipApp")
  const _component_i18n_t = _resolveComponent("i18n-t")

  return (_openBlock(), _createElementBlock("div", { class: "overflow-x-auto" }, [ _createElementVNode("div", {
        class: _normalizeClass(["comparison-grid", [totalColumns.value === 4 ? 'min-w-[800px]' : 'min-w-[600px]', `columns-${totalColumns.value}`]]),
        style: _normalizeStyle({ '--columns': totalColumns.value })
      }, [ _createElementVNode("div", { class: "comparison-header" }, [ _hoisted_1, _createTextVNode("\n\n        " + "\n        "), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.columns, (col) => {
            return (_openBlock(), _createElementBlock("div", {
              key: col.name,
              class: "comparison-cell comparison-cell-header"
            }, [
              _createElementVNode("span", { class: "inline-flex items-center gap-1.5 truncate" }, [
                _createVNode(_component_LinkBase, {
                  to: _ctx.packageRoute(col.name, col.version),
                  class: "text-sm truncate",
                  block: "",
                  title: col.version ? `${col.name}@${col.version}` : col.name
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(col.name), 1 /* TEXT */),
                    (col.version)
                      ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                        _createTextVNode("@"),
                        _toDisplayString(col.version)
                      ], 64 /* STABLE_FRAGMENT */))
                      : _createCommentVNode("v-if", true)
                  ]),
                  _: 2 /* DYNAMIC */
                }, 8 /* PROPS */, ["to", "title"]),
                (col.replacement)
                  ? (_openBlock(), _createBlock(_component_TooltipApp, {
                    key: 0,
                    text: getReplacementTooltip(col),
                    position: "bottom"
                  }, {
                    default: _withCtx(() => [
                      _createElementVNode("span", {
                        class: "i-lucide:lightbulb w-3.5 h-3.5 text-amber-500 shrink-0 cursor-help",
                        role: "img",
                        "aria-label": _ctx.$t('package.replacement.title')
                      }, null, 8 /* PROPS */, ["aria-label"])
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 8 /* PROPS */, ["text"]))
                  : _createCommentVNode("v-if", true)
              ])
            ]))
          }), 128 /* KEYED_FRAGMENT */)), _createTextVNode("\n\n        " + "\n        "), (__props.showNoDependency) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "comparison-cell comparison-cell-header comparison-cell-nodep"
            }, [ _createElementVNode("span", { class: "inline-flex items-center gap-1.5 text-sm font-medium text-accent italic truncate" }, [ _createTextVNode(_toDisplayString(_ctx.$t('compare.no_dependency.label')) + "\n            ", 1 /* TEXT */), _createVNode(_component_TooltipApp, {
                  interactive: "",
                  position: "bottom"
                }, {
                  content: _withCtx(() => [
                    _createElementVNode("p", _hoisted_2, _toDisplayString(_ctx.$t('compare.no_dependency.tooltip_title')), 1 /* TEXT */),
                    _createElementVNode("p", { class: "text-xs text-fg-muted" }, [
                      _createVNode(_component_i18n_t, {
                        keypath: "compare.no_dependency.tooltip_description",
                        tag: "span"
                      }, {
                        link: _withCtx(() => [
                          _createVNode(_component_LinkBase, { to: "https://e18e.dev/docs/replacements/" }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_ctx.$t('compare.no_dependency.e18e_community')), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]),
                        _: 1 /* STABLE */
                      })
                    ])
                  ]),
                  default: _withCtx(() => [
                    _createElementVNode("span", {
                      class: "i-lucide:lightbulb w-3.5 h-3.5 text-amber-500 shrink-0 cursor-help",
                      role: "img",
                      "aria-label": _ctx.$t('compare.no_dependency.tooltip_title')
                    }, null, 8 /* PROPS */, ["aria-label"])
                  ]),
                  _: 1 /* STABLE */
                }) ]) ])) : _createCommentVNode("v-if", true) ]), _createTextVNode("\n\n      " + "\n      "), _renderSlot(_ctx.$slots, "default") ], 6 /* CLASS, STYLE */) ]))
}
}

})
