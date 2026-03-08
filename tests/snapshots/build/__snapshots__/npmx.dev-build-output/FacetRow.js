import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx } from "vue"


const _hoisted_1 = { class: "text-xs text-fg-muted uppercase tracking-wider" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:info w-3 h-3 text-fg-subtle cursor-help", "aria-hidden": "true" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "i-svg-spinners:ring-resize w-4 h-4 text-fg-subtle", "aria-hidden": "true" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle text-sm" }, "-")
const _hoisted_5 = { dir: "auto" }
const _hoisted_6 = { dir: "auto" }
import type { FacetValue } from '#shared/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'FacetRow',
  props: {
    label: { type: String, required: true },
    description: { type: String, required: false },
    values: { type: Array, required: true },
    facetLoading: { type: Boolean, required: false },
    columnLoading: { type: Array, required: false },
    bar: { type: Boolean, required: false }
  },
  setup(__props: any) {

const props = __props
// Check if all values are numeric (for bar visualization)
const isNumeric = computed(() => {
  return props.values.every(v => v === null || v === undefined || typeof v.raw === 'number')
})
// Show bar if explicitly enabled, or if not specified and values are numeric
const showBar = computed(() => {
  return props.bar ?? isNumeric.value
})
// Get max value for bar width calculation
const maxValue = computed(() => {
  if (!isNumeric.value) return 0
  return Math.max(...props.values.map(v => (typeof v?.raw === 'number' ? v.raw : 0)))
})
// Calculate bar width percentage for a value
function getBarWidth(value: FacetValue | null | undefined): number {
  if (!isNumeric.value || !maxValue.value || !value || typeof value.raw !== 'number') return 0
  return (value.raw / maxValue.value) * 100
}
function getStatusClass(status?: FacetValue['status'], hasBar = false): string {
  // When there's a bar, only apply text color, not background
  if (hasBar) {
    switch (status) {
      case 'muted':
        return 'text-fg-subtle'
      default:
        return 'text-fg'
    }
  }
  // Original behavior when no bar
  switch (status) {
    case 'good':
      return 'bg-emerald-400/20 px-3 py-0.5 rounded-xl'
    case 'info':
      return 'bg-blue-400/20 px-3 py-0.5 rounded-xl'
    case 'warning':
      return 'bg-amber-400/20 px-3 py-0.5 rounded-xl'
    case 'bad':
      return 'bg-red-400/20 px-3 py-0.5 rounded-xl'
    case 'muted':
      return 'text-fg-subtle'
    default:
      return 'text-fg'
  }
}
function getStatusBarClass(status?: FacetValue['status']): string {
  switch (status) {
    case 'good':
      return 'bg-emerald-500/20'
    case 'info':
      return 'bg-blue-500/20'
    case 'warning':
      return 'bg-amber-500/20'
    case 'bad':
      return 'bg-red-500/20'
    default:
      return 'bg-fg/5'
  }
}
// Check if a specific cell is loading
function isCellLoading(index: number): boolean {
  return props.facetLoading || (props.columnLoading?.[index] ?? false)
}

return (_ctx: any,_cache: any) => {
  const _component_TooltipApp = _resolveComponent("TooltipApp")
  const _component_DateTime = _resolveComponent("DateTime")

  return (_openBlock(), _createElementBlock("div", { class: "contents" }, [ _createElementVNode("div", { class: "comparison-label flex items-center gap-1.5 px-4 py-3 border-b border-border" }, [ _createElementVNode("span", _hoisted_1, _toDisplayString(__props.label), 1 /* TEXT */), (__props.description) ? (_openBlock(), _createBlock(_component_TooltipApp, {
            key: 0,
            text: __props.description,
            position: "top"
          }, {
            default: _withCtx(() => [
              _hoisted_2
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["text"])) : _createCommentVNode("v-if", true) ]), _createTextVNode("\n\n    " + "\n    "), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.values, (value, index) => {
        return (_openBlock(), _createElementBlock("div", {
          key: index,
          class: "comparison-cell relative flex items-center justify-center px-4 py-3 border-b border-border"
        }, [
          (showBar.value && value && getBarWidth(value) > 0)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(["absolute inset-y-1 inset-is-1 rounded-sm transition-all duration-300", getStatusBarClass(value.status)]),
              style: _normalizeStyle({ width: `calc(${getBarWidth(value)}% - 8px)` }),
              "aria-hidden": "true"
            }))
            : _createCommentVNode("v-if", true),
          _createTextVNode("\n\n      " + "\n      "),
          (isCellLoading(index))
            ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
              _hoisted_3
            ], 64 /* STABLE_FRAGMENT */))
            : (!value)
              ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                _hoisted_4
              ], 64 /* STABLE_FRAGMENT */))
            : (_openBlock(), _createElementBlock(_Fragment, { key: 2 }, [
              (value.tooltip)
                ? (_openBlock(), _createBlock(_component_TooltipApp, {
                  key: 0,
                  text: value.tooltip,
                  position: "top"
                }, {
                  default: _withCtx(() => [
                    _createElementVNode("span", {
                      class: _normalizeClass(["relative font-mono text-sm text-center tabular-nums cursor-help", getStatusClass(value.status, showBar.value && getBarWidth(value) > 0)]),
                      "data-status": value.status
                    }, [
                      (value.type === 'date')
                        ? (_openBlock(), _createBlock(_component_DateTime, {
                          key: 0,
                          datetime: value.display,
                          "date-style": "medium"
                        }, null, 8 /* PROPS */, ["datetime"]))
                        : (_openBlock(), _createElementBlock("span", {
                          key: 1,
                          dir: "auto"
                        }, _toDisplayString(value.display), 1 /* TEXT */))
                    ], 10 /* CLASS, PROPS */, ["data-status"])
                  ]),
                  _: 2 /* DYNAMIC */
                }, 8 /* PROPS */, ["text"]))
                : (_openBlock(), _createElementBlock("span", {
                  key: 1,
                  class: _normalizeClass(["relative font-mono text-sm text-center tabular-nums", getStatusClass(value.status, showBar.value && getBarWidth(value) > 0)]),
                  "data-status": value.status
                }, [
                  (value.type === 'date')
                    ? (_openBlock(), _createBlock(_component_DateTime, {
                      key: 0,
                      datetime: value.display,
                      "date-style": "medium"
                    }, null, 8 /* PROPS */, ["datetime"]))
                    : (_openBlock(), _createElementBlock("span", {
                      key: 1,
                      dir: "auto"
                    }, _toDisplayString(value.display), 1 /* TEXT */))
                ]))
            ], 64 /* STABLE_FRAGMENT */)),
          _createTextVNode("\n\n      " + "\n      " + "\n\n      " + "\n      ")
        ]))
      }), 128 /* KEYED_FRAGMENT */)) ]))
}
}

})
