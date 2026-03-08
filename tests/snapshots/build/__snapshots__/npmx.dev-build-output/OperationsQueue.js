import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:refresh-ccw w-4 h-4", "aria-hidden": "true" })
const _hoisted_2 = { class: "font-mono text-sm text-fg-subtle" }
const _hoisted_3 = { class: "font-mono text-xs text-fg-subtle mt-1" }
const _hoisted_4 = { class: "font-mono text-sm text-fg truncate" }
const _hoisted_5 = { class: "font-mono text-xs text-fg-subtle mt-0.5 truncate" }
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:check w-4 h-4", "aria-hidden": "true" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:x w-4 h-4", "aria-hidden": "true" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:lock w-4 h-4 text-amber-400 shrink-0", "aria-hidden": "true" })
const _hoisted_9 = { class: "font-mono text-sm text-amber-400" }
const _hoisted_10 = { for: "otp-input", class: "sr-only" }
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("div", { class: "flex-1 h-px bg-amber-500/30" })
const _hoisted_12 = { class: "text-xs text-amber-400 font-mono uppercase" }
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("div", { class: "flex-1 h-px bg-amber-500/30" })
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:external-link w-4 h-4 inline-block me-1", "aria-hidden": "true" })
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:chevron-right rtl-flip w-3 h-3 transition-transform duration-200 [[open]>&]:rotate-90", "aria-hidden": "true" })
const _hoisted_16 = { class: "truncate block" }
const _hoisted_17 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:x w-3 h-3", "aria-hidden": "true" })
import type { PendingOperation } from '~~/cli/src/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'OperationsQueue',
  setup(__props) {

const {
  isConnected,
  pendingOperations,
  approvedOperations,
  completedOperations,
  activeOperations,
  operations,
  hasOperations,
  hasPendingOperations,
  hasApprovedOperations,
  hasActiveOperations,
  hasCompletedOperations,
  removeOperation,
  clearOperations,
  approveOperation,
  approveAll,
  executeOperations,
  retryOperation,
  refreshState,
} = useConnector()
const isExecuting = shallowRef(false)
const otpInput = shallowRef('')
const otpError = shallowRef('')
const authUrl = computed(() => {
  const op = operations.value.find(o => o.status === 'running' && o.authUrl)
  return op?.authUrl ?? null
})
const authPollTimer = shallowRef<ReturnType<typeof setInterval> | null>(null)
function startAuthPolling() {
  stopAuthPolling()
  let remaining = 3
  authPollTimer.value = setInterval(async () => {
    try {
      await refreshState()
    } catch {
      stopAuthPolling()
      return
    }
    remaining--
    if (remaining <= 0) {
      stopAuthPolling()
    }
  }, 20000)
}
function stopAuthPolling() {
  if (authPollTimer.value) {
    clearInterval(authPollTimer.value)
    authPollTimer.value = null
  }
}
onUnmounted(stopAuthPolling)
function handleOpenAuthUrl() {
  if (authUrl.value) {
    window.open(authUrl.value, '_blank', 'noopener,noreferrer')
    startAuthPolling()
  }
}
/** Check if any active operation needs OTP (fallback for web auth failures) */
const hasOtpFailures = computed(() =>
  activeOperations.value.some(
    (op: PendingOperation) =>
      op.status === 'failed' && (op.result?.requiresOtp || op.result?.authFailure),
  ),
)
async function handleApproveAll() {
  await approveAll()
}
async function handleExecute(otp?: string) {
  isExecuting.value = true
  try {
    await executeOperations(otp)
  } finally {
    isExecuting.value = false
  }
}
/** Retry all OTP-failed operations with the provided OTP */
async function handleRetryWithOtp() {
  const otp = otpInput.value.trim()
  if (!otp) {
    otpError.value = 'OTP required'
    return
  }
  if (!/^\d{6}$/.test(otp)) {
    otpError.value = 'OTP must be a 6-digit code'
    return
  }
  otpError.value = ''
  otpInput.value = ''
  // First, re-approve all OTP/auth-failed operations
  const otpFailedOps = activeOperations.value.filter(
    (op: PendingOperation) =>
      op.status === 'failed' && (op.result?.requiresOtp || op.result?.authFailure),
  )
  for (const op of otpFailedOps) {
    await retryOperation(op.id)
  }
  // Then execute with OTP
  await handleExecute(otp)
}
/** Retry failed operations with web auth (no OTP) */
async function handleRetryWebAuth() {
  // Find all failed operations that need auth retry
  const failedOps = activeOperations.value.filter(
    (op: PendingOperation) =>
      op.status === 'failed' && (op.result?.requiresOtp || op.result?.authFailure),
  )
  for (const op of failedOps) {
    await retryOperation(op.id)
  }
  await handleExecute()
}
async function handleClearAll() {
  await clearOperations()
  otpInput.value = ''
  otpError.value = ''
}
function getStatusColor(status: string): string {
  switch (status) {
    case 'pending':
      return 'bg-yellow-500'
    case 'approved':
      return 'bg-blue-500'
    case 'running':
      return 'bg-purple-500'
    case 'completed':
      return 'bg-green-500'
    case 'failed':
      return 'bg-red-500'
    default:
      return 'bg-fg-subtle'
  }
}
function getStatusIcon(status: string): string {
  switch (status) {
    case 'pending':
      return 'i-lucide:clock'
    case 'approved':
      return 'i-lucide:check'
    case 'running':
      return 'i-svg-spinners:ring-resize'
    case 'completed':
      return 'i-lucide:check'
    case 'failed':
      return 'i-lucide:x'
    default:
      return 'i-lucide:circle-question-mark'
  }
}
// Auto-refresh while executing
const { pause: pauseRefresh, resume: resumeRefresh } = useIntervalFn(() => refreshState(), 1000, {
  immediate: false,
})
watch(isExecuting, executing => {
  if (executing) {
    resumeRefresh()
  } else {
    pauseRefresh()
  }
})

return (_ctx: any,_cache: any) => {
  const _component_InputBase = _resolveComponent("InputBase")

  return (_unref(isConnected))
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        class: "space-y-4"
      }, [ _createElementVNode("div", { class: "flex items-center justify-between" }, [ _createElementVNode("h3", { class: "font-mono text-sm font-medium text-fg" }, [ _createTextVNode(_toDisplayString(_ctx.$t('operations.queue.title')) + "\n        ", 1 /* TEXT */), (_unref(hasActiveOperations)) ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                class: "text-fg-muted"
              }, "(" + _toDisplayString(_unref(activeOperations).length) + ")", 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", { class: "flex items-center gap-2" }, [ (_unref(hasOperations)) ? (_openBlock(), _createElementBlock("button", {
                key: 0,
                type: "button",
                class: "px-2 py-1 font-mono text-xs text-fg-muted hover:text-fg bg-bg-subtle border border-border rounded transition-colors duration-200 hover:border-border-hover focus-visible:outline-accent/70",
                "aria-label": _ctx.$t('operations.queue.clear_all'),
                onClick: handleClearAll
              }, _toDisplayString(_ctx.$t('operations.queue.clear_all')), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createElementVNode("button", {
              type: "button",
              class: "p-1 text-fg-muted hover:text-fg transition-colors duration-200 rounded focus-visible:outline-accent/70",
              "aria-label": _ctx.$t('operations.queue.refresh'),
              onClick: _cache[0] || (_cache[0] = (...args) => (refreshState && refreshState(...args)))
            }, [ _hoisted_1 ], 8 /* PROPS */, ["aria-label"]) ]) ]), _createTextVNode("\n\n    "), _createTextVNode("\n    "), (!_unref(hasActiveOperations) && !_unref(hasCompletedOperations)) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "py-8 text-center"
          }, [ _createElementVNode("p", _hoisted_2, _toDisplayString(_ctx.$t('operations.queue.empty')), 1 /* TEXT */), _createElementVNode("p", _hoisted_3, _toDisplayString(_ctx.$t('operations.queue.empty_hint')), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    "), _createTextVNode("\n    "), (_unref(hasActiveOperations)) ? (_openBlock(), _createElementBlock("ul", {
            key: 0,
            class: "space-y-2",
            "aria-label": _ctx.$t('operations.queue.active_label')
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(activeOperations), (op) => {
              return (_openBlock(), _createElementBlock("li", {
                key: op.id,
                class: "flex items-start gap-3 p-3 bg-bg-subtle border border-border rounded-lg"
              }, [
                _createElementVNode("span", {
                  class: "flex-shrink-0 w-5 h-5 flex items-center justify-center",
                  "aria-label": op.status
                }, [
                  _createElementVNode("span", {
                    class: _normalizeClass(["w-4 h-4", [getStatusIcon(op.status), getStatusColor(op.status).replace('bg-', 'text-')]]),
                    "aria-hidden": "true"
                  }, null, 2 /* CLASS */)
                ], 8 /* PROPS */, ["aria-label"]),
                _createTextVNode("\n\n        " + "\n        "),
                _createElementVNode("div", { class: "flex-1 min-w-0" }, [
                  _createElementVNode("p", _hoisted_4, _toDisplayString(op.description), 1 /* TEXT */),
                  _createElementVNode("p", _hoisted_5, _toDisplayString(op.command), 1 /* TEXT */),
                  _createTextVNode("\n          " + "\n          "),
                  (op.result?.requiresOtp && op.status === 'failed')
                    ? (_openBlock(), _createElementBlock("p", {
                      key: 0,
                      class: "mt-1 text-xs text-amber-400"
                    }, _toDisplayString(_ctx.$t('operations.queue.otp_required')), 1 /* TEXT */))
                    : (op.result && (op.status === 'completed' || op.status === 'failed'))
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 1,
                        class: "mt-2 p-2 bg-bg-muted border border-border rounded text-xs font-mono"
                      }, [
                        (op.result.stdout)
                          ? (_openBlock(), _createElementBlock("pre", {
                            key: 0,
                            class: "text-fg-muted whitespace-pre-wrap"
                          }, _toDisplayString(op.result.stdout), 1 /* TEXT */))
                          : _createCommentVNode("v-if", true),
                        (op.result.stderr)
                          ? (_openBlock(), _createElementBlock("pre", {
                            key: 0,
                            class: "text-red-400 whitespace-pre-wrap"
                          }, _toDisplayString(op.result.stderr), 1 /* TEXT */))
                          : _createCommentVNode("v-if", true)
                      ]))
                    : _createCommentVNode("v-if", true),
                  _createTextVNode("\n          " + "\n          ")
                ]),
                _createTextVNode("\n\n        " + "\n        "),
                _createElementVNode("div", { class: "flex-shrink-0 flex items-center gap-1" }, [
                  (op.status === 'pending')
                    ? (_openBlock(), _createElementBlock("button", {
                      key: 0,
                      type: "button",
                      class: "p-1 text-fg-muted hover:text-green-400 transition-colors duration-200 rounded focus-visible:outline-accent/70",
                      "aria-label": _ctx.$t('operations.queue.approve_operation'),
                      onClick: _cache[1] || (_cache[1] = ($event: any) => (_unref(approveOperation)(op.id)))
                    }, [
                      _hoisted_6
                    ]))
                    : _createCommentVNode("v-if", true),
                  (op.status !== 'running')
                    ? (_openBlock(), _createElementBlock("button", {
                      key: 0,
                      type: "button",
                      class: "p-1 text-fg-muted hover:text-red-400 transition-colors duration-200 rounded focus-visible:outline-accent/70",
                      "aria-label": _ctx.$t('operations.queue.remove_operation'),
                      onClick: _cache[2] || (_cache[2] = ($event: any) => (_unref(removeOperation)(op.id)))
                    }, [
                      _hoisted_7
                    ]))
                    : _createCommentVNode("v-if", true)
                ])
              ]))
            }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    "), _createTextVNode("\n    "), (hasOtpFailures.value) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "p-3 bg-amber-500/10 border border-amber-500/30 rounded-lg",
            role: "alert"
          }, [ _createElementVNode("div", { class: "flex items-center gap-2 mb-2" }, [ _hoisted_8, _createElementVNode("span", _hoisted_9, _toDisplayString(_ctx.$t('operations.queue.otp_prompt')), 1 /* TEXT */) ]), _createElementVNode("form", {
              class: "flex flex-col gap-1",
              onSubmit: _withModifiers(handleRetryWithOtp, ["prevent"])
            }, [ _createElementVNode("div", { class: "flex items-center gap-2" }, [ _createElementVNode("label", _hoisted_10, _toDisplayString(_ctx.$t('operations.queue.otp_label')), 1 /* TEXT */), _createVNode(_component_InputBase, {
                  id: "otp-input",
                  type: "text",
                  name: "otp-code",
                  inputmode: "numeric",
                  pattern: "[0-9]*",
                  maxlength: "6",
                  placeholder: _ctx.$t('operations.queue.otp_placeholder'),
                  autocomplete: "one-time-code",
                  spellcheck: "false",
                  class: _normalizeClass(['flex-1 min-w-25', otpError.value ? 'border-red-500 focus:outline-red-500' : '']),
                  size: "small",
                  onInput: _cache[3] || (_cache[3] = ($event: any) => (otpError.value = '')),
                  modelValue: otpInput.value,
                  "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((otpInput).value = $event))
                }, null, 10 /* CLASS, PROPS */, ["placeholder", "modelValue"]), _createElementVNode("button", {
                  type: "submit",
                  disabled: isExecuting.value,
                  class: "px-3 py-2 font-mono text-xs text-bg bg-amber-500 rounded transition-all duration-200 hover:bg-amber-400 disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-amber-500/50"
                }, _toDisplayString(isExecuting.value ? _ctx.$t('operations.queue.retrying') : _ctx.$t('operations.queue.retry_otp')), 9 /* TEXT, PROPS */, ["disabled"]) ]), (otpError.value) ? (_openBlock(), _createElementBlock("p", {
                  key: 0,
                  class: "text-xs text-red-400 font-mono"
                }, _toDisplayString(otpError.value), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 32 /* NEED_HYDRATION */), _createElementVNode("div", { class: "flex items-center gap-2 my-3" }, [ _hoisted_11, _createElementVNode("span", _hoisted_12, _toDisplayString(_ctx.$t('common.or')), 1 /* TEXT */), _hoisted_13 ]), _createElementVNode("button", {
              type: "button",
              disabled: isExecuting.value,
              class: "w-full px-3 py-2 font-mono text-xs text-fg bg-bg-subtle border border-border rounded transition-all duration-200 hover:text-fg hover:border-border-hover disabled:opacity-50 disabled:cursor-not-allowed",
              onClick: handleRetryWebAuth
            }, _toDisplayString(isExecuting.value ? _ctx.$t('operations.queue.retrying') : _ctx.$t('operations.queue.retry_web_auth')), 9 /* TEXT, PROPS */, ["disabled"]) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    "), _createTextVNode("\n    "), (_unref(hasActiveOperations)) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "flex items-center gap-2 pt-2"
          }, [ (_unref(hasPendingOperations)) ? (_openBlock(), _createElementBlock("button", {
                key: 0,
                type: "button",
                class: "flex-1 px-4 py-2 font-mono text-sm text-fg bg-bg-subtle border border-border rounded-md transition-colors duration-200 hover:border-border-hover focus-visible:outline-accent/70",
                onClick: handleApproveAll
              }, _toDisplayString(_ctx.$t('operations.queue.approve_all')) + " (" + _toDisplayString(_unref(pendingOperations).length) + ")\n      ", 1 /* TEXT */)) : _createCommentVNode("v-if", true), (_unref(hasApprovedOperations) && !hasOtpFailures.value) ? (_openBlock(), _createElementBlock("button", {
                key: 0,
                type: "button",
                disabled: isExecuting.value,
                class: "flex-1 px-4 py-2 font-mono text-sm text-bg bg-fg rounded-md transition-all duration-200 hover:bg-fg/90 disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline-accent/70 focus-visible:ring-offset-2 focus-visible:ring-offset-bg",
                onClick: _cache[5] || (_cache[5] = ($event: any) => (handleExecute()))
              }, _toDisplayString(isExecuting.value ? _ctx.$t('operations.queue.executing') : `${_ctx.$t('operations.queue.execute')} (${_unref(approvedOperations).length})`), 1 /* TEXT */)) : _createCommentVNode("v-if", true), (authUrl.value) ? (_openBlock(), _createElementBlock("button", {
                key: 0,
                type: "button",
                class: "flex-1 px-4 py-2 font-mono text-sm text-accent bg-accent/10 border border-accent/30 rounded-md transition-colors duration-200 hover:bg-accent/20",
                onClick: handleOpenAuthUrl
              }, [ _hoisted_14, _createTextVNode("\n        "), _toDisplayString(_ctx.$t('operations.queue.open_web_auth')) ])) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    "), _createTextVNode("\n    "), (_unref(hasCompletedOperations)) ? (_openBlock(), _createElementBlock("details", {
            key: 0,
            class: "mt-4 border-t border-border pt-4"
          }, [ _createElementVNode("summary", { class: "flex items-center gap-2 font-mono text-xs text-fg-muted hover:text-fg transition-colors duration-200 select-none" }, [ _hoisted_15, _createTextVNode("\n        " + _toDisplayString(_ctx.$t('operations.queue.log')) + " (" + _toDisplayString(_unref(completedOperations).length) + ")\n      ", 1 /* TEXT */) ]), _createElementVNode("ul", {
              class: "mt-2 space-y-1",
              "aria-label": _ctx.$t('operations.queue.log_label')
            }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(completedOperations), (op) => {
                return (_openBlock(), _createElementBlock("li", {
                  key: op.id,
                  class: _normalizeClass(["flex items-start gap-2 p-2 text-xs font-mono rounded", op.status === 'completed' ? 'text-fg-muted' : 'text-red-400/80'])
                }, [
                  _createElementVNode("span", {
                    class: _normalizeClass(["w-3.5 h-3.5 shrink-0 mt-0.5", 
                op.status === 'completed'
                  ? 'i-lucide:check text-green-500'
                  : 'i-lucide:x text-red-500'
              ]),
                    "aria-hidden": "true"
                  }, null, 2 /* CLASS */),
                  _createElementVNode("div", { class: "flex-1 min-w-0" }, [
                    _createElementVNode("span", _hoisted_16, _toDisplayString(op.description), 1 /* TEXT */),
                    _createTextVNode("\n            " + "\n            "),
                    (op.status === 'failed' && op.result?.stderr)
                      ? (_openBlock(), _createElementBlock("pre", {
                        key: 0,
                        class: "mt-1 text-red-400/70 whitespace-pre-wrap text-2xs"
                      }, _toDisplayString(op.result.stderr), 1 /* TEXT */))
                      : _createCommentVNode("v-if", true)
                  ]),
                  _createElementVNode("button", {
                    type: "button",
                    class: "p-0.5 text-fg-subtle hover:text-fg-muted transition-colors duration-200 rounded focus-visible:outline-accent/70",
                    "aria-label": _ctx.$t('operations.queue.remove_from_log'),
                    onClick: _cache[6] || (_cache[6] = ($event: any) => (_unref(removeOperation)(op.id)))
                  }, [
                    _hoisted_17
                  ], 8 /* PROPS */, ["aria-label"])
                ], 2 /* CLASS */))
              }), 128 /* KEYED_FRAGMENT */)) ], 8 /* PROPS */, ["aria-label"]) ])) : _createCommentVNode("v-if", true) ]))
      : _createCommentVNode("v-if", true)
}
}

})
