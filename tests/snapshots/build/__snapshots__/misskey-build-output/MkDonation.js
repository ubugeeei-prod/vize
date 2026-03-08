import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("path", { stroke: "none", d: "M0 0h24v24H0z", fill: "none" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("path", { d: "M15 11v.01" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("path", { d: "M5.173 8.378a3 3 0 1 1 4.656 -1.377" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("path", { d: "M16 4v3.803a6.019 6.019 0 0 1 2.658 3.197h1.341a1 1 0 0 1 1 1v2a1 1 0 0 1 -1 1h-1.342c-.336 .95 -.907 1.8 -1.658 2.473v2.027a1.5 1.5 0 0 1 -3 0v-.583a6.04 6.04 0 0 1 -1 .083h-4a6.04 6.04 0 0 1 -1 -.083v.583a1.5 1.5 0 0 1 -3 0v-2l.001 -.027a6 6 0 0 1 3.999 -10.473h2.5l4.5 -3h.001z" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
import MkButton from '@/components/MkButton.vue'
import MkLink from '@/components/MkLink.vue'
import { host } from '@@/js/config.js'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'
import { miLocalStorage } from '@/local-storage.js'
import { instance } from '@/instance.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkDonation',
  emits: ["closed"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const zIndex = os.claimZIndex('low');
function close() {
	miLocalStorage.setItem('latestDonationInfoShownAt', Date.now().toString());
	emit('closed');
}
function neverShow() {
	miLocalStorage.setItem('neverShowDonationInfo', 'true');
	close();
}

return (_ctx: any,_cache: any) => {
  const _component_I18n = _resolveComponent("I18n")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["_panel _shadow", _ctx.$style.root])
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.icon)
      }, [ _createElementVNode("svg", {
          xmlns: "http://www.w3.org/2000/svg",
          class: "icon icon-tabler icon-tabler-pig-money",
          width: "40",
          height: "40",
          viewBox: "0 0 24 24",
          "stroke-width": "1",
          stroke: "currentColor",
          fill: "none",
          "stroke-linecap": "round",
          "stroke-linejoin": "round"
        }, [ _hoisted_1, _hoisted_2, _hoisted_3, _hoisted_4 ]) ]), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.main)
      }, [ _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.title)
        }, _toDisplayString(_unref(i18n).ts.didYouLikeMisskey), 1 /* TEXT */), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.text)
        }, [ _createVNode(_component_I18n, {
            src: _unref(i18n).ts.pleaseDonate,
            tag: "span"
          }, {
            host: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(instance).name ?? _unref(host)), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["src"]), _createElementVNode("div", { style: "margin-top: 0.2em;" }, [ _createVNode(MkLink, {
              target: "_blank",
              url: "https://misskey-hub.net/docs/for-users/resources/donate/"
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.learnMore), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }) ]) ]), _createElementVNode("div", { class: "_buttons" }, [ _createVNode(MkButton, { onClick: close }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.remindMeLater), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }), _createVNode(MkButton, { onClick: neverShow }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.neverShow), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }) ]) ]), _createElementVNode("button", {
        class: _normalizeClass(["_button", _ctx.$style.close]),
        onClick: close
      }, [ _hoisted_5 ]) ]))
}
}

})
