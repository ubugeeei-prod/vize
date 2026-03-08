import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "w-3 h-3 rounded-full bg-green-500", "aria-hidden": "true" })
const _hoisted_2 = { class: "font-mono text-xs text-fg-muted" }
const _hoisted_3 = { class: "text-sm text-fg-muted" }
const _hoisted_4 = { for: "handle-input", class: "block text-xs text-fg-subtle uppercase tracking-wider mb-1.5" }
const _hoisted_5 = { class: "text-fg-subtle hover:text-fg-muted transition-colors duration-200 focus-visible:(outline-2 outline-accent/70)" }
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("span", { class: "font-bold" }, "npmx.dev")
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("hr", { class: "color-border" })
import { useAtproto } from '~/composables/atproto/useAtproto'
import { authRedirect } from '~/utils/atproto/helpers'
import { isAtIdentifierString } from '@atproto/lex'

export default /*@__PURE__*/_defineComponent({
  __name: 'AuthModal.client',
  setup(__props) {

const authModal = useModal('auth-modal')
const handleInput = shallowRef('')
const errorMessage = shallowRef('')
const route = useRoute()
const { user, logout } = useAtproto()
// https://atproto.com supports 4 locales as of 2026-02-07
const { locale } = useI18n()
const currentLang = locale.value.split('-')[0] ?? 'en'
const localeSubPath = ['ko', 'pt', 'ja'].includes(currentLang) ? currentLang : ''
const atprotoLink = `https://atproto.com/${localeSubPath}`
async function handleBlueskySignIn() {
  await authRedirect('https://bsky.social', { redirectTo: route.fullPath, locale: locale.value })
}
async function handleCreateAccount() {
  await authRedirect('https://npmx.social', {
    create: true,
    redirectTo: route.fullPath,
    locale: locale.value,
  })
}
async function handleLogin() {
  if (handleInput.value) {
    // URLS to PDSs are valid for initiating oauth flows
    if (handleInput.value.startsWith('https://') || isAtIdentifierString(handleInput.value)) {
      await authRedirect(handleInput.value, {
        redirectTo: route.fullPath,
        locale: locale.value,
      })
    } else {
      errorMessage.value = $t('auth.modal.default_input_error')
    }
  }
}
watch(handleInput, newHandleInput => {
  errorMessage.value = ''
  if (!newHandleInput) return
  const normalized = newHandleInput.trim().toLowerCase().replace(/@/g, '')
  if (normalized !== newHandleInput) {
    handleInput.value = normalized
  }
})
watch(user, async newUser => {
  if (newUser?.relogin) {
    await authRedirect(newUser.did, {
      redirectTo: route.fullPath,
    })
  }
})

return (_ctx: any,_cache: any) => {
  const _component_LinkBase = _resolveComponent("LinkBase")
  const _component_ButtonBase = _resolveComponent("ButtonBase")
  const _component_InputBase = _resolveComponent("InputBase")
  const _component_i18n_t = _resolveComponent("i18n-t")
  const _component_Modal = _resolveComponent("Modal")

  return (_openBlock(), _createBlock(_component_Modal, {
      modalTitle: _ctx.$t('auth.modal.title'),
      class: "max-w-lg",
      id: "auth-modal"
    }, {
      default: _withCtx(() => [
        (_unref(user)?.handle)
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "space-y-4"
          }, [
            _createElementVNode("div", { class: "flex items-center gap-3 p-4 bg-bg-subtle border border-border rounded-lg" }, [
              _hoisted_1,
              _createElementVNode("div", null, [
                _createElementVNode("p", _hoisted_2, _toDisplayString(_ctx.$t('auth.modal.connected_as', { handle: _unref(user).handle })), 1 /* TEXT */)
              ])
            ]),
            _createElementVNode("div", { class: "flex flex-col space-y-4" }, [
              _createVNode(_component_LinkBase, {
                variant: "button-secondary",
                to: { name: 'profile-handle', params: { handle: _unref(user).handle } },
                "prefetch-on": "interaction",
                class: "w-full",
                onClick: _cache[0] || (_cache[0] = ($event: any) => (_unref(authModal).close()))
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_ctx.$t('auth.modal.profile')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to"]),
              _createVNode(_component_ButtonBase, {
                class: "w-full",
                onClick: _cache[1] || (_cache[1] = (...args) => (logout && logout(...args)))
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_ctx.$t('auth.modal.disconnect')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ])
          ]))
          : (_openBlock(), _createElementBlock("form", {
            key: 1,
            class: "space-y-4",
            onSubmit: _withModifiers(handleLogin, ["prevent"])
          }, [
            _createElementVNode("p", _hoisted_3, _toDisplayString(_ctx.$t('auth.modal.connect_prompt')), 1 /* TEXT */),
            _createElementVNode("div", { class: "space-y-3" }, [
              _createElementVNode("div", null, [
                _createElementVNode("label", _hoisted_4, _toDisplayString(_ctx.$t('auth.modal.handle_label')), 1 /* TEXT */),
                _createVNode(_component_InputBase, {
                  id: "handle-input",
                  type: "text",
                  name: "handle",
                  placeholder: _ctx.$t('auth.modal.handle_placeholder'),
                  "no-correct": "",
                  class: "w-full",
                  size: "medium",
                  modelValue: handleInput.value,
                  "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((handleInput).value = $event))
                }, null, 8 /* PROPS */, ["placeholder", "modelValue"]),
                (errorMessage.value)
                  ? (_openBlock(), _createElementBlock("p", {
                    key: 0,
                    class: "text-red-500 text-xs mt-1",
                    role: "alert"
                  }, _toDisplayString(errorMessage.value), 1 /* TEXT */))
                  : _createCommentVNode("v-if", true)
              ]),
              _createElementVNode("details", { class: "text-sm" }, [
                _createElementVNode("summary", _hoisted_5, _toDisplayString(_ctx.$t('auth.modal.what_is_atmosphere')), 1 /* TEXT */),
                _createElementVNode("div", { class: "mt-3" }, [
                  _createVNode(_component_i18n_t, {
                    keypath: "auth.modal.atmosphere_explanation",
                    tag: "p",
                    scope: "global"
                  }, {
                    npmx: _withCtx(() => [
                      _hoisted_6
                    ]),
                    atproto: _withCtx(() => [
                      _createVNode(_component_LinkBase, { to: _unref(atprotoLink) }, {
                        default: _withCtx(() => [
                          _createTextVNode(" AT Protocol ")
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["to"])
                    ]),
                    bluesky: _withCtx(() => [
                      _createVNode(_component_LinkBase, { to: "https://bsky.app" }, {
                        default: _withCtx(() => [
                          _createTextVNode(" Bluesky ")
                        ]),
                        _: 1 /* STABLE */
                      })
                    ]),
                    tangled: _withCtx(() => [
                      _createVNode(_component_LinkBase, { to: "https://tangled.org" }, {
                        default: _withCtx(() => [
                          _createTextVNode(" Tangled ")
                        ]),
                        _: 1 /* STABLE */
                      })
                    ]),
                    _: 1 /* STABLE */
                  })
                ])
              ])
            ]),
            _createVNode(_component_ButtonBase, {
              type: "submit",
              variant: "primary",
              disabled: !handleInput.value.trim(),
              class: "w-full"
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('auth.modal.connect')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["disabled"]),
            _createVNode(_component_ButtonBase, {
              type: "button",
              class: "w-full",
              onClick: handleCreateAccount
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('auth.modal.create_account')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }),
            _hoisted_7,
            _createVNode(_component_ButtonBase, {
              type: "button",
              class: "w-full",
              onClick: handleBlueskySignIn,
              classicon: "i-simple-icons:bluesky"
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('auth.modal.connect_bluesky')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            })
          ])),
        _createTextVNode("\n\n    "),
        _createTextVNode("\n    ")
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["modalTitle"]))
}
}

})
