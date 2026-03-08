import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { style: "text-align: center; padding: 0 16px;" }
const _hoisted_2 = { href: "https://misskey-hub.net/docs/for-users/features/timeline/", target: "_blank", class: "_link" }
import { i18n } from '@/i18n.js'
import { basicTimelineIconClass, basicTimelineTypes } from '@/timelines.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkTutorialDialog.Timeline',
  setup(__props) {


return (_ctx: any,_cache: any) => {
  const _component_I18n = _resolveComponent("I18n")

  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ _createElementVNode("div", _hoisted_1, _toDisplayString(_unref(i18n).ts._initialTutorial._timeline.description1), 1 /* TEXT */), _createElementVNode("div", { class: "_gaps_s" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(basicTimelineTypes), (tl) => {
          return (_openBlock(), _createElementBlock("div", null, [
            _createElementVNode("i", {
              class: _normalizeClass(_unref(basicTimelineIconClass)(tl))
            }, null, 2 /* CLASS */),
            _createTextVNode(),
            _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._timelines[tl]), 1 /* TEXT */),
            _createTextVNode(" … " + _toDisplayString(_unref(i18n).ts._initialTutorial._timeline[tl]), 1 /* TEXT */)
          ]))
        }), 256 /* UNKEYED_FRAGMENT */)) ]), _createElementVNode("div", { class: "_gaps_s" }, [ _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._initialTutorial._timeline.description2), 1 /* TEXT */), _createElementVNode("img", {
          class: _normalizeClass(_ctx.$style.image),
          src: "/client-assets/tutorial/timeline_tab.png"
        }) ]), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.divider)
      }), _createVNode(_component_I18n, {
        src: _unref(i18n).ts._initialTutorial._timeline.description3,
        tag: "div",
        style: "padding: 0 16px;"
      }, {
        link: _withCtx(() => [
          _createElementVNode("a", _hoisted_2, _toDisplayString(_unref(i18n).ts.help), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["src"]) ]))
}
}

})
