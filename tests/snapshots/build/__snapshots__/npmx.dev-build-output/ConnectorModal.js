import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "w-3 h-3 rounded-full bg-green-500", "aria-hidden": "true" })
const _hoisted_2 = { class: "font-mono text-sm text-fg" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("div", { class: "border-t border-border my-3" })
const _hoisted_4 = { class: "inline-block text-xs font-bold uppercase tracking-wider text-fg rounded" }
const _hoisted_5 = { class: "text-sm text-fg-muted" }
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle" }, "$")
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle ms-2" }, "pnpm npmx-connector")
const _hoisted_8 = { class: "text-sm text-fg-muted" }
const _hoisted_9 = { for: "connector-token", class: "block text-xs text-fg-subtle uppercase tracking-wider mb-1.5" }
const _hoisted_10 = { class: "text-fg-subtle hover:text-fg-muted transition-colors duration-200" }
const _hoisted_11 = { for: "connector-port", class: "block text-xs text-fg-subtle uppercase tracking-wider mb-1.5" }
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("div", { class: "border-t border-border my-3" })
const _hoisted_13 = { class: "inline-block text-xs font-bold uppercase tracking-wider text-fg rounded" }
const _hoisted_14 = { class: "text-sm text-fg-muted mt-1" }

export default /*@__PURE__*/_defineComponent({
  __name: 'ConnectorModal',
  setup(__props) {

const { isConnected, isConnecting, npmUser, error, hasOperations, connect, disconnect } =
  useConnector()
const { settings } = useSettings()
const tokenInput = shallowRef('')
const portInput = shallowRef('31415')
const { copied, copy } = useClipboard({ copiedDuring: 2000 })
const hasAttemptedConnect = shallowRef(false)
watch(isConnected, connected => {
  if (!connected) {
    tokenInput.value = ''
    hasAttemptedConnect.value = false
  }
})
async function handleConnect() {
  hasAttemptedConnect.value = true
  const port = Number.parseInt(portInput.value, 10) || 31415
  await connect(tokenInput.value.trim(), port)
}
function handleDisconnect() {
  disconnect()
}
// function copyCommand() {
//   let command = executeNpmxConnectorCommand.value
//   if (portInput.value !== '31415') {
//     command += ` --port ${portInput.value}`
//   }
//   copy(command)
// }
// const selectedPM = useSelectedPackageManager()
// const executeNpmxConnectorCommand = computed(() => {
//   return getExecuteCommand({
//     packageName: 'npmx-connector',
//     packageManager: selectedPM.value,
//   })
// })

return (_ctx: any,_cache: any) => {
  const _component_SettingsToggle = _resolveComponent("SettingsToggle")
  const _component_OrgOperationsQueue = _resolveComponent("OrgOperationsQueue")
  const _component_ButtonBase = _resolveComponent("ButtonBase")
  const _component_LinkBase = _resolveComponent("LinkBase")
  const _component_i18n_t = _resolveComponent("i18n-t")
  const _component_InputBase = _resolveComponent("InputBase")
  const _component_Modal = _resolveComponent("Modal")

  return (_openBlock(), _createBlock(_component_Modal, {
      modalTitle: _ctx.$t('connector.modal.title'),
      class: _normalizeClass(_unref(isConnected) && _unref(hasOperations) ? 'max-w-2xl' : 'max-w-md'),
      id: "connector-modal"
    }, {
      default: _withCtx(() => [
        (_unref(isConnected))
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "space-y-4"
          }, [
            _createElementVNode("div", { class: "flex items-center gap-3 p-4 bg-bg-subtle border border-border rounded-lg" }, [
              _hoisted_1,
              _createElementVNode("div", null, [
                _createElementVNode("p", _hoisted_2, _toDisplayString(_ctx.$t('connector.modal.connected')), 1 /* TEXT */),
                (_unref(npmUser))
                  ? (_openBlock(), _createElementBlock("p", {
                    key: 0,
                    class: "font-mono text-xs text-fg-muted"
                  }, _toDisplayString(_ctx.$t('connector.modal.connected_as_user', { user: _unref(npmUser) })), 1 /* TEXT */))
                  : _createCommentVNode("v-if", true)
              ])
            ]),
            _createTextVNode("\n\n      "),
            _createTextVNode("\n      "),
            _createElementVNode("div", { class: "flex flex-col gap-2" }, [
              _createVNode(_component_SettingsToggle, {
                label: _ctx.$t('connector.modal.auto_open_url'),
                modelValue: _unref(settings).connector.autoOpenURL,
                "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((_unref(settings).connector.autoOpenURL) = $event))
              }, null, 8 /* PROPS */, ["label", "modelValue"])
            ]),
            _hoisted_3,
            _createTextVNode("\n\n      "),
            _createTextVNode("\n      "),
            _createVNode(_component_OrgOperationsQueue),
            (!_unref(hasOperations))
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: "text-sm text-fg-muted"
              }, _toDisplayString(_ctx.$t('connector.modal.connected_hint')), 1 /* TEXT */))
              : _createCommentVNode("v-if", true),
            _createVNode(_component_ButtonBase, {
              type: "button",
              class: "w-full",
              onClick: handleDisconnect
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('connector.modal.disconnect')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            })
          ]))
          : (_openBlock(), _createElementBlock("form", {
            key: 1,
            class: "space-y-4",
            onSubmit: _withModifiers(handleConnect, ["prevent"])
          }, [
            _createElementVNode("div", { class: "p-3 bg-amber-500/10 border border-amber-500/30 rounded-lg" }, [
              _createElementVNode("div", null, [
                _createElementVNode("span", _hoisted_4, _toDisplayString(_ctx.$t('connector.modal.contributor_badge')), 1 /* TEXT */),
                _createElementVNode("p", { class: "text-sm text-fg-muted" }, [
                  _createVNode(_component_i18n_t, {
                    keypath: "connector.modal.contributor_notice",
                    scope: "global"
                  }, {
                    link: _withCtx(() => [
                      _createVNode(_component_LinkBase, { to: "https://github.com/npmx-dev/npmx.dev/blob/main/CONTRIBUTING.md#local-connector-cli" }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_ctx.$t('connector.modal.contributor_link')), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      })
                    ]),
                    _: 1 /* STABLE */
                  })
                ])
              ])
            ]),
            _createElementVNode("p", _hoisted_5, _toDisplayString(_ctx.$t('connector.modal.run_hint')), 1 /* TEXT */),
            _createElementVNode("div", {
              class: "flex items-center p-3 bg-bg-muted border border-border rounded-lg font-mono text-sm",
              dir: "ltr"
            }, [
              _hoisted_6,
              _hoisted_7,
              _createVNode(_component_ButtonBase, {
                "aria-label": _unref(copied) ? _ctx.$t('connector.modal.copied') : _ctx.$t('connector.modal.copy_command'),
                onClick: _cache[1] || (_cache[1] = ($event: any) => (_unref(copy)('pnpm npmx-connector'))),
                class: "ms-auto",
                classicon: _unref(copied) ? 'i-lucide:check text-green-500' : 'i-lucide:copy'
              }, null, 8 /* PROPS */, ["aria-label", "classicon"])
            ]),
            _createTextVNode("\n\n      "),
            _createTextVNode("\n\n      "),
            _createElementVNode("p", _hoisted_8, _toDisplayString(_ctx.$t('connector.modal.paste_token')), 1 /* TEXT */),
            _createElementVNode("div", { class: "space-y-3" }, [
              _createElementVNode("div", null, [
                _createElementVNode("label", _hoisted_9, _toDisplayString(_ctx.$t('connector.modal.token_label')), 1 /* TEXT */),
                _createVNode(_component_InputBase, {
                  id: "connector-token",
                  type: "password",
                  name: "connector-token",
                  placeholder: _ctx.$t('connector.modal.token_placeholder'),
                  "no-correct": "",
                  class: "w-full",
                  size: "medium",
                  modelValue: tokenInput.value,
                  "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((tokenInput).value = $event))
                }, null, 8 /* PROPS */, ["placeholder", "modelValue"])
              ]),
              _createElementVNode("details", { class: "text-sm" }, [
                _createElementVNode("summary", _hoisted_10, _toDisplayString(_ctx.$t('connector.modal.advanced')), 1 /* TEXT */),
                _createElementVNode("div", { class: "mt-3" }, [
                  _createElementVNode("label", _hoisted_11, _toDisplayString(_ctx.$t('connector.modal.port_label')), 1 /* TEXT */),
                  _createVNode(_component_InputBase, {
                    id: "connector-port",
                    type: "text",
                    name: "connector-port",
                    inputmode: "numeric",
                    autocomplete: "off",
                    class: "w-full",
                    size: "medium",
                    modelValue: portInput.value,
                    "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((portInput).value = $event))
                  }, null, 8 /* PROPS */, ["modelValue"]),
                  _hoisted_12,
                  _createElementVNode("div", { class: "flex flex-col gap-2" }, [
                    _createVNode(_component_SettingsToggle, {
                      label: _ctx.$t('connector.modal.auto_open_url'),
                      modelValue: _unref(settings).connector.autoOpenURL,
                      "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((_unref(settings).connector.autoOpenURL) = $event))
                    }, null, 8 /* PROPS */, ["label", "modelValue"])
                  ])
                ])
              ])
            ]),
            _createTextVNode("\n\n      "),
            _createTextVNode("\n      "),
            (_unref(error) && hasAttemptedConnect.value)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                role: "alert",
                class: "p-3 text-sm text-red-400 bg-red-500/10 border border-red-500/20 rounded-md"
              }, _toDisplayString(_unref(error)), 1 /* TEXT */))
              : _createCommentVNode("v-if", true),
            _createTextVNode("\n\n      "),
            _createTextVNode("\n      "),
            _createElementVNode("div", {
              role: "alert",
              class: "p-3 text-sm text-red-400 bg-red-500/10 border border-red-500/20 rounded-md"
            }, [
              _createElementVNode("p", _hoisted_13, _toDisplayString(_ctx.$t('connector.modal.warning')), 1 /* TEXT */),
              _createElementVNode("p", _hoisted_14, _toDisplayString(_ctx.$t('connector.modal.warning_text')), 1 /* TEXT */)
            ]),
            _createVNode(_component_ButtonBase, {
              type: "submit",
              variant: "primary",
              disabled: !tokenInput.value.trim() || _unref(isConnecting),
              class: "w-full"
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(isConnecting) ? _ctx.$t('connector.modal.connecting') : _ctx.$t('connector.modal.connect')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["disabled"])
          ])),
        _createTextVNode("\n\n    "),
        _createTextVNode("\n    ")
      ]),
      _: 1 /* STABLE */
    }, 10 /* CLASS, PROPS */, ["modalTitle"]))
}
}

})
