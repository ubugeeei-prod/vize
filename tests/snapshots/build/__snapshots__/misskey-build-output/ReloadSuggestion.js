import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-info-circle" })
import { i18n } from '@/i18n.js'
import { shouldSuggestReload } from '@/utility/reload-suggest.js'
import { unisonReload } from '@/utility/unison-reload.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'ReloadSuggestion',
  setup(__props) {

function reload() {
	unisonReload();
}
function skip() {
	shouldSuggestReload.value = false;
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ _createElementVNode("span", {
        class: _normalizeClass(_ctx.$style.icon)
      }, [ _hoisted_1 ]), _createElementVNode("span", {
        class: _normalizeClass(_ctx.$style.title)
      }, _toDisplayString(_unref(i18n).ts.reloadRequiredToApplySettings), 1 /* TEXT */), _createElementVNode("span", {
        class: _normalizeClass(_ctx.$style.body)
      }, [ _createElementVNode("button", {
          class: "_textButton",
          style: "color: var(--MI_THEME-fgOnAccent);",
          onClick: reload
        }, _toDisplayString(_unref(i18n).ts.reload), 1 /* TEXT */), _createTextVNode(" | "), _createElementVNode("button", {
          class: "_textButton",
          style: "color: var(--MI_THEME-fgOnAccent);",
          onClick: skip
        }, _toDisplayString(_unref(i18n).ts.skip), 1 /* TEXT */) ]) ]))
}
}

})
