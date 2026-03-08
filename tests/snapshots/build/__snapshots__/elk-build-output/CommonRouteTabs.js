import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { "ws-nowrap": "true", mxa: "true", "sm:px2": "true", "sm:py3": "true", "xl:pb4": "true", "xl:pt5": "true", py2: "true", "text-center": "true", "border-b-3": "true", "text-secondary-light": "true", "hover:text-secondary": "true", "border-transparent": "true" }
const _hoisted_2 = { "ws-nowrap": "true", mxa: "true", "sm:px2": "true", "sm:py3": "true", py2: "true", "text-center": "true", "text-secondary-light": "true", op50: "true" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { "i-ri:arrow-down-s-line": "true", "text-sm": "true", "me--1": "true", block: "true" })
import type { CommonRouteTabMoreOption, CommonRouteTabOption } from '#shared/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'CommonRouteTabs',
  props: {
    options: { type: Array, required: true },
    moreOptions: { type: null, required: false },
    command: { type: Boolean, required: false },
    replace: { type: Boolean, required: false },
    preventScrollTop: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {

const { t } = useI18n()
const router = useRouter()
useCommands(() => __props.command
  ? __props.options.map(tab => ({
      scope: 'Tabs',
      name: tab.display,
      icon: tab.icon ?? 'i-ri:file-list-2-line',
      onActivate: () => router.replace(tab.to),
    }))
  : [])

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")
  const _component_CommonDropdownItem = _resolveComponent("CommonDropdownItem")
  const _component_CommonDropdown = _resolveComponent("CommonDropdown")

  return (_openBlock(), _createElementBlock("div", {
      flex: "",
      "w-full": "",
      "items-center": "",
      "lg:text-lg": "",
      "of-x-auto": "",
      "scrollbar-hide": "",
      border: "b base"
    }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.options.filter(item => !item.hide), (option, index) => {
        return (_openBlock(), _createElementBlock(_Fragment, { key: option?.name || index }, [
          (!option.disabled)
            ? (_openBlock(), _createBlock(_component_NuxtLink, {
              key: 0,
              to: option.to,
              replace: __props.replace,
              relative: "",
              flex: "",
              "flex-auto": "",
              "cursor-pointer": "",
              "sm:px6": "",
              px2: "",
              rounded: "",
              "transition-all": "",
              tabindex: "0",
              "hover:bg-active": "",
              "transition-100": "",
              "exact-active-class": "children:(text-secondary !border-primary !op100 !text-base)",
              onClick: _cache[0] || (_cache[0] = ($event: any) => (!__props.preventScrollTop && _ctx.$scrollToTop()))
            }, {
              default: _withCtx(() => [
                _createElementVNode("span", _hoisted_1, _toDisplayString(option.display || '&nbsp;'), 1 /* TEXT */)
              ]),
              _: 2 /* DYNAMIC */
            }, 8 /* PROPS */, ["to", "replace"]))
            : (_openBlock(), _createElementBlock("div", {
              key: 1,
              flex: "",
              "flex-auto": "",
              "sm:px6": "",
              px2: "",
              "xl:pb4": "",
              "xl:pt5": ""
            }, [
              _createElementVNode("span", _hoisted_2, _toDisplayString(option.display), 1 /* TEXT */)
            ]))
        ], 64 /* STABLE_FRAGMENT */))
      }), 128 /* KEYED_FRAGMENT */)), (_ctx.isHydrated && __props.moreOptions?.options?.length) ? (_openBlock(), _createBlock(_component_CommonDropdown, {
          key: 0,
          placement: "bottom",
          flex: "",
          "cursor-pointer": "",
          "mx-1.25rem": ""
        }, {
          popper: _withCtx(() => [
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.moreOptions.options.filter(item => !item.hide), (option, index) => {
              return (_openBlock(), _createBlock(_component_NuxtLink, {
                key: option?.name || index,
                to: option.to
              }, {
                default: _withCtx(() => [
                  _createVNode(_component_CommonDropdownItem, null, {
                    default: _withCtx(() => [
                      _createElementVNode("span", {
                        flex: "~ row",
                        "gap-x-4": "",
                        "items-center": "",
                        class: _normalizeClass(option.match ? 'text-primary' : '')
                      }, [
                        (option.icon)
                          ? (_openBlock(), _createElementBlock("span", {
                            key: 0,
                            class: _normalizeClass([option.icon, option.match ? 'text-primary' : 'text.secondary']),
                            "text-md": "",
                            "me--1": "",
                            block: ""
                          }))
                          : (_openBlock(), _createElementBlock("span", {
                            key: 1,
                            block: ""
                          }, " ")),
                        _createElementVNode("span", null, _toDisplayString(option.display), 1 /* TEXT */)
                      ], 2 /* CLASS */)
                    ]),
                    _: 2 /* DYNAMIC */
                  })
                ]),
                _: 2 /* DYNAMIC */
              }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["to"]))
            }), 128 /* KEYED_FRAGMENT */))
          ]),
          default: _withCtx(() => [
            _createVNode(_component_CommonTooltip, {
              placement: "top",
              content: __props.moreOptions.tooltip || _unref(t)('action.more')
            }, {
              default: _withCtx(() => [
                _createElementVNode("button", {
                  "cursor-pointer": "",
                  flex: "",
                  "gap-1": "",
                  "w-12": "",
                  rounded: "",
                  "hover:bg-active": "",
                  "btn-action-icon": "",
                  op75: "",
                  px4: "",
                  group: "",
                  "aria-label": _unref(t)('action.more'),
                  class: _normalizeClass(__props.moreOptions.match ? 'text-primary' : 'text-secondary')
                }, [
                  (__props.moreOptions.icon)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(__props.moreOptions.icon),
                      "text-sm": "",
                      "me--1": "",
                      block: ""
                    }))
                    : _createCommentVNode("v-if", true),
                  _hoisted_3
                ], 10 /* CLASS, PROPS */, ["aria-label"])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["content"])
          ]),
          _: 1 /* STABLE */
        })) : _createCommentVNode("v-if", true) ]))
}
}

})
