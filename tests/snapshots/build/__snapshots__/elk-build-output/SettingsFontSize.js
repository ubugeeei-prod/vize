import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, renderList as _renderList, toDisplayString as _toDisplayString, unref as _unref } from "vue"


const _hoisted_1 = { id: "interface-fs", "font-medium": "true" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { "text-xs": "true", "text-secondary": "true" }, "Aa")
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { "text-xl": "true", "text-secondary": "true" }, "Aa")
import type { FontSize } from '~/composables/settings'
import { DEFAULT_FONT_SIZE } from '~/constants'

export default /*@__PURE__*/_defineComponent({
  __name: 'SettingsFontSize',
  setup(__props) {

const userSettings = useUserSettings()
const sizes = (Array.from({ length: 11 })).fill(0).map((x, i) => `${10 + i}px`) as FontSize[]
function setFontSize(e: Event) {
  if (e.target && 'valueAsNumber' in e.target)
    userSettings.value.fontSize = sizes[e.target.valueAsNumber as number]
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("section", { "space-y-2": "" }, [ _createElementVNode("h2", _hoisted_1, _toDisplayString(_ctx.$t('settings.interface.font_size')), 1 /* TEXT */), _createElementVNode("div", {
        flex: "",
        "items-center": "",
        "space-x-4": "",
        "select-settings": ""
      }, [ _hoisted_2, _createElementVNode("div", {
          "flex-1": "",
          relative: "",
          flex: "",
          "items-center": ""
        }, [ _createElementVNode("input", {
            "aria-labelledby": "interface-fs",
            value: _unref(sizes).indexOf(_unref(userSettings).fontSize),
            "aria-valuetext": `${_unref(userSettings).fontSize}${_unref(userSettings).fontSize === _unref(DEFAULT_FONT_SIZE) ? ` ${_ctx.$t('settings.interface.default')}` : ''}`,
            min: 0,
            max: _unref(sizes).length - 1,
            step: 1,
            type: "range",
            "focus:outline-none": "",
            "appearance-none": "",
            "bg-transparent": "",
            "w-full": "",
            "cursor-pointer": "",
            onChange: setFontSize
          }, null, 40 /* PROPS, NEED_HYDRATION */, ["value", "aria-valuetext", "min", "max", "step"]), _createElementVNode("div", {
            flex: "",
            "items-center": "",
            "justify-between": "",
            absolute: "",
            "w-full": "",
            "pointer-events-none": ""
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(sizes).length, (i) => {
              return (_openBlock(), _createElementBlock("div", {
                key: i,
                class: "container-marker",
                "h-3": "",
                "w-3": "",
                "rounded-full": "",
                "bg-secondary-light": "",
                relative: ""
              }, [
                ((_unref(sizes).indexOf(_unref(userSettings).fontSize)) === i - 1)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    absolute: "",
                    "rounded-full": "",
                    class: "-top-1 -left-1",
                    "bg-primary": "",
                    "h-5": "",
                    "w-5": ""
                  }))
                  : _createCommentVNode("v-if", true)
              ]))
            }), 128 /* KEYED_FRAGMENT */)) ]) ]), _hoisted_3 ]) ]))
}
}

})
