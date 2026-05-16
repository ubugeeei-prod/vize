import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:check text-green-500 w-6 h-6", "aria-hidden": "true" })
const _hoisted_2 = { class: "font-mono text-sm text-fg" }
const _hoisted_3 = { class: "text-xs text-fg-muted" }
const _hoisted_4 = { class: "text-sm text-fg-muted" }
const _hoisted_5 = { class: "font-mono text-lg text-fg" }
const _hoisted_6 = { class: "font-medium mb-1" }
const _hoisted_7 = { class: "font-medium mb-1" }
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:x text-red-500 w-5 h-5", "aria-hidden": "true" })
const _hoisted_9 = { class: "font-mono text-sm text-fg" }
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:check text-green-500 w-5 h-5", "aria-hidden": "true" })
const _hoisted_11 = { class: "font-mono text-sm text-fg" }
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:x text-red-500 w-5 h-5", "aria-hidden": "true" })
const _hoisted_13 = { class: "font-mono text-sm text-fg" }
const _hoisted_14 = { class: "font-medium mb-1" }
const _hoisted_15 = { class: "text-xs text-yellow-400/80" }
const _hoisted_16 = { class: "text-sm text-fg-muted" }
const _hoisted_17 = { class: "px-3 py-2 text-sm text-fg-muted bg-bg-subtle hover:text-fg transition-colors select-none" }
const _hoisted_18 = { class: "p-3 text-xs font-mono text-fg-muted bg-bg-muted overflow-x-auto" }
const _hoisted_19 = { role: "alert", class: "p-3 text-sm text-red-400 bg-red-500/10 border border-red-500/20 rounded-md" }
import { checkPackageName } from '~/utils/package-name'

export default /*@__PURE__*/_defineComponent({
  __name: 'ClaimPackageModal',
  props: {
    packageName: { type: String, required: true },
    packageScope: { type: String, required: false },
    canPublishToScope: { type: Boolean, required: true }
  },
  setup(__props: any, { expose: __expose }) {

const props = __props
const {
  isConnected,
  state,
  npmUser,
  addOperation,
  approveOperation,
  executeOperations,
  refreshState,
} = useConnector()
const isPublishing = shallowRef(false)
const publishSuccess = shallowRef(false)
const publishError = shallowRef<string | null>(null)
const {
  data: checkResult,
  refresh: checkAvailability,
  status,
  error: checkError,
} = useAsyncData(
  (_nuxtApp, { signal }) => {
    return checkPackageName(props.packageName, { signal })
  },
  { default: () => null, immediate: false },
)
const isChecking = computed(() => {
  return status.value === 'pending'
})
const mergedError = computed(() => {
  return checkResult.value !== null
    ? null
    : (publishError.value ??
        (checkError.value instanceof Error
          ? checkError.value.message
          : $t('claim.modal.failed_to_check')))
})
const connectorModal = useModal('connector-modal')
async function handleClaim() {
  if (!checkResult.value?.available || !isConnected.value) return
  isPublishing.value = true
  publishError.value = null
  try {
    // Add the operation
    const operation = await addOperation({
      type: 'package:init',
      params: { name: props.packageName, ...(npmUser.value && { author: npmUser.value }) },
      description: `Initialize package ${props.packageName}`,
      command: `npm publish (${props.packageName}@0.0.0)`,
    })
    if (!operation) {
      throw new Error('Failed to create operation')
    }
    // Auto-approve and execute
    await approveOperation(operation.id)
    await executeOperations()
    // Refresh state and check if operation completed successfully
    await refreshState()
    // Find the operation and check its status
    const completedOp = state.value.operations.find(op => op.id === operation.id)
    if (completedOp?.status === 'completed') {
      publishSuccess.value = true
    } else if (completedOp?.status === 'failed') {
      if (completedOp.result?.requiresOtp) {
        // OTP is needed - open connector panel to handle it
        close()
        connectorModal.open()
      } else {
        publishError.value = completedOp.result?.stderr || 'Failed to publish package'
      }
    } else {
      // Still pending/approved/running - open connector panel to show progress
      close()
      connectorModal.open()
    }
  } catch (err) {
    publishError.value = err instanceof Error ? err.message : $t('claim.modal.failed_to_claim')
  } finally {
    isPublishing.value = false
  }
}
const dialogRef = useTemplateRef('dialogRef')
function open() {
  // Reset state and check availability each time modal is opened
  publishError.value = null
  publishSuccess.value = false
  checkAvailability()
  dialogRef.value?.showModal()
}
function close() {
  dialogRef.value?.close()
}
// Computed for similar packages with warnings
const hasDangerousSimilarPackages = computed(() => {
  if (!checkResult.value?.similarPackages) return false
  return checkResult.value.similarPackages.some(
    pkg => pkg.similarity === 'exact-match' || pkg.similarity === 'very-similar',
  )
})
const isScoped = computed(() => props.packageName.startsWith('@'))
// Preview of the package.json that will be published
const previewPackageJson = computed(() => {
  const access = isScoped.value ? 'public' : undefined
  return {
    name: props.packageName,
    version: '0.0.0',
    description: `Placeholder for ${props.packageName}`,
    main: 'index.js',
    scripts: {},
    keywords: [],
    author: npmUser.value ? `${npmUser.value} (https://www.npmjs.com/~${npmUser.value})` : '',
    license: 'UNLICENSED',
    private: false,
    ...(access && { publishConfig: { access } }),
  }
})
__expose({ open, close })

return (_ctx: any,_cache: any) => {
  const _component_LoadingSpinner = _resolveComponent("LoadingSpinner")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_Modal = _resolveComponent("Modal")

  return (_openBlock(), _createBlock(_component_Modal, {
      ref_key: "dialogRef", ref: dialogRef,
      modalTitle: _ctx.$t('claim.modal.title'),
      id: "claim-package-modal",
      class: "max-w-md"
    }, {
      default: _withCtx(() => [
        (isChecking.value)
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "py-8 text-center"
          }, [
            _createVNode(_component_LoadingSpinner, { text: _ctx.$t('claim.modal.checking') }, null, 8 /* PROPS */, ["text"])
          ]))
          : (publishSuccess.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 1,
              class: "space-y-4"
            }, [
              _createElementVNode("div", { class: "flex items-center gap-3 p-4 bg-green-500/10 border border-green-500/20 rounded-lg" }, [
                _hoisted_1,
                _createElementVNode("div", null, [
                  _createElementVNode("p", _hoisted_2, _toDisplayString(_ctx.$t('claim.modal.success')), 1 /* TEXT */),
                  _createElementVNode("p", _hoisted_3, _toDisplayString(_ctx.$t('claim.modal.success_detail', { name: __props.packageName })), 1 /* TEXT */)
                ])
              ]),
              _createElementVNode("p", _hoisted_4, _toDisplayString(_ctx.$t('claim.modal.success_hint')), 1 /* TEXT */),
              _createElementVNode("div", { class: "flex gap-3" }, [
                _createVNode(_component_NuxtLink, {
                  to: _ctx.packageRoute(__props.packageName),
                  class: "flex-1 px-4 py-2 font-mono text-sm text-center text-bg bg-fg rounded-md transition-colors duration-200 hover:bg-fg/90 focus-visible:outline-accent/70",
                  onClick: close
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_ctx.$t('claim.modal.view_package')), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["to"]),
                _createElementVNode("button", {
                  type: "button",
                  class: "flex-1 px-4 py-2 font-mono text-sm text-fg-muted bg-bg-subtle border border-border rounded-md transition-colors duration-200 hover:text-fg hover:border-border-hover focus-visible:outline-accent/70",
                  onClick: close
                }, _toDisplayString(_ctx.$t('common.close')), 1 /* TEXT */)
              ])
            ]))
          : (_unref(checkResult))
            ? (_openBlock(), _createElementBlock("div", {
              key: 2,
              class: "space-y-4"
            }, [
              _createElementVNode("div", { class: "p-4 bg-bg-subtle border border-border rounded-lg" }, [
                _createElementVNode("p", _hoisted_5, _toDisplayString(_unref(checkResult).name), 1 /* TEXT */)
              ]),
              _createTextVNode("\n\n      "),
              _createTextVNode("\n      "),
              (_unref(checkResult).validationErrors?.length)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: "p-3 text-sm text-red-400 bg-red-500/10 border border-red-500/20 rounded-md",
                  role: "alert"
                }, [
                  _createElementVNode("p", _hoisted_6, _toDisplayString(_ctx.$t('claim.modal.invalid_name')), 1 /* TEXT */),
                  _createElementVNode("ul", { class: "list-disc list-inside space-y-1" }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(checkResult).validationErrors, (err) => {
                      return (_openBlock(), _createElementBlock("li", { key: err }, _toDisplayString(err), 1 /* TEXT */))
                    }), 128 /* KEYED_FRAGMENT */))
                  ])
                ]))
                : _createCommentVNode("v-if", true),
              _createTextVNode("\n\n      "),
              _createTextVNode("\n      "),
              (_unref(checkResult).validationWarnings?.length)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: "p-3 text-sm text-yellow-400 bg-yellow-500/10 border border-yellow-500/20 rounded-md",
                  role: "alert"
                }, [
                  _createElementVNode("p", _hoisted_7, _toDisplayString(_ctx.$t('common.warnings')), 1 /* TEXT */),
                  _createElementVNode("ul", { class: "list-disc list-inside space-y-1" }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(checkResult).validationWarnings, (warn) => {
                      return (_openBlock(), _createElementBlock("li", { key: warn }, _toDisplayString(warn), 1 /* TEXT */))
                    }), 128 /* KEYED_FRAGMENT */))
                  ])
                ]))
                : _createCommentVNode("v-if", true),
              _createTextVNode("\n\n      "),
              _createTextVNode("\n      "),
              (_unref(checkResult).valid)
                ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                  (_unref(isConnected) && !__props.canPublishToScope)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "flex items-center gap-3 p-4 bg-red-500/10 border border-red-500/20 rounded-lg"
                    }, [
                      _hoisted_8,
                      _createElementVNode("p", _hoisted_9, _toDisplayString(_ctx.$t('claim.modal.missing_permission', { scope: __props.packageScope })), 1 /* TEXT */)
                    ]))
                    : (_unref(checkResult).available)
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 1,
                        class: "flex items-center gap-3 p-4 bg-green-500/10 border border-green-500/20 rounded-lg"
                      }, [
                        _hoisted_10,
                        _createElementVNode("p", _hoisted_11, _toDisplayString(_ctx.$t('claim.modal.available')), 1 /* TEXT */)
                      ]))
                    : (_openBlock(), _createElementBlock("div", {
                      key: 2,
                      class: "flex items-center gap-3 p-4 bg-red-500/10 border border-red-500/20 rounded-lg"
                    }, [
                      _hoisted_12,
                      _createElementVNode("p", _hoisted_13, _toDisplayString(_ctx.$t('claim.modal.taken')), 1 /* TEXT */)
                    ]))
                ], 64 /* STABLE_FRAGMENT */))
                : _createCommentVNode("v-if", true),
              _createTextVNode("\n\n      "),
              _createTextVNode("\n      "),
              (_unref(checkResult).similarPackages?.length && _unref(checkResult).available)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: _normalizeClass(["p-4 border rounded-lg", 
              hasDangerousSimilarPackages.value
                ? 'bg-yellow-500/10 border-yellow-500/20'
                : 'bg-bg-subtle border-border'
            ])
                }, [
                  _createElementVNode("p", {
                    class: _normalizeClass(["text-sm font-medium mb-3", hasDangerousSimilarPackages.value ? 'text-yellow-400' : 'text-fg-muted'])
                  }, [
                    (hasDangerousSimilarPackages.value)
                      ? (_openBlock(), _createElementBlock("span", { key: 0 }, _toDisplayString(_ctx.$t('claim.modal.similar_warning')), 1 /* TEXT */))
                      : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(_ctx.$t('claim.modal.related')), 1 /* TEXT */))
                  ], 2 /* CLASS */),
                  _createElementVNode("ul", { class: "space-y-2" }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(checkResult).similarPackages.slice(0, 5), (pkg) => {
                      return (_openBlock(), _createElementBlock("li", {
                        key: pkg.name,
                        class: "flex items-start gap-2"
                      }, [
                        (pkg.similarity === 'exact-match')
                          ? (_openBlock(), _createElementBlock("span", {
                            key: 0,
                            class: "i-lucide:circle-alert text-red-500 w-4 h-4 mt-0.5 shrink-0",
                            "aria-hidden": "true"
                          }))
                          : (pkg.similarity === 'very-similar')
                            ? (_openBlock(), _createElementBlock("span", {
                              key: 1,
                              class: "i-lucide:circle-alert text-yellow-500 w-4 h-4 mt-0.5 shrink-0",
                              "aria-hidden": "true"
                            }))
                          : (_openBlock(), _createElementBlock("span", {
                            key: 2,
                            class: "w-4 h-4 shrink-0"
                          })),
                        _createElementVNode("div", { class: "min-w-0" }, [
                          _createVNode(_component_NuxtLink, {
                            to: _ctx.packageRoute(pkg.name),
                            class: "font-mono text-sm text-fg hover:underline focus-visible:outline-accent/70 rounded",
                            target: "_blank"
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(pkg.name), 1 /* TEXT */)
                            ]),
                            _: 2 /* DYNAMIC */
                          }, 8 /* PROPS */, ["to"]),
                          (pkg.description)
                            ? (_openBlock(), _createElementBlock("p", {
                              key: 0,
                              class: "text-xs text-fg-subtle truncate"
                            }, _toDisplayString(pkg.description), 1 /* TEXT */))
                            : _createCommentVNode("v-if", true)
                        ])
                      ]))
                    }), 128 /* KEYED_FRAGMENT */))
                  ])
                ]))
                : _createCommentVNode("v-if", true),
              _createTextVNode("\n\n      "),
              _createTextVNode("\n      "),
              (mergedError.value)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  role: "alert",
                  class: "p-3 text-sm text-red-400 bg-red-500/10 border border-red-500/20 rounded-md"
                }, _toDisplayString(mergedError.value), 1 /* TEXT */))
                : _createCommentVNode("v-if", true),
              _createTextVNode("\n\n      "),
              _createTextVNode("\n      "),
              (_unref(checkResult).available && _unref(checkResult).valid)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: "space-y-3"
                }, [
                  (!isScoped.value)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "p-3 text-sm text-yellow-400 bg-yellow-500/10 border border-yellow-500/20 rounded-md"
                    }, [
                      _createElementVNode("p", _hoisted_14, _toDisplayString(_ctx.$t('claim.modal.scope_warning_title')), 1 /* TEXT */),
                      _createElementVNode("p", _hoisted_15, _toDisplayString(_ctx.$t('claim.modal.scope_warning_text', {
                  username: _unref(npmUser) || 'username',
                  name: __props.packageName,
                })), 1 /* TEXT */)
                    ]))
                    : _createCommentVNode("v-if", true),
                  _createTextVNode("\n\n        "),
                  _createTextVNode("\n        "),
                  (!_unref(isConnected))
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "space-y-3"
                    }, [
                      _createElementVNode("div", { class: "p-3 text-sm text-yellow-400 bg-yellow-500/10 border border-yellow-500/20 rounded-md" }, [
                        _createElementVNode("p", null, _toDisplayString(_ctx.$t('claim.modal.connect_required')), 1 /* TEXT */)
                      ]),
                      _createElementVNode("button", {
                        type: "button",
                        class: "w-full px-4 py-2 font-mono text-sm text-bg bg-fg rounded-md transition-colors duration-200 hover:bg-fg/90 focus-visible:outline-accent/70",
                        onClick: _cache[0] || (_cache[0] = ($event: any) => (_unref(connectorModal).open))
                      }, _toDisplayString(_ctx.$t('claim.modal.connect_button')), 1 /* TEXT */)
                    ]))
                    : (_unref(isConnected) && __props.canPublishToScope)
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 1,
                        class: "space-y-3"
                      }, [
                        _createElementVNode("p", _hoisted_16, _toDisplayString(_ctx.$t('claim.modal.publish_hint')), 1 /* TEXT */),
                        _createTextVNode("\n\n          "),
                        _createTextVNode("\n          "),
                        _createElementVNode("details", { class: "border border-border rounded-md overflow-hidden" }, [
                          _createElementVNode("summary", _hoisted_17, _toDisplayString(_ctx.$t('claim.modal.preview_json')), 1 /* TEXT */),
                          _createElementVNode("pre", _hoisted_18, _toDisplayString(JSON.stringify(previewPackageJson.value, null, 2)), 1 /* TEXT */)
                        ]),
                        _createElementVNode("button", {
                          type: "button",
                          disabled: isPublishing.value,
                          class: "w-full px-4 py-2 font-mono text-sm text-bg bg-fg rounded-md transition-colors duration-200 hover:bg-fg/90 disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline-accent/70",
                          onClick: handleClaim
                        }, _toDisplayString(isPublishing.value ? _ctx.$t('claim.modal.publishing') : _ctx.$t('claim.modal.claim_button')), 9 /* TEXT, PROPS */, ["disabled"])
                      ]))
                    : _createCommentVNode("v-if", true),
                  _createTextVNode("\n\n        "),
                  _createTextVNode("\n        ")
                ]))
                : _createCommentVNode("v-if", true),
              _createTextVNode("\n\n      "),
              _createTextVNode("\n      "),
              (!_unref(checkResult).available || !_unref(checkResult).valid)
                ? (_openBlock(), _createElementBlock("button", {
                  key: 0,
                  type: "button",
                  class: "w-full px-4 py-2 font-mono text-sm text-fg-muted bg-bg-subtle border border-border rounded-md transition-colors duration-200 hover:text-fg hover:border-border-hover focus-visible:outline-accent/70",
                  onClick: close
                }, _toDisplayString(_ctx.$t('common.close')), 1 /* TEXT */))
                : _createCommentVNode("v-if", true)
            ]))
          : (mergedError.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 3,
              class: "space-y-4"
            }, [
              _createElementVNode("div", _hoisted_19, _toDisplayString(mergedError.value), 1 /* TEXT */),
              _createElementVNode("button", {
                type: "button",
                class: "w-full px-4 py-2 font-mono text-sm text-fg-muted bg-bg-subtle border border-border rounded-md transition-colors duration-200 hover:text-fg hover:border-border-hover focus-visible:outline-accent/70",
                onClick: _cache[1] || (_cache[1] = () => _unref(checkAvailability)())
              }, _toDisplayString(_ctx.$t('common.retry')), 1 /* TEXT */)
            ]))
          : _createCommentVNode("v-if", true),
        _createTextVNode("\n\n    "),
        _createTextVNode("\n    "),
        _createTextVNode("\n\n    "),
        _createTextVNode("\n    "),
        _createTextVNode("\n\n    "),
        _createTextVNode("\n    ")
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["modalTitle"]))
}
}

})
