import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-bulb" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots" })
import { i18n } from '@/i18n.js'
import { store } from '@/store.js'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'
import { TIPS, hideAllTips, closeTip } from '@/tips.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkTip',
  props: {
    k: { type: null, required: true },
    warn: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {

const props = __props
function _closeTip() {
	closeTip(props.k);
}
function showMenu(ev: PointerEvent) {
	os.popupMenu([{
		icon: 'ti ti-bulb-off',
		text: i18n.ts.hideAllTips,
		danger: true,
		action: () => {
			hideAllTips();
			os.success();
		},
	}], ev.currentTarget ?? ev.target);
}

return (_ctx: any,_cache: any) => {
  return (!_unref(store).r.tips.value[props.k])
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        class: _normalizeClass(["_selectable _gaps_s", [_ctx.$style.root, { [_ctx.$style.warn]: __props.warn }]])
      }, [ _createElementVNode("div", { style: "font-weight: bold;" }, [ _hoisted_1, _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.tip) + ":", 1 /* TEXT */) ]), _createElementVNode("div", null, [ _renderSlot(_ctx.$slots, "default") ]), _createElementVNode("div", null, [ _createVNode(MkButton, {
            inline: "",
            primary: "",
            rounded: "",
            small: "",
            onClick: _cache[0] || (_cache[0] = ($event: any) => (_closeTip()))
          }, {
            default: _withCtx(() => [
              _hoisted_2,
              _createTextVNode(" "),
              _createTextVNode(_toDisplayString(_unref(i18n).ts.gotIt), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }), _createElementVNode("button", {
            class: "_button",
            style: "padding: 8px; margin-left: 4px;",
            onClick: showMenu
          }, [ _hoisted_3 ]) ]) ]))
      : _createCommentVNode("v-if", true)
}
}

})
