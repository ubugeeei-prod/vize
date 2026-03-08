import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("path", { stroke: "none", d: "M0 0h24v24H0z", fill: "none" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("path", { d: "M12 3a9 9 0 0 1 3.618 17.243l-2.193 -5.602a3 3 0 1 0 -2.849 0l-2.193 5.603a9 9 0 0 1 3.617 -17.244z" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
import MkButton from '@/components/MkButton.vue'
import { host } from '@@/js/config.js'
import { i18n } from '@/i18n.js'
import { instance } from '@/instance.js'
import { miLocalStorage } from '@/local-storage.js'
import * as os from '@/os.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSourceCodeAvailablePopup',
  emits: ["closed"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const zIndex = os.claimZIndex('low');
function close() {
	miLocalStorage.setItem('modifiedVersionMustProminentlyOfferInAgplV3Section13Read', 'true');
	emit('closed');
}

return (_ctx: any,_cache: any) => {
  const _component_I18n = _resolveComponent("I18n")
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["_panel _shadow", _ctx.$style.root])
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.icon)
      }, [ _createElementVNode("svg", {
          xmlns: "http://www.w3.org/2000/svg",
          class: "icon icon-tabler icon-tabler-brand-open-source",
          width: "40",
          height: "40",
          viewBox: "0 0 24 24",
          "stroke-width": "1",
          stroke: "currentColor",
          fill: "none",
          "stroke-linecap": "round",
          "stroke-linejoin": "round"
        }, [ _hoisted_1, _hoisted_2 ]) ]), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.main)
      }, [ _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.title)
        }, [ _createVNode(_component_I18n, {
            src: _unref(i18n).ts.aboutX,
            tag: "span"
          }, {
            x: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(instance).name ?? _unref(host)), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["src"]) ]), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.text)
        }, [ _createVNode(_component_I18n, {
            src: _unref(i18n).ts._aboutMisskey.thisIsModifiedVersion,
            tag: "span"
          }, {
            name: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(instance).name ?? _unref(host)), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["src"]), _createVNode(_component_I18n, {
            src: _unref(i18n).ts.correspondingSourceIsAvailable,
            tag: "span"
          }, {
            anchor: _withCtx(() => [
              _createVNode(_component_MkA, {
                to: "/about-misskey",
                class: "_link"
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.aboutMisskey), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["src"]) ]), _createElementVNode("div", { class: "_buttons" }, [ _createVNode(MkButton, { onClick: close }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.gotIt), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }) ]) ]), _createElementVNode("button", {
        class: _normalizeClass(["_button", _ctx.$style.close]),
        onClick: close
      }, [ _hoisted_3 ]) ]))
}
}

})
