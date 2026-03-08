import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"


const _hoisted_1 = { id: "interface-cm", "font-medium": "true" }
import type { ColorMode } from '~/composables/settings'

export default /*@__PURE__*/_defineComponent({
  __name: 'SettingsColorMode',
  setup(__props) {

const colorMode = useColorMode()
function setColorMode(mode: ColorMode) {
  colorMode.preference = mode
}
const modes = [
  {
    icon: 'i-ri-moon-line',
    label: 'settings.interface.dark_mode',
    mode: 'dark',
  },
  {
    icon: 'i-ri-sun-line',
    label: 'settings.interface.light_mode',
    mode: 'light',
  },
  {
    icon: 'i-ri-computer-line',
    label: 'settings.interface.system_mode',
    mode: 'system',
  },
] as const

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("section", { "space-y-2": "" }, [ _createElementVNode("h2", _hoisted_1, _toDisplayString(_ctx.$t('settings.interface.color_mode')), 1 /* TEXT */), _createElementVNode("div", {
        flex: "~ gap4 wrap",
        "w-full": "",
        role: "group",
        "aria-labelledby": "interface-cm"
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(modes), ({ icon, label, mode }) => {
          return (_openBlock(), _createElementBlock("button", {
            key: mode,
            type: "button",
            "btn-text": "",
            "flex-1": "",
            flex: "~ gap-1 center",
            p4: "",
            border: "~ base rounded",
            "bg-base": "",
            "ws-nowrap": "",
            "aria-pressed": _unref(colorMode).preference === mode ? 'true' : 'false',
            class: _normalizeClass(_unref(colorMode).preference === mode ? 'pointer-events-none' : 'filter-saturate-0'),
            onClick: _cache[0] || (_cache[0] = ($event: any) => (setColorMode(_ctx.mode)))
          }, [
            _createElementVNode("span", {
              class: _normalizeClass(`${icon}`)
            }),
            _createTextVNode("\n        " + _toDisplayString(_ctx.$t(label)), 1 /* TEXT */)
          ], 10 /* CLASS, PROPS */, ["aria-pressed"]))
        }), 128 /* KEYED_FRAGMENT */)) ]) ]))
}
}

})
