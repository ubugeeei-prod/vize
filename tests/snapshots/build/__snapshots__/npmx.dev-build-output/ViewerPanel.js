import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, vModelText as _vModelText } from "vue"


const _hoisted_1 = { class: "font-mono text-sm truncate max-w-[65vw] sm:max-w-none" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:settings w-3.5 h-3.5" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("label", { for: "change-ratio" }, "Change ratio")
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "slider-label" }, "Change ratio")
const _hoisted_5 = { class: "slider-value tabular-nums" }
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("label", { for: "diff-distance" }, "Diff distance")
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("span", { class: "slider-label" }, "Diff distance")
const _hoisted_8 = { class: "slider-value tabular-nums" }
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("label", { for: "char-edits" }, "Char edits")
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("span", { class: "slider-label" }, "Char edits")
const _hoisted_11 = { class: "slider-value tabular-nums" }
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("div", { class: "i-lucide:file-text w-12 h-12 mx-auto text-fg-subtle mb-4" })
const _hoisted_13 = { class: "text-fg-muted mb-2" }
const _hoisted_14 = { class: "text-fg-subtle text-sm mb-4" }
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("div", { class: "i-svg-spinners-ring-resize w-6 h-6 mx-auto text-fg-muted" })
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("p", { class: "mt-2 text-sm text-fg-muted" }, "Loading diff...")
const _hoisted_17 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:triangle-alert w-8 h-8 mx-auto text-fg-subtle mb-2 block" })
const _hoisted_18 = { class: "text-fg-muted text-sm mb-2" }
import type { FileDiffResponse, FileChange } from '#shared/types'
import { onClickOutside } from '@vueuse/core'

export default /*@__PURE__*/_defineComponent({
  __name: 'ViewerPanel',
  props: {
    packageName: { type: String, required: true },
    fromVersion: { type: String, required: true },
    toVersion: { type: String, required: true },
    file: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const bytesFormatter = useBytesFormatter()
const mergeModifiedLines = ref(true)
const maxChangeRatio = ref(0.45)
const maxDiffDistance = ref(30)
const inlineMaxCharEdits = ref(2)
const wordWrap = ref(false)
const showOptions = ref(false)
const optionsDropdownRef = useTemplateRef('optionsDropdownRef')
onClickOutside(optionsDropdownRef, () => {
  showOptions.value = false
})
// Maximum file size we'll try to load (250KB) - must match server
const MAX_FILE_SIZE = 250 * 1024
const isFilesTooLarge = computed(() => {
  const newSize = props.file?.newSize
  const oldSize = props.file?.oldSize
  return (
    (newSize !== undefined && newSize > MAX_FILE_SIZE) ||
    (oldSize !== undefined && oldSize > MAX_FILE_SIZE)
  )
})
const apiUrl = computed(() =>
  isFilesTooLarge.value
    ? null
    : `/api/registry/compare-file/${props.packageName}/v/${props.fromVersion}...${props.toVersion}/${props.file.path}`,
)
const apiQuery = computed(() => ({
  mergeModifiedLines: String(mergeModifiedLines.value),
  maxChangeRatio: String(maxChangeRatio.value),
  maxDiffDistance: String(maxDiffDistance.value),
  inlineMaxCharEdits: String(inlineMaxCharEdits.value),
}))
const {
  data: diff,
  status,
  error: loadError,
} = useFetch<FileDiffResponse>(() => apiUrl.value!, {
  query: apiQuery,
  timeout: 15000,
})
function calcPercent(value: number, min: number, max: number): number {
  if (max === min) return 0
  const percent = ((value - min) / (max - min)) * 100
  return Math.min(100, Math.max(0, percent))
}
function getStepMarks(min: number, max: number, step: number): number[] {
  const marks: number[] = []
  const range = max - min
  const stepCount = Math.floor(range / step)
  if (stepCount <= 10) {
    for (let i = 1; i <= stepCount; i++) {
      const positionPercent = ((i * step) / range) * 100
      marks.push(positionPercent)
    }
  }
  return marks
}
const changeRatioMarks = computed(() => getStepMarks(0, 1, 0.1))
const diffDistanceMarks = computed(() => getStepMarks(1, 60, 10))
const charEditMarks = computed(() => [] as number[]) // no dots for char edits slider
const changeRatioPercent = computed(() => calcPercent(maxChangeRatio.value, 0, 1))
const diffDistancePercent = computed(() => calcPercent(maxDiffDistance.value, 1, 60))
const charEditPercent = computed(() => calcPercent(inlineMaxCharEdits.value, 0, 10))
// Build code browser URL
function getCodeUrl(version: string): string {
  return `/package-code/${props.packageName}/v/${version}/${props.file.path}`
}

return (_ctx: any,_cache: any) => {
  const _component_SettingsToggle = _resolveComponent("SettingsToggle")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_DiffTable = _resolveComponent("DiffTable")

  return (_openBlock(), _createElementBlock("div", { class: "h-full flex flex-col bg-bg" }, [ _createElementVNode("div", { class: "flex flex-wrap items-center justify-between gap-3 px-4 py-3 bg-bg-subtle border-b border-border shrink-0" }, [ _createElementVNode("div", { class: "flex items-center gap-3 min-w-0 flex-1" }, [ _createElementVNode("span", {
            class: _normalizeClass([
              'w-4 h-4 shrink-0',
              __props.file.type === 'added'
                ? 'i-lucide:plus text-green-500'
                : __props.file.type === 'removed'
                  ? 'i-lucide:minus text-red-500'
                  : 'i-lucide:pen text-yellow-500',
            ])
          }, null, 2 /* CLASS */), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("span", _hoisted_1, _toDisplayString(__props.file.path), 1 /* TEXT */), _createTextVNode("\n\n        " + "\n        "), (_unref(diff)?.stats) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (_unref(diff).stats.additions > 0) ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  class: "text-xs text-green-500 font-mono shrink-0"
                }, "\n            +" + _toDisplayString(_unref(diff).stats.additions), 1 /* TEXT */)) : _createCommentVNode("v-if", true), (_unref(diff).stats.deletions > 0) ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  class: "text-xs text-red-500 font-mono shrink-0"
                }, "\n            -" + _toDisplayString(_unref(diff).stats.deletions), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true), _createTextVNode("\n\n        " + "\n        "), (__props.file.oldSize || __props.file.newSize) ? (_openBlock(), _createElementBlock("span", {
              key: 0,
              class: "text-xs text-fg-subtle shrink-0"
            }, [ (__props.file.type === 'modified') ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _toDisplayString(_unref(bytesFormatter).format(__props.file.oldSize ?? 0)), _createTextVNode(" →\n            "), _toDisplayString(_unref(bytesFormatter).format(__props.file.newSize ?? 0)) ], 64 /* STABLE_FRAGMENT */)) : (__props.file.type === 'added') ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ _toDisplayString(_unref(bytesFormatter).format(__props.file.newSize ?? 0)) ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock(_Fragment, { key: 2 }, [ _toDisplayString(_unref(bytesFormatter).format(__props.file.oldSize ?? 0)) ], 64 /* STABLE_FRAGMENT */)) ])) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", { class: "flex items-center gap-2 shrink-0" }, [ _createElementVNode("div", {
            ref_key: "optionsDropdownRef", ref: optionsDropdownRef,
            class: "relative"
          }, [ _createElementVNode("button", {
              type: "button",
              class: _normalizeClass(["px-2 py-1 text-xs text-fg-muted hover:text-fg bg-bg-muted border border-border rounded transition-colors flex items-center gap-1.5", { 'bg-bg-elevated text-fg': showOptions.value }]),
              onClick: _cache[0] || (_cache[0] = ($event: any) => (showOptions.value = !showOptions.value))
            }, [ _hoisted_2, _createTextVNode("\n            Options\n            "), _createElementVNode("span", {
                class: _normalizeClass(["i-lucide:chevron-down w-3 h-3 transition-transform", { 'rotate-180': showOptions.value }])
              }, null, 2 /* CLASS */) ], 2 /* CLASS */), _createTextVNode("\n\n          " + "\n          "), _createVNode(_Transition, {
              "enter-active-class": "transition duration-150 ease-out motion-reduce:transition-none",
              "enter-from-class": "opacity-0 scale-95 motion-reduce:scale-100",
              "enter-to-class": "opacity-100 scale-100",
              "leave-active-class": "transition duration-100 ease-in motion-reduce:transition-none",
              "leave-from-class": "opacity-100 scale-100",
              "leave-to-class": "opacity-0 scale-95 motion-reduce:scale-100"
            }, {
              default: _withCtx(() => [
                (showOptions.value)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: "absolute inset-ie-0 top-full mt-2 z-20 p-4 bg-bg-elevated border border-border rounded-xl shadow-2xl overflow-auto",
                    style: _normalizeStyle({
                  width: mergeModifiedLines.value
                    ? 'min(420px, calc(100vw - 24px))'
                    : 'min(320px, calc(100vw - 24px))',
                  maxWidth: 'calc(100vw - 24px)',
                  maxHeight: '70vh',
                })
                  }, [
                    _createElementVNode("div", { class: "flex flex-col gap-2" }, [
                      _createVNode(_component_SettingsToggle, {
                        label: "Merge modified lines",
                        modelValue: mergeModifiedLines.value,
                        "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((mergeModifiedLines).value = $event))
                      }, null, 8 /* PROPS */, ["modelValue"]),
                      _createTextVNode("\n\n                " + "\n                "),
                      _createVNode(_component_SettingsToggle, {
                        label: "Word wrap",
                        modelValue: wordWrap.value,
                        "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((wordWrap).value = $event))
                      }, null, 8 /* PROPS */, ["modelValue"]),
                      _createTextVNode("\n\n                " + "\n                "),
                      _createElementVNode("div", {
                        class: _normalizeClass(["flex flex-col gap-2 transition-opacity duration-150", mergeModifiedLines.value ? 'opacity-100' : 'opacity-0'])
                      }, [
                        _createElementVNode("div", { class: "sr-only" }, [
                          _hoisted_3
                        ]),
                        _createElementVNode("div", {
                          class: _normalizeClass(["slider-shell w-full min-w-0", { 'is-disabled': !mergeModifiedLines.value }])
                        }, [
                          _createElementVNode("div", { class: "slider-labels" }, [
                            _hoisted_4,
                            _createElementVNode("span", _hoisted_5, _toDisplayString(maxChangeRatio.value.toFixed(2)), 1 /* TEXT */)
                          ]),
                          _createElementVNode("div", { class: "slider-track" }, [
                            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(changeRatioMarks.value, (mark) => {
                              return (_openBlock(), _createElementBlock("div", {
                                key: `cr-${mark}`,
                                class: "slider-mark",
                                style: _normalizeStyle({ left: `calc(${mark}% - 11px)` })
                              }, 4 /* STYLE */))
                            }), 128 /* KEYED_FRAGMENT */)),
                            _createElementVNode("div", {
                              class: "slider-range",
                              style: _normalizeStyle({ width: `${changeRatioPercent.value}%` })
                            }, null, 4 /* STYLE */)
                          ]),
                          _withDirectives(_createElementVNode("input", {
                            id: "change-ratio",
                            "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((maxChangeRatio).value = $event)),
                            type: "range",
                            min: "0",
                            max: "1",
                            step: "0.01",
                            disabled: !mergeModifiedLines.value,
                            class: "slider-input"
                          }, null, 8 /* PROPS */, ["disabled"]), [
                            [
                              _vModelText,
                              maxChangeRatio.value,
                              void 0,
                              { number: true }
                            ]
                          ])
                        ], 2 /* CLASS */),
                        _createTextVNode("\n\n                  " + "\n                  "),
                        _createElementVNode("div", { class: "sr-only" }, [
                          _hoisted_6
                        ]),
                        _createElementVNode("div", {
                          class: _normalizeClass(["slider-shell w-full min-w-0", { 'is-disabled': !mergeModifiedLines.value }])
                        }, [
                          _createElementVNode("div", { class: "slider-labels" }, [
                            _hoisted_7,
                            _createElementVNode("span", _hoisted_8, _toDisplayString(maxDiffDistance.value), 1 /* TEXT */)
                          ]),
                          _createElementVNode("div", { class: "slider-track" }, [
                            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(diffDistanceMarks.value, (mark) => {
                              return (_openBlock(), _createElementBlock("div", {
                                key: `dd-${mark}`,
                                class: "slider-mark",
                                style: _normalizeStyle({ left: `calc(${mark}% - 11px)` })
                              }, 4 /* STYLE */))
                            }), 128 /* KEYED_FRAGMENT */)),
                            _createElementVNode("div", {
                              class: "slider-range",
                              style: _normalizeStyle({ width: `${diffDistancePercent.value}%` })
                            }, null, 4 /* STYLE */)
                          ]),
                          _withDirectives(_createElementVNode("input", {
                            id: "diff-distance",
                            "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((maxDiffDistance).value = $event)),
                            type: "range",
                            min: "1",
                            max: "60",
                            step: "1",
                            disabled: !mergeModifiedLines.value,
                            class: "slider-input"
                          }, null, 8 /* PROPS */, ["disabled"]), [
                            [
                              _vModelText,
                              maxDiffDistance.value,
                              void 0,
                              { number: true }
                            ]
                          ])
                        ], 2 /* CLASS */),
                        _createTextVNode("\n\n                  " + "\n                  "),
                        _createElementVNode("div", { class: "sr-only" }, [
                          _hoisted_9
                        ]),
                        _createElementVNode("div", {
                          class: _normalizeClass(["slider-shell w-full min-w-0", { 'is-disabled': !mergeModifiedLines.value }])
                        }, [
                          _createElementVNode("div", { class: "slider-labels" }, [
                            _hoisted_10,
                            _createElementVNode("span", _hoisted_11, _toDisplayString(inlineMaxCharEdits.value), 1 /* TEXT */)
                          ]),
                          _createElementVNode("div", { class: "slider-track" }, [
                            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(charEditMarks.value, (mark) => {
                              return (_openBlock(), _createElementBlock("div", {
                                key: `ce-${mark}`,
                                class: "slider-mark",
                                style: _normalizeStyle({ left: `calc(${mark}% - 11px)` })
                              }, 4 /* STYLE */))
                            }), 128 /* KEYED_FRAGMENT */)),
                            _createElementVNode("div", {
                              class: "slider-range",
                              style: _normalizeStyle({ width: `${charEditPercent.value}%` })
                            }, null, 4 /* STYLE */)
                          ]),
                          _withDirectives(_createElementVNode("input", {
                            id: "char-edits",
                            "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((inlineMaxCharEdits).value = $event)),
                            type: "range",
                            min: "0",
                            max: "10",
                            step: "1",
                            disabled: !mergeModifiedLines.value,
                            class: "slider-input"
                          }, null, 8 /* PROPS */, ["disabled"]), [
                            [
                              _vModelText,
                              inlineMaxCharEdits.value,
                              void 0,
                              { number: true }
                            ]
                          ])
                        ], 2 /* CLASS */)
                      ], 2 /* CLASS */)
                    ])
                  ]))
                  : _createCommentVNode("v-if", true)
              ]),
              _: 1 /* STABLE */
            }) ], 512 /* NEED_PATCH */), _createTextVNode("\n\n        " + "\n        "), (__props.file.type !== 'removed') ? (_openBlock(), _createBlock(_component_NuxtLink, {
              key: 0,
              to: getCodeUrl(__props.toVersion),
              class: "px-2 py-1 text-xs text-fg-muted hover:text-fg bg-bg-muted border border-border rounded transition-colors",
              target: "_blank"
            }, {
              default: _withCtx(() => [
                _createTextVNode("\n          View file\n        ")
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true) ]) ]), _createTextVNode("\n\n    " + "\n    "), _createElementVNode("div", { class: "flex-1 overflow-auto relative" }, [ (isFilesTooLarge.value) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "py-20 text-center"
          }, [ _hoisted_12, _createElementVNode("p", _hoisted_13, _toDisplayString(_ctx.$t('compare.file_too_large')), 1 /* TEXT */), _createElementVNode("p", _hoisted_14, _toDisplayString(_ctx.$t('compare.file_size_warning', {
                size: _unref(bytesFormatter).format(Math.max(__props.file.newSize ?? 0, __props.file.oldSize ?? 0)),
              })), 1 /* TEXT */) ])) : (_unref(status) === 'pending') ? (_openBlock(), _createElementBlock("div", {
              key: 1,
              class: "py-12 text-center"
            }, [ _hoisted_15, _hoisted_16 ])) : (_unref(status) === 'error') ? (_openBlock(), _createElementBlock("div", {
              key: 2,
              class: "py-8 text-center"
            }, [ _hoisted_17, _createElementVNode("p", _hoisted_18, _toDisplayString(_unref(loadError)?.message || 'Failed to load diff'), 1 /* TEXT */), _createElementVNode("div", { class: "flex items-center justify-center gap-2" }, [ (__props.file.type !== 'removed') ? (_openBlock(), _createBlock(_component_NuxtLink, {
                    key: 0,
                    to: getCodeUrl(__props.toVersion),
                    class: "text-xs text-fg-muted hover:text-fg underline"
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode("\n            View in code browser\n          ")
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true) ]) ])) : (_unref(diff) && _unref(diff).hunks.length === 0) ? (_openBlock(), _createElementBlock("div", {
              key: 3,
              class: "py-8 text-center text-fg-muted text-sm"
            }, "\n        No content changes detected\n      ")) : (_unref(diff)) ? (_openBlock(), _createBlock(_component_DiffTable, {
              key: 4,
              hunks: _unref(diff).hunks,
              type: _unref(diff).type,
              "file-name": __props.file.path,
              "word-wrap": wordWrap.value
            }, null, 8 /* PROPS */, ["hunks", "type", "file-name", "word-wrap"])) : _createCommentVNode("v-if", true), _createTextVNode("\n      " + "\n      " + "\n\n      " + "\n      " + "\n\n      " + "\n      " + "\n\n      " + "\n      ") ]) ]))
}
}

})
