import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'NavFooter',
  setup(__props) {

const buildInfo = useBuildInfo()
const timeAgoOptions = useTimeAgoOptions()
const config = useRuntimeConfig()
const userSettings = useUserSettings()
const buildTimeDate = new Date(buildInfo.time)
const buildTimeAgo = useTimeAgo(buildTimeDate, timeAgoOptions)
const colorMode = useColorMode()
function toggleDark() {
  colorMode.preference = colorMode.value === 'dark' ? 'light' : 'dark'
}

return (_ctx: any,_cache: any) => {
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_i18n_t = _resolveComponent("i18n-t")

  return (_openBlock(), _createElementBlock("footer", {
      p4: "",
      "text-sm": "",
      "text-secondary-light": "",
      flex: "~ col"
    }, [ _createElementVNode("div", {
        flex: "~ gap2",
        "items-center": "",
        mb4: ""
      }, [ _createVNode(_component_CommonTooltip, { content: _ctx.$t('nav.toggle_theme') }, {
          default: _withCtx(() => [
            _createElementVNode("button", {
              flex: "",
              "i-ri:sun-line": "",
              "dark-i-ri:moon-line": "",
              "text-lg": "",
              "aria-label": _ctx.$t('nav.toggle_theme'),
              onClick: _cache[0] || (_cache[0] = ($event: any) => (toggleDark()))
            }, null, 8 /* PROPS */, ["aria-label"])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["content"]), _createVNode(_component_CommonTooltip, { content: _ctx.$t('nav.zen_mode') }, {
          default: _withCtx(() => [
            _createElementVNode("button", {
              flex: "",
              "text-lg": "",
              class: _normalizeClass(_ctx.getPreferences(_unref(userSettings), 'zenMode') ? 'i-ri:layout-right-2-line' : 'i-ri:layout-right-line'),
              "aria-label": _ctx.$t('nav.zen_mode'),
              onClick: _cache[1] || (_cache[1] = ($event: any) => (_ctx.togglePreferences('zenMode')))
            }, null, 10 /* CLASS, PROPS */, ["aria-label"])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["content"]), _createVNode(_component_CommonTooltip, { content: _ctx.$t('magic_keys.dialog_header') }, {
          default: _withCtx(() => [
            _createElementVNode("button", {
              flex: "",
              "i-ri:keyboard-box-line": "",
              "dark-i-ri:keyboard-box-line": "",
              "text-lg": "",
              "aria-label": _ctx.$t('magic_keys.dialog_header'),
              onClick: _cache[2] || (_cache[2] = (...args) => (_ctx.toggleKeyboardShortcuts && _ctx.toggleKeyboardShortcuts(...args)))
            }, null, 8 /* PROPS */, ["aria-label"])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["content"]), _createVNode(_component_CommonTooltip, { content: _ctx.$t('settings.about.sponsor_action') }, {
          default: _withCtx(() => [
            _createVNode(_component_NuxtLink, {
              flex: "",
              "text-lg": "",
              "i-ri-heart-3-line": "",
              hover: "i-ri-heart-3-fill text-rose",
              "aria-label": _ctx.$t('settings.about.sponsor_action'),
              href: "https://github.com/sponsors/elk-zone",
              target: "_blank"
            }, null, 8 /* PROPS */, ["aria-label"])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["content"]) ]), _createElementVNode("div", null, [ (_ctx.isHydrated) ? (_openBlock(), _createBlock(_component_i18n_t, {
            key: 0,
            keypath: "nav.built_at"
          }, {
            default: _withCtx(() => [
              _createElementVNode("time", {
                datetime: String(_unref(buildTimeDate)),
                title: _ctx.$d(_unref(buildTimeDate), 'long')
              }, _toDisplayString(_unref(buildTimeAgo)), 9 /* TEXT, PROPS */, ["datetime", "title"])
            ]),
            _: 1 /* STABLE */
          })) : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(_ctx.$t('nav.built_at', [_ctx.$d(_unref(buildTimeDate), 'shortDate')])), 1 /* TEXT */)), _createTextVNode("\n      &middot;\n      "), (_unref(buildInfo).env === 'release') ? (_openBlock(), _createBlock(_component_NuxtLink, {
            key: 0,
            external: "",
            href: `https://github.com/elk-zone/elk/releases/tag/v${_unref(buildInfo).version}`,
            target: "_blank",
            "font-mono": ""
          }, {
            default: _withCtx(() => [
              _createTextVNode("\n        v"),
              _createTextVNode(_toDisplayString(_unref(buildInfo).version), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["href"])) : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(_unref(buildInfo).env), 1 /* TEXT */)), (_unref(buildInfo).commit && _unref(buildInfo).branch !== 'release') ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createTextVNode("\n        &middot;\n        "), _createVNode(_component_NuxtLink, {
              external: "",
              href: `https://github.com/elk-zone/elk/commit/${_unref(buildInfo).commit}`,
              target: "_blank",
              "font-mono": ""
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(buildInfo).shortCommit), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["href"]) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", null, [ _createVNode(_component_NuxtLink, {
          "cursor-pointer": "",
          "hover:underline": "",
          to: "/settings/about"
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_ctx.$t('settings.about.label')), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }), (_unref(config).public.privacyPolicyUrl) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createTextVNode("\n        &middot;\n        "), _createVNode(_component_NuxtLink, {
              "cursor-pointer": "",
              "hover:underline": "",
              to: _unref(config).public.privacyPolicyUrl
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('nav.privacy')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to"]) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true), _createTextVNode("\n      &middot;\n      "), _createVNode(_component_NuxtLink, {
          href: "/m.webtoo.ls/@elk",
          target: "_blank"
        }, {
          default: _withCtx(() => [
            _createTextVNode("\n        Mastodon\n      ")
          ]),
          _: 1 /* STABLE */
        }), _createTextVNode("\n      &middot;\n      "), _createVNode(_component_NuxtLink, {
          href: "https://chat.elk.zone",
          target: "_blank",
          external: ""
        }, {
          default: _withCtx(() => [
            _createTextVNode("\n        Discord\n      ")
          ]),
          _: 1 /* STABLE */
        }), _createTextVNode("\n      &middot;\n      "), _createVNode(_component_NuxtLink, {
          href: "https://github.com/elk-zone/elk",
          target: "_blank",
          external: ""
        }, {
          default: _withCtx(() => [
            _createTextVNode("\n        GitHub\n      ")
          ]),
          _: 1 /* STABLE */
        }) ]) ]))
}
}

})
