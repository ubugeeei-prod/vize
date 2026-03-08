import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx } from "vue"


const _hoisted_1 = { class: "text-xs text-fg-muted uppercase tracking-wider font-medium" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "i-svg-spinners:ring-resize w-4 h-4 text-fg-subtle", "aria-hidden": "true" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle text-sm" }, "-")
const _hoisted_4 = { dir: "auto" }
import type { FacetValue } from '#shared/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'FacetCard',
  props: {
    label: { type: String, required: true },
    description: { type: String, required: false },
    values: { type: Array, required: true },
    facetLoading: { type: Boolean, required: false },
    columnLoading: { type: Array, required: false },
    bar: { type: Boolean, required: false },
    headers: { type: Array, required: true }
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
function getStatusClass(status?: FacetValue['status']): string {
  switch (status) {
    case 'good':
      return 'text-emerald-400'
    case 'info':
      return 'text-blue-400'
    case 'warning':
      return 'text-amber-400'
    case 'bad':
      return 'text-red-400'
    default:
      return 'text-fg'
  }
}
// Check if a specific cell is loading
function isCellLoading(index: number): boolean {
  return props.facetLoading || (props.columnLoading?.[index] ?? false)
}
// Get short package name (without version) for mobile display
function getShortName(header: string): string {
  const atIndex = header.lastIndexOf('@')
  if (atIndex > 0) {
    return header.substring(0, atIndex)
  }
  return header
}

return (_ctx: any,_cache: any) => {
  const _component_TooltipApp = _resolveComponent("TooltipApp")
  const _component_DateTime = _resolveComponent("DateTime")

  return (_openBlock(), _createElementBlock("div", { class: "border border-border rounded-lg overflow-hidden" }, [ _createElementVNode("div", { class: "flex items-center gap-1.5 px-3 py-2 bg-bg-subtle border-b border-border" }, [ _createElementVNode("span", _hoisted_1, _toDisplayString(__props.label), 1 /* TEXT */), (__props.description) ? (_openBlock(), _createBlock(_component_TooltipApp, {
            key: 0,
            text: __props.description,
            position: "top"
          }, {
            default: _withCtx(() => [
              _createElementVNode("span", {
                class: "i-lucide:info w-3 h-3 text-fg-subtle",
                title: __props.description,
                "aria-hidden": "true"
              }, null, 8 /* PROPS */, ["title"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["text"])) : _createCommentVNode("v-if", true) ]), _createTextVNode("\n\n    " + "\n    "), _createElementVNode("div", { class: "divide-y divide-border" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.values, (value, index) => {
          return (_openBlock(), _createElementBlock("div", {
            key: index,
            class: "relative flex items-center justify-between gap-2 px-3 py-2"
          }, [
            (showBar.value && value && getBarWidth(value) > 0)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: "absolute inset-y-0 inset-is-0 bg-fg/5 transition-all duration-300",
                style: _normalizeStyle({ width: `${getBarWidth(value)}%` }),
                "aria-hidden": "true"
              }))
              : _createCommentVNode("v-if", true),
            _createTextVNode("\n\n        " + "\n        "),
            _createElementVNode("span", {
              class: "relative font-mono text-xs text-fg-muted truncate flex-shrink-0",
              title: __props.headers[index]
            }, _toDisplayString(getShortName(__props.headers[index] ?? '')), 9 /* TEXT, PROPS */, ["title"]),
            _createTextVNode("\n\n        " + "\n        "),
            _createElementVNode("span", { class: "relative min-w-0 text-end" }, [
              (isCellLoading(index))
                ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                  _hoisted_2
                ], 64 /* STABLE_FRAGMENT */))
                : (!value)
                  ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                    _hoisted_3
                  ], 64 /* STABLE_FRAGMENT */))
                : (_openBlock(), _createElementBlock("span", {
                  key: 2,
                  class: _normalizeClass(["font-mono text-sm tabular-nums", getStatusClass(value.status)])
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
                ])),
              _createTextVNode("\n\n          " + "\n          " + "\n\n          " + "\n          ")
            ])
          ]))
        }), 128 /* KEYED_FRAGMENT */)) ]) ]))
}
}

})
