import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, resolveDirective as _resolveDirective, normalizeClass as _normalizeClass } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-up-left" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-up" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-up-right" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-left" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-focus-2" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-down-left" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-down" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-down-right" })

export default /*@__PURE__*/_defineComponent({
  __name: 'MkPositionSelector',
  props: {
    "x": { default: 'center' },
    "xModifiers": {},
    "y": { default: 'center' },
    "yModifiers": {},
  },
  emits: ["update:x", "update:y"],
  setup(__props) {

const x = _useModel(__props, "x")
const y = _useModel(__props, "y")

return (_ctx: any,_cache: any) => {
  const _directive_panel = _resolveDirective("panel")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.root])
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.items)
      }, [ _createElementVNode("button", {
          class: _normalizeClass(["_button", [_ctx.$style.item, x.value === 'left' && y.value === 'top' ? _ctx.$style.active : null]]),
          onClick: _cache[0] || (_cache[0] = () => { x.value = 'left'; y.value = 'top'; })
        }, [ _hoisted_1 ], 2 /* CLASS */), _createElementVNode("button", {
          class: _normalizeClass(["_button", [_ctx.$style.item, x.value === 'center' && y.value === 'top' ? _ctx.$style.active : null]]),
          onClick: _cache[1] || (_cache[1] = () => { x.value = 'center'; y.value = 'top'; })
        }, [ _hoisted_2 ], 2 /* CLASS */), _createElementVNode("button", {
          class: _normalizeClass(["_button", [_ctx.$style.item, x.value === 'right' && y.value === 'top' ? _ctx.$style.active : null]]),
          onClick: _cache[2] || (_cache[2] = () => { x.value = 'right'; y.value = 'top'; })
        }, [ _hoisted_3 ], 2 /* CLASS */), _createElementVNode("button", {
          class: _normalizeClass(["_button", [_ctx.$style.item, x.value === 'left' && y.value === 'center' ? _ctx.$style.active : null]]),
          onClick: _cache[3] || (_cache[3] = () => { x.value = 'left'; y.value = 'center'; })
        }, [ _hoisted_4 ], 2 /* CLASS */), _createElementVNode("button", {
          class: _normalizeClass(["_button", [_ctx.$style.item, x.value === 'center' && y.value === 'center' ? _ctx.$style.active : null]]),
          onClick: _cache[4] || (_cache[4] = () => { x.value = 'center'; y.value = 'center'; })
        }, [ _hoisted_5 ], 2 /* CLASS */), _createElementVNode("button", {
          class: _normalizeClass(["_button", [_ctx.$style.item, x.value === 'right' && y.value === 'center' ? _ctx.$style.active : null]]),
          onClick: _cache[5] || (_cache[5] = () => { x.value = 'right'; y.value = 'center'; })
        }, [ _hoisted_6 ], 2 /* CLASS */), _createElementVNode("button", {
          class: _normalizeClass(["_button", [_ctx.$style.item, x.value === 'left' && y.value === 'bottom' ? _ctx.$style.active : null]]),
          onClick: _cache[6] || (_cache[6] = () => { x.value = 'left'; y.value = 'bottom'; })
        }, [ _hoisted_7 ], 2 /* CLASS */), _createElementVNode("button", {
          class: _normalizeClass(["_button", [_ctx.$style.item, x.value === 'center' && y.value === 'bottom' ? _ctx.$style.active : null]]),
          onClick: _cache[7] || (_cache[7] = () => { x.value = 'center'; y.value = 'bottom'; })
        }, [ _hoisted_8 ], 2 /* CLASS */), _createElementVNode("button", {
          class: _normalizeClass(["_button", [_ctx.$style.item, x.value === 'right' && y.value === 'bottom' ? _ctx.$style.active : null]]),
          onClick: _cache[8] || (_cache[8] = () => { x.value = 'right'; y.value = 'bottom'; })
        }, [ _hoisted_9 ], 2 /* CLASS */) ]) ]))
}
}

})
