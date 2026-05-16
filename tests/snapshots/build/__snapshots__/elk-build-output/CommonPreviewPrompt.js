import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { "font-bold": "true", "text-rose": "true" }
const _hoisted_2 = { "font-bold": "true" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("div", { "i-ri-git-pull-request-line": "true", absolute: "true", "text-10em": "true", "bottom--10": "true", "inset-ie--10": "true", "text-rose": "true", op10: "true", class: "-z-1" })

export default /*@__PURE__*/_defineComponent({
  __name: 'CommonPreviewPrompt',
  setup(__props) {

const build = useBuildInfo()

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_i18n_t = _resolveComponent("i18n-t")

  return (_openBlock(), _createElementBlock("div", {
      "m-2": "",
      p5: "",
      "bg-rose:10": "",
      relative: "",
      "rounded-lg": "",
      "of-hidden": "",
      flex: "~ col gap-3"
    }, [ _createElementVNode("h2", _hoisted_1, _toDisplayString(_ctx.$t('help.build_preview.title')), 1 /* TEXT */), _createElementVNode("p", null, [ _createVNode(_component_i18n_t, { keypath: "help.build_preview.desc1" }, {
          default: _withCtx(() => [
            _createVNode(_component_NuxtLink, {
              href: `https://github.com/elk-zone/elk/commit/${_unref(build).commit}`,
              target: "_blank",
              "text-rose": "",
              "hover:underline": ""
            }, {
              default: _withCtx(() => [
                _createElementVNode("code", null, _toDisplayString(_unref(build).shortCommit), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["href"])
          ]),
          _: 1 /* STABLE */
        }) ]), _createElementVNode("p", null, _toDisplayString(_ctx.$t('help.build_preview.desc2')), 1 /* TEXT */), _createElementVNode("p", _hoisted_2, _toDisplayString(_ctx.$t('help.build_preview.desc3')), 1 /* TEXT */), _hoisted_3 ]))
}
}

})
