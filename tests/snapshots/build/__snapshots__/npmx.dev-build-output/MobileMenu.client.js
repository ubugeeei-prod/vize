import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Teleport as _Teleport, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "font-mono text-sm text-fg-muted" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:x w-5 h-5", "aria-hidden": "true" })
const _hoisted_3 = { class: "px-3 py-2 block font-mono text-xs text-fg-subtle uppercase tracking-wider" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:terminal w-3 h-3 text-fg-muted", "aria-hidden": "true" })
const _hoisted_5 = { class: "flex-1" }
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("span", { class: "w-2 h-2 rounded-full bg-green-500", "aria-hidden": "true" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:at-sign w-3 h-3 text-fg-muted", "aria-hidden": "true" })
const _hoisted_8 = { class: "flex-1 truncate" }
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:at-sign w-3 h-3 text-fg-muted", "aria-hidden": "true" })
const _hoisted_10 = { class: "flex-1" }
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("div", { class: "mx-4 my-2 border-t border-border" })
import { useFocusTrap } from '@vueuse/integrations/useFocusTrap'
import { useAtproto } from '~/composables/atproto/useAtproto'
import type { NavigationConfigWithGroups } from '~/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'MobileMenu.client',
  props: {
    links: { type: null, required: true },
    "open": { default: false }
  },
  emits: ["update:open"],
  setup(__props: any) {

const isOpen = _useModel(__props, "open")
const { isConnected, npmUser, avatar: npmAvatar } = useConnector()
const { user: atprotoUser } = useAtproto()
const navRef = useTemplateRef('navRef')
const { activate, deactivate } = useFocusTrap(navRef, { allowOutsideClick: true })
function closeMenu() {
  isOpen.value = false
}
function handleShowConnector() {
  const connectorModal = document.querySelector<HTMLDialogElement>('#connector-modal')
  if (connectorModal) {
    closeMenu()
    connectorModal.showModal()
  }
}
function handleShowAuth() {
  const authModal = document.querySelector<HTMLDialogElement>('#auth-modal')
  if (authModal) {
    closeMenu()
    authModal.showModal()
  }
}
// Close menu on route change
const route = useRoute()
watch(() => route.fullPath, closeMenu)
// Close on escape
onKeyStroke(
  e => isKeyWithoutModifiers(e, 'Escape') && isOpen.value,
  e => {
    isOpen.value = false
  },
)
// Prevent body scroll when menu is open
const isLocked = useScrollLock(document)
watch(isOpen, open => (isLocked.value = open))
watch(isOpen, open => (open ? nextTick(activate) : deactivate()))
onUnmounted(deactivate)

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (_openBlock(), _createBlock(_Teleport, { to: "body" }, [ _createVNode(_Transition, {
        "enter-active-class": "transition-opacity duration-200",
        "leave-active-class": "transition-opacity duration-150",
        "enter-from-class": "opacity-0",
        "leave-to-class": "opacity-0"
      }, {
        default: _withCtx(() => [
          (isOpen.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "fixed inset-0 z-[60] sm:hidden",
              role: "dialog",
              "aria-modal": "true",
              "aria-label": _ctx.$t('nav.mobile_menu')
            }, [
              _createElementVNode("button", {
                type: "button",
                class: "absolute inset-0 bg-black/60 cursor-default",
                "aria-label": _ctx.$t('common.close'),
                onClick: closeMenu
              }, null, 8 /* PROPS */, ["aria-label"]),
              _createTextVNode("\n\n        "),
              _createTextVNode("\n        "),
              _createVNode(_Transition, {
                "enter-active-class": "transition-transform duration-200",
                "enter-from-class": "translate-x-full",
                "enter-to-class": "translate-x-0",
                "leave-active-class": "transition-transform duration-200",
                "leave-from-class": "translate-x-0",
                "leave-to-class": "translate-x-full"
              }, {
                default: _withCtx(() => [
                  (isOpen.value)
                    ? (_openBlock(), _createElementBlock("nav", {
                      key: 0,
                      ref: "navRef",
                      class: "absolute inset-ie-0 top-0 bottom-0 w-72 bg-bg border-is border-border shadow-xl flex flex-col"
                    }, [
                      _createElementVNode("div", { class: "flex items-center justify-between p-4 border-b border-border" }, [
                        _createElementVNode("span", _hoisted_1, _toDisplayString(_ctx.$t('nav.menu')), 1 /* TEXT */),
                        _createElementVNode("button", {
                          type: "button",
                          class: "p-2 -m-2 text-fg-subtle hover:text-fg transition-colors duration-200 focus-visible:outline-accent/70 rounded",
                          "aria-label": _ctx.$t('common.close'),
                          onClick: closeMenu
                        }, [
                          _hoisted_2
                        ], 8 /* PROPS */, ["aria-label"])
                      ]),
                      _createTextVNode("\n\n            "),
                      _createTextVNode("\n            "),
                      _createElementVNode("div", { class: "px-2 py-2" }, [
                        _createElementVNode("span", _hoisted_3, _toDisplayString(_ctx.$t('account_menu.account')), 1 /* TEXT */),
                        _createTextVNode("\n\n              " + "\n              "),
                        (_unref(isConnected) && _unref(npmUser))
                          ? (_openBlock(), _createElementBlock("button", {
                            key: 0,
                            type: "button",
                            class: "w-full flex items-center gap-3 px-3 py-3 rounded-md font-mono text-sm text-fg hover:bg-bg-subtle transition-colors duration-200 text-start",
                            onClick: handleShowConnector
                          }, [
                            (_unref(npmAvatar))
                              ? (_openBlock(), _createElementBlock("img", {
                                key: 0,
                                src: _unref(npmAvatar),
                                alt: _unref(npmUser),
                                width: "20",
                                height: "20",
                                class: "w-5 h-5 rounded-full object-cover"
                              }))
                              : (_openBlock(), _createElementBlock("span", {
                                key: 1,
                                class: "w-5 h-5 rounded-full bg-bg-muted flex items-center justify-center"
                              }, [
                                _hoisted_4
                              ])),
                            _createElementVNode("span", _hoisted_5, "~" + _toDisplayString(_unref(npmUser)), 1 /* TEXT */),
                            _hoisted_6
                          ]))
                          : _createCommentVNode("v-if", true),
                        _createTextVNode("\n\n              " + "\n              "),
                        (_unref(atprotoUser))
                          ? (_openBlock(), _createElementBlock("button", {
                            key: 0,
                            type: "button",
                            class: "w-full flex items-center gap-3 px-3 py-3 rounded-md font-mono text-sm text-fg hover:bg-bg-subtle transition-colors duration-200 text-start",
                            onClick: handleShowAuth
                          }, [
                            (_unref(atprotoUser).avatar)
                              ? (_openBlock(), _createElementBlock("img", {
                                key: 0,
                                src: _unref(atprotoUser).avatar,
                                alt: _unref(atprotoUser).handle,
                                width: "20",
                                height: "20",
                                class: "w-5 h-5 rounded-full object-cover"
                              }))
                              : (_openBlock(), _createElementBlock("span", {
                                key: 1,
                                class: "w-5 h-5 rounded-full bg-bg-muted flex items-center justify-center"
                              }, [
                                _hoisted_7
                              ])),
                            _createElementVNode("span", _hoisted_8, "@" + _toDisplayString(_unref(atprotoUser).handle), 1 /* TEXT */)
                          ]))
                          : (_openBlock(), _createElementBlock("button", {
                            key: 1,
                            type: "button",
                            class: "w-full flex items-center gap-3 px-3 py-3 rounded-md font-mono text-sm text-fg hover:bg-bg-subtle transition-colors duration-200 text-start",
                            onClick: handleShowAuth
                          }, [
                            _createElementVNode("span", { class: "w-5 h-5 rounded-full bg-bg-muted flex items-center justify-center" }, [
                              _hoisted_9
                            ]),
                            _createElementVNode("span", _hoisted_10, _toDisplayString(_ctx.$t('account_menu.connect_atmosphere')), 1 /* TEXT */)
                          ])),
                        _createTextVNode("\n\n              " + "\n              ")
                      ]),
                      _createTextVNode("\n\n            "),
                      _createTextVNode("\n            "),
                      _hoisted_11,
                      _createTextVNode("\n\n            "),
                      _createTextVNode("\n            "),
                      _createElementVNode("div", { class: "flex-1 overflow-y-auto overscroll-contain py-2" }, [
                        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.links, (group, index) => {
                          return (_openBlock(), _createElementBlock(_Fragment, null, [
                            (group.type === 'separator')
                              ? (_openBlock(), _createElementBlock("div", {
                                key: `seperator-${index}`,
                                class: "mx-4 my-2 border-t border-border"
                              }))
                              : _createCommentVNode("v-if", true),
                            (group.type === 'group')
                              ? (_openBlock(), _createElementBlock("div", {
                                key: group.name,
                                class: "p-2"
                              }, [
                                (group.label)
                                  ? (_openBlock(), _createElementBlock("span", {
                                    key: 0,
                                    class: "px-3 py-2 font-mono text-xs text-fg-subtle uppercase tracking-wider"
                                  }, _toDisplayString(group.label), 1 /* TEXT */))
                                  : _createCommentVNode("v-if", true),
                                _createElementVNode("div", null, [
                                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(group.items, (link) => {
                                    return (_openBlock(), _createBlock(_component_NuxtLink, {
                                      key: link.name,
                                      to: link.to,
                                      href: link.href,
                                      target: link.target,
                                      class: "flex items-center gap-3 px-3 py-3 rounded-md font-mono text-sm text-fg hover:bg-bg-subtle transition-colors duration-200",
                                      onClick: closeMenu
                                    }, {
                                      default: _withCtx(() => [
                                        _createElementVNode("span", {
                                          class: _normalizeClass(["w-5 h-5 text-fg-muted", link.iconClass]),
                                          "aria-hidden": "true"
                                        }, null, 2 /* CLASS */),
                                        _createTextVNode("\n                      "),
                                        _createTextVNode(_toDisplayString(link.label), 1 /* TEXT */),
                                        _createTextVNode("\n                      "),
                                        (link.external)
                                          ? (_openBlock(), _createElementBlock("span", {
                                            key: 0,
                                            class: "i-lucide:external-link rtl-flip w-3 h-3 ms-auto text-fg-subtle",
                                            "aria-hidden": "true"
                                          }))
                                          : _createCommentVNode("v-if", true)
                                      ]),
                                      _: 2 /* DYNAMIC */
                                    }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["to", "href", "target"]))
                                  }), 128 /* KEYED_FRAGMENT */))
                                ])
                              ]))
                              : _createCommentVNode("v-if", true)
                          ], 64 /* STABLE_FRAGMENT */))
                        }), 256 /* UNKEYED_FRAGMENT */))
                      ])
                    ]))
                    : _createCommentVNode("v-if", true)
                ]),
                _: 1 /* STABLE */
              })
            ]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }) ]))
}
}

})
