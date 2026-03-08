import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:terminal w-3 h-3 text-fg-muted", "aria-hidden": "true" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:at-sign w-3 h-3 text-fg-muted", "aria-hidden": "true" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:terminal w-4 h-4 text-fg-muted", "aria-hidden": "true" })
const _hoisted_4 = { class: "font-mono text-sm text-fg truncate block" }
const _hoisted_5 = { class: "text-xs text-fg-subtle" }
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:at-sign w-4 h-4 text-fg-muted", "aria-hidden": "true" })
const _hoisted_7 = { class: "font-mono text-sm text-fg truncate block" }
const _hoisted_8 = { class: "text-xs text-fg-subtle" }
const _hoisted_9 = { class: "font-mono text-sm text-fg block" }
const _hoisted_10 = { class: "text-xs text-fg-subtle" }
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:at-sign w-4 h-4 text-fg-muted", "aria-hidden": "true" })
const _hoisted_12 = { class: "font-mono text-sm text-fg block" }
const _hoisted_13 = { class: "text-xs text-fg-subtle" }
import { useAtproto } from '~/composables/atproto/useAtproto'
import { useModal } from '~/composables/useModal'

export default /*@__PURE__*/_defineComponent({
  __name: 'AccountMenu.client',
  setup(__props) {

const {
  isConnected: isNpmConnected,
  isConnecting: isNpmConnecting,
  npmUser,
  avatar: npmAvatar,
  activeOperations,
  hasPendingOperations,
} = useConnector()
const { user: atprotoUser } = useAtproto()
const isOpen = shallowRef(false)
/** Check if connected to at least one service */
const hasAnyConnection = computed(() => isNpmConnected.value || !!atprotoUser.value)
/** Check if connected to both services */
const hasBothConnections = computed(() => isNpmConnected.value && !!atprotoUser.value)
/** Only show count of active (pending/approved/running) operations */
const operationCount = computed(() => activeOperations.value.length)
const accountMenuRef = useTemplateRef('accountMenuRef')
onClickOutside(accountMenuRef, () => {
  isOpen.value = false
})
useEventListener('keydown', event => {
  if (event.key === 'Escape' && isOpen.value) {
    isOpen.value = false
  }
})
const connectorModal = useModal('connector-modal')
function openConnectorModal() {
  if (connectorModal) {
    isOpen.value = false
    connectorModal.open()
  }
}
const authModal = useModal('auth-modal')
function openAuthModal() {
  if (authModal) {
    isOpen.value = false
    authModal.open()
  }
}

return (_ctx: any,_cache: any) => {
  const _component_ButtonBase = _resolveComponent("ButtonBase")
  const _component_HeaderConnectorModal = _resolveComponent("HeaderConnectorModal")
  const _component_HeaderAuthModal = _resolveComponent("HeaderAuthModal")

  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createElementVNode("div", {
        ref_key: "accountMenuRef", ref: accountMenuRef,
        class: "relative flex min-w-28 justify-end"
      }, [ _createVNode(_component_ButtonBase, {
          type: "button",
          "aria-expanded": isOpen.value,
          "aria-haspopup": "true",
          onClick: _cache[0] || (_cache[0] = ($event: any) => (isOpen.value = !isOpen.value)),
          class: "border-none"
        }, {
          default: _withCtx(() => [
            (hasAnyConnection.value)
              ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                class: _normalizeClass(["flex items-center", hasBothConnections.value ? '-space-x-2' : ''])
              }, [
                (_unref(isNpmConnected) && _unref(npmAvatar))
                  ? (_openBlock(), _createElementBlock("img", {
                    key: 0,
                    src: _unref(npmAvatar),
                    alt: _unref(npmUser) || _ctx.$t('account_menu.npm_cli'),
                    width: "24",
                    height: "24",
                    class: "w-6 h-6 rounded-full ring-2 ring-bg object-cover"
                  }))
                  : (_unref(isNpmConnected))
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 1,
                      class: "w-6 h-6 rounded-full bg-bg-muted ring-2 ring-bg flex items-center justify-center"
                    }, [
                      _hoisted_1
                    ]))
                  : _createCommentVNode("v-if", true),
                _createTextVNode("\n\n        "),
                _createTextVNode("\n        "),
                (_unref(atprotoUser)?.avatar)
                  ? (_openBlock(), _createElementBlock("img", {
                    key: 0,
                    src: _unref(atprotoUser).avatar,
                    alt: _unref(atprotoUser).handle,
                    width: "24",
                    height: "24",
                    class: _normalizeClass(["w-6 h-6 rounded-full ring-2 ring-bg object-cover", hasBothConnections.value ? 'relative z-10' : ''])
                  }))
                  : (_unref(atprotoUser))
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 1,
                      class: _normalizeClass(["w-6 h-6 rounded-full bg-bg-muted ring-2 ring-bg flex items-center justify-center", hasBothConnections.value ? 'relative z-10' : ''])
                    }, [
                      _hoisted_2
                    ]))
                  : _createCommentVNode("v-if", true)
              ]))
              : _createCommentVNode("v-if", true),
            _createTextVNode("\n\n      "),
            _createTextVNode("\n      "),
            (!hasAnyConnection.value)
              ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                class: "font-mono text-sm"
              }, _toDisplayString(_ctx.$t('account_menu.connect')), 1 /* TEXT */))
              : _createCommentVNode("v-if", true),
            _createTextVNode("\n\n      "),
            _createTextVNode("\n      "),
            _createElementVNode("span", {
              class: _normalizeClass(["i-lucide:chevron-down w-3 h-3 transition-transform duration-200", { 'rotate-180': isOpen.value }]),
              "aria-hidden": "true"
            }, null, 2 /* CLASS */),
            _createTextVNode("\n\n      "),
            _createTextVNode("\n      "),
            (_unref(isNpmConnected) && operationCount.value > 0)
              ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                class: _normalizeClass(["absolute -top-1 -inset-ie-1 min-w-[1rem] h-4 px-1 flex items-center justify-center font-mono text-3xs rounded-full", _unref(hasPendingOperations) ? 'bg-yellow-500 text-black' : 'bg-blue-500 text-white']),
                "aria-hidden": "true"
              }, _toDisplayString(operationCount.value), 1 /* TEXT */))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["aria-expanded"]), _createTextVNode("\n\n    " + "\n    "), _createVNode(_Transition, {
          "enter-active-class": "transition-all duration-150",
          "leave-active-class": "transition-all duration-100",
          "enter-from-class": "opacity-0 translate-y-1",
          "leave-to-class": "opacity-0 translate-y-1"
        }, {
          default: _withCtx(() => [
            (isOpen.value)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: "absolute inset-ie-0 top-full pt-2 w-72 z-50",
                role: "menu"
              }, [
                _createElementVNode("div", { class: "bg-bg-subtle/80 backdrop-blur-sm border border-border-subtle rounded-lg shadow-lg shadow-bg-elevated/50 overflow-hidden px-1" }, [
                  (hasAnyConnection.value)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "py-1"
                    }, [
                      (_unref(isNpmConnected) && _unref(npmUser))
                        ? (_openBlock(), _createBlock(_component_ButtonBase, {
                          key: 0,
                          role: "menuitem",
                          class: "w-full text-start gap-x-3 border-none",
                          onClick: openConnectorModal,
                          out: ""
                        }, {
                          default: _withCtx(() => [
                            (_unref(npmAvatar))
                              ? (_openBlock(), _createElementBlock("img", {
                                key: 0,
                                src: _unref(npmAvatar),
                                alt: _unref(npmUser),
                                width: "32",
                                height: "32",
                                class: "w-8 h-8 rounded-full object-cover"
                              }))
                              : (_openBlock(), _createElementBlock("span", {
                                key: 1,
                                class: "w-8 h-8 rounded-full bg-bg-muted flex items-center justify-center"
                              }, [
                                _hoisted_3
                              ])),
                            _createElementVNode("span", { class: "flex-1 min-w-0" }, [
                              _createElementVNode("span", _hoisted_4, "~" + _toDisplayString(_unref(npmUser)), 1 /* TEXT */),
                              _createElementVNode("span", _hoisted_5, _toDisplayString(_ctx.$t('account_menu.npm_cli')), 1 /* TEXT */)
                            ]),
                            (operationCount.value > 0)
                              ? (_openBlock(), _createElementBlock("span", {
                                key: 0,
                                class: _normalizeClass(["px-1.5 py-0.5 font-mono text-xs rounded", 
                    _unref(hasPendingOperations)
                      ? 'bg-yellow-500/20 text-yellow-600'
                      : 'bg-blue-500/20 text-blue-500'
                  ])
                              }, _toDisplayString(_ctx.$t('account_menu.ops', {
                      count: operationCount.value,
                    })), 1 /* TEXT */))
                              : _createCommentVNode("v-if", true)
                          ]),
                          _: 1 /* STABLE */
                        }))
                        : _createCommentVNode("v-if", true),
                      _createTextVNode("\n\n            "),
                      _createTextVNode("\n            "),
                      (_unref(atprotoUser))
                        ? (_openBlock(), _createBlock(_component_ButtonBase, {
                          key: 0,
                          role: "menuitem",
                          class: "w-full text-start gap-x-3 border-none",
                          onClick: openAuthModal
                        }, {
                          default: _withCtx(() => [
                            (_unref(atprotoUser).avatar)
                              ? (_openBlock(), _createElementBlock("img", {
                                key: 0,
                                src: _unref(atprotoUser).avatar,
                                alt: _unref(atprotoUser).handle,
                                width: "32",
                                height: "32",
                                class: "w-8 h-8 rounded-full object-cover"
                              }))
                              : (_openBlock(), _createElementBlock("span", {
                                key: 1,
                                class: "w-8 h-8 rounded-full bg-bg-muted flex items-center justify-center"
                              }, [
                                _hoisted_6
                              ])),
                            _createElementVNode("span", { class: "flex-1 min-w-0" }, [
                              _createElementVNode("span", _hoisted_7, "@" + _toDisplayString(_unref(atprotoUser).handle), 1 /* TEXT */),
                              _createElementVNode("span", _hoisted_8, _toDisplayString(_ctx.$t('account_menu.atmosphere')), 1 /* TEXT */)
                            ])
                          ]),
                          _: 1 /* STABLE */
                        }))
                        : _createCommentVNode("v-if", true)
                    ]))
                    : _createCommentVNode("v-if", true),
                  _createTextVNode("\n\n          " + "\n          "),
                  (hasAnyConnection.value && (!_unref(isNpmConnected) || !_unref(atprotoUser)))
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "border-t border-border"
                    }))
                    : _createCommentVNode("v-if", true),
                  _createTextVNode("\n\n          " + "\n          "),
                  (!_unref(isNpmConnected) || !_unref(atprotoUser))
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "py-1"
                    }, [
                      (!_unref(isNpmConnected))
                        ? (_openBlock(), _createBlock(_component_ButtonBase, {
                          key: 0,
                          role: "menuitem",
                          class: "w-full text-start gap-x-3 border-none",
                          onClick: openConnectorModal
                        }, {
                          default: _withCtx(() => [
                            _createElementVNode("span", { class: "w-8 h-8 rounded-full bg-bg-muted flex items-center justify-center" }, [
                              (_unref(isNpmConnecting))
                                ? (_openBlock(), _createElementBlock("span", {
                                  key: 0,
                                  class: "i-svg-spinners:ring-resize w-4 h-4 text-yellow-500 animate-spin",
                                  "aria-hidden": "true"
                                }))
                                : (_openBlock(), _createElementBlock("span", {
                                  key: 1,
                                  class: "i-lucide:terminal w-4 h-4 text-fg-muted",
                                  "aria-hidden": "true"
                                }))
                            ]),
                            _createElementVNode("span", { class: "flex-1 min-w-0" }, [
                              _createElementVNode("span", _hoisted_9, _toDisplayString(_unref(isNpmConnecting)
                        ? _ctx.$t('account_menu.connecting')
                        : _ctx.$t('account_menu.connect_npm_cli')), 1 /* TEXT */),
                              _createElementVNode("span", _hoisted_10, _toDisplayString(_ctx.$t('account_menu.npm_cli_desc')), 1 /* TEXT */)
                            ])
                          ]),
                          _: 1 /* STABLE */
                        }))
                        : _createCommentVNode("v-if", true),
                      (!_unref(atprotoUser))
                        ? (_openBlock(), _createBlock(_component_ButtonBase, {
                          key: 0,
                          role: "menuitem",
                          class: "w-full text-start gap-x-3 border-none",
                          onClick: openAuthModal
                        }, {
                          default: _withCtx(() => [
                            _createElementVNode("span", { class: "w-8 h-8 rounded-full bg-bg-muted flex items-center justify-center" }, [
                              _hoisted_11
                            ]),
                            _createElementVNode("span", { class: "flex-1 min-w-0" }, [
                              _createElementVNode("span", _hoisted_12, _toDisplayString(_ctx.$t('account_menu.connect_atmosphere')), 1 /* TEXT */),
                              _createElementVNode("span", _hoisted_13, _toDisplayString(_ctx.$t('account_menu.atmosphere_desc')), 1 /* TEXT */)
                            ])
                          ]),
                          _: 1 /* STABLE */
                        }))
                        : _createCommentVNode("v-if", true)
                    ]))
                    : _createCommentVNode("v-if", true)
                ])
              ]))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }) ], 512 /* NEED_PATCH */), _createVNode(_component_HeaderConnectorModal), _createVNode(_component_HeaderAuthModal) ], 64 /* STABLE_FRAGMENT */))
}
}

})
