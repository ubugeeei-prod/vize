import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx } from "vue"

import * as Misskey from 'misskey-js'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkRolePreview',
  props: {
    role: { type: null, required: true },
    forModeration: { type: Boolean, required: true },
    detailed: { type: Boolean, required: false, default: true }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")
  const _directive_adaptive_bg = _resolveDirective("adaptive-bg")

  return (_openBlock(), _createBlock(_component_MkA, {
      to: __props.forModeration ? `/admin/roles/${__props.role.id}` : `/roles/${__props.role.id}`,
      class: _normalizeClass(_ctx.$style.root),
      tabindex: "-1",
      style: _normalizeStyle({ '--color': __props.role.color })
    }, {
      default: _withCtx(() => [
        (__props.forModeration)
          ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
            ('isPublic' in __props.role && __props.role.isPublic)
              ? (_openBlock(), _createElementBlock("i", {
                key: 0,
                class: _normalizeClass(["ti ti-world", _ctx.$style.icon]),
                style: "color: var(--MI_THEME-success)"
              }))
              : (_openBlock(), _createElementBlock("i", {
                key: 1,
                class: _normalizeClass(["ti ti-lock", _ctx.$style.icon]),
                style: "color: var(--MI_THEME-warn)"
              }))
          ], 64 /* STABLE_FRAGMENT */))
          : _createCommentVNode("v-if", true),
        _createElementVNode("div", {
          class: _normalizeClass(["_panel", _ctx.$style.body])
        }, [
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.bodyTitle)
          }, [
            _createElementVNode("span", {
              class: _normalizeClass(_ctx.$style.bodyIcon)
            }, [
              (__props.role.iconUrl)
                ? (_openBlock(), _createElementBlock("img", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.bodyBadge),
                  src: __props.role.iconUrl
                }))
                : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                  (__props.role.isAdministrator)
                    ? (_openBlock(), _createElementBlock("i", {
                      key: 0,
                      class: "ti ti-crown",
                      style: "color: var(--MI_THEME-accent);"
                    }))
                    : (__props.role.isModerator)
                      ? (_openBlock(), _createElementBlock("i", {
                        key: 1,
                        class: "ti ti-shield",
                        style: "color: var(--MI_THEME-accent);"
                      }))
                    : (_openBlock(), _createElementBlock("i", {
                      key: 2,
                      class: "ti ti-user",
                      style: "opacity: 0.7;"
                    }))
                ], 64 /* STABLE_FRAGMENT */))
            ]),
            _createElementVNode("span", {
              class: _normalizeClass(_ctx.$style.bodyName)
            }, _toDisplayString(__props.role.name), 1 /* TEXT */),
            (__props.detailed && 'target' in __props.role && 'usersCount' in __props.role)
              ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                (__props.role.target === 'manual')
                  ? (_openBlock(), _createElementBlock("span", {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.bodyUsers)
                  }, _toDisplayString(__props.role.usersCount) + " users", 1 /* TEXT */))
                  : (__props.role.target === 'conditional')
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 1,
                      class: _normalizeClass(_ctx.$style.bodyUsers)
                    }, "? users"))
                  : _createCommentVNode("v-if", true)
              ], 64 /* STABLE_FRAGMENT */))
              : _createCommentVNode("v-if", true)
          ]),
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.bodyDescription)
          }, _toDisplayString(__props.role.description), 1 /* TEXT */)
        ])
      ]),
      _: 1 /* STABLE */
    }, 12 /* STYLE, PROPS */, ["to"]))
}
}

})
