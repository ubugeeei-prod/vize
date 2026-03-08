import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { "i-ri:close-line": "true" })
const _hoisted_2 = { mxa: "true", "text-4xl": "true", mb4: "true" }
const _hoisted_3 = { "text-primary": "true" }
const _hoisted_4 = { "text-xl": "true", "font-script": "true", "hover:text-primary": "true", transition: "true", "duration-300": "true" }

export default /*@__PURE__*/_defineComponent({
  __name: 'HelpPreview',
  emits: ["close"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const vAutoFocus = (el: HTMLElement) => el.focus()

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _directive_auto_focus = _resolveDirective("auto-focus")

  return (_openBlock(), _createElementBlock("div", {
      "my-8": "",
      "px-3": "",
      "sm:px-8": "",
      "md:max-w-200": "",
      flex: "~ col gap-4",
      relative: ""
    }, [ _createElementVNode("button", {
        type: "button",
        "btn-action-icon": "",
        absolute: "",
        "top--8": "",
        "right-0": "",
        m1: "",
        "aria-label": _ctx.$t('action.close'),
        onClick: _cache[0] || (_cache[0] = ($event: any) => (emit('close')))
      }, [ _hoisted_1 ], 8 /* PROPS */, ["aria-label"]), _createElementVNode("img", {
        alt: _ctx.$t('app_logo'),
        src: `/${''}logo.svg`,
        "w-20": "",
        "h-20": "",
        height: "80",
        width: "80",
        mxa: "",
        class: "rtl-flip"
      }, null, 8 /* PROPS */, ["alt", "src"]), _createElementVNode("h1", _hoisted_2, _toDisplayString(_ctx.$t('help.title')), 1 /* TEXT */), _createElementVNode("p", null, _toDisplayString(_ctx.$t('help.desc_para1')), 1 /* TEXT */), _createElementVNode("p", null, [ _createElementVNode("b", _hoisted_3, _toDisplayString(_ctx.$t('help.desc_highlight')), 1 /* TEXT */), _createTextVNode("\n      " + _toDisplayString(_ctx.$t('help.desc_para2')), 1 /* TEXT */) ]), _createElementVNode("p", null, [ _createTextVNode(_toDisplayString(_ctx.$t('help.desc_para4')) + "\n      ", 1 /* TEXT */), _createVNode(_component_NuxtLink, {
          "font-bold": "",
          "text-primary": "",
          href: "https://github.com/elk-zone/elk",
          target: "_blank"
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_ctx.$t('help.desc_para5')), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }), _createTextVNode("\n      " + _toDisplayString(_ctx.$t('help.desc_para6')), 1 /* TEXT */) ]), _createVNode(_component_NuxtLink, {
        "hover:text-primary": "",
        href: "https://github.com/sponsors/elk-zone",
        target: "_blank"
      }, {
        default: _withCtx(() => [
          _createTextVNode(_toDisplayString(_ctx.$t('help.desc_para3')), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }), _createElementVNode("p", {
        flex: "~ gap-2 wrap justify-center",
        mxa: ""
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.elkTeamMembers, (team) => {
          return (_openBlock(), _createElementBlock(_Fragment, { key: team.github }, [
            _createVNode(_component_NuxtLink, {
              href: team.link,
              target: "_blank",
              external: "",
              "rounded-full": "",
              transition: "",
              "duration-300": "",
              border: "~ transparent",
              hover: "scale-105 border-primary"
            }, {
              default: _withCtx(() => [
                _createElementVNode("img", {
                  src: `/avatars/${team.github}-100x100.png`,
                  alt: team.display,
                  "rounded-full": "",
                  "w-15": "",
                  "h-15": "",
                  height: "60",
                  width: "60"
                }, null, 8 /* PROPS */, ["src", "alt"])
              ]),
              _: 2 /* DYNAMIC */
            }, 8 /* PROPS */, ["href"])
          ], 64 /* STABLE_FRAGMENT */))
        }), 128 /* KEYED_FRAGMENT */)) ]), _createElementVNode("p", {
        italic: "",
        flex: "",
        "justify-center": "",
        "w-full": ""
      }, [ _createVNode(_component_NuxtLink, {
          href: "https://github.com/sponsors/elk-zone",
          target: "_blank"
        }, {
          default: _withCtx(() => [
            _createElementVNode("span", _hoisted_4, _toDisplayString(_ctx.$t('help.footer_team')), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }) ]), _createElementVNode("button", {
        type: "button",
        "btn-solid": "",
        mxa: "",
        onClick: _cache[1] || (_cache[1] = ($event: any) => (emit('close')))
      }, _toDisplayString(_ctx.$t('action.enter_app')), 1 /* TEXT */) ]))
}
}

})
