import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref, vModelText as _vModelText, vModelCheckbox as _vModelCheckbox, vModelRadio as _vModelRadio } from "vue"


const _hoisted_1 = { "text-primary": "true" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:close-line": "true" })
const _hoisted_3 = { mxa: "true", "text-4xl": "true", mb4: "true" }
const _hoisted_4 = { "text-xl": "true" }
const _hoisted_5 = { "pl-2": "true", for: "dontlike", "font-bold": "true" }
const _hoisted_6 = { "pl-6": "true" }
const _hoisted_7 = { "pl-2": "true", for: "spam", "font-bold": "true" }
const _hoisted_8 = { "pl-6": "true" }
const _hoisted_9 = { "pl-2": "true", for: "violation", "font-bold": "true" }
const _hoisted_10 = { "pl-2": "true", for: "other", "font-bold": "true" }
const _hoisted_11 = { "pl-6": "true" }
const _hoisted_12 = { "mt-8": "true", "mb-4": "true", "font-bold": "true" }
const _hoisted_13 = { "mt-8": "true", "mb-2": "true", "font-bold": "true" }
const _hoisted_14 = { "pb-1": "true" }
const _hoisted_15 = { mxa: "true", "text-4xl": "true", mb4: "true" }
const _hoisted_16 = { "text-primary": "true", "font-bold": "true" }
const _hoisted_17 = { mxa: "true", "text-4xl": "true", mb4: "true" }
const _hoisted_18 = { "text-xl": "true" }
const _hoisted_19 = /*#__PURE__*/ _createElementVNode("br")
const _hoisted_20 = /*#__PURE__*/ _createElementVNode("br")
const _hoisted_21 = /*#__PURE__*/ _createElementVNode("br")
import type { mastodon } from 'masto'
import { toggleBlockAccount, toggleFollowAccount, toggleMuteAccount, useRelationship } from '~/composables/masto/relationship'

export default /*@__PURE__*/_defineComponent({
  __name: 'ReportModal',
  props: {
    account: { type: null, required: true },
    status: { type: null, required: false }
  },
  emits: ["close"],
  async setup(__props: any, { emit: __emit }) {

let __temp: any, __restore: any

const emit = __emit
const { client } = useMasto()
const step = ref('selectCategory')
const serverRules = ref((await client.value.v2.instance.fetch()).rules || [])
const reportReason = ref('')
const selectedRuleIds = ref([])
const availableStatuses = ref(__props.status ? [__props.status] : [])
const selectedStatusIds = ref(__props.status ? [__props.status.id] : [])
const additionalComments = ref('')
const forwardReport = ref(false)
const dismissButton = ref<HTMLDivElement>()
loadStatuses() // Load statuses asynchronously ahead of time
function categoryChosen() {
  step.value = reportReason.value === 'dontlike' ? 'furtherActions' : 'selectStatuses'
  resetModal()
}
async function loadStatuses() {
  if (__props.status) {
    // Load the 5 statuses before and after the reported status
    const prevStatuses = await client.value.v1.accounts.$select(__props.account.id).statuses.list({
      maxId: __props.status.id,
      limit: 5,
    })
    const nextStatuses = await client.value.v1.accounts.$select(__props.account.id).statuses.list({
      minId: __props.status.id,
      limit: 5,
    })
    availableStatuses.value = availableStatuses.value.concat(prevStatuses)
    availableStatuses.value = availableStatuses.value.concat(nextStatuses)
  }
  else {
    // Reporting an account directly
    // Load the 10 most recent statuses
    const mostRecentStatuses = await client.value.v1.accounts.$select(__props.account.id).statuses.list({
      limit: 10,
    })
    availableStatuses.value = mostRecentStatuses
  }
  availableStatuses.value.sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime())
}
async function submitReport() {
;(
  ([__temp,__restore] = _withAsyncContext(() => client.value.v1.reports.create({)),
  await __temp,
  __restore()
)
    accountId: __props.account.id,
    statusIds: selectedStatusIds.value,
    comment: additionalComments.value,
    forward: forwardReport.value,
    category: reportReason.value === 'spam' ? 'spam' : reportReason.value === 'violation' ? 'violation' : 'other',
    ruleIds: reportReason.value === 'violation' ? selectedRuleIds.value : null,
  })
  step.value = 'furtherActions'
  resetModal()
}
function unfollow() {
  emit('close')
  toggleFollowAccount(useRelationship(__props.account).value!, __props.account)
}
function mute() {
  emit('close')
  toggleMuteAccount(useRelationship(__props.account).value!, __props.account)
}
function block() {
  emit('close')
  toggleBlockAccount(useRelationship(__props.account).value!, __props.account)
}
function resetModal() {
  // TODO: extract this scroll/reset logic into ModalDialog element
  dismissButton.value?.scrollIntoView() // scroll to top
}

return (_ctx: any,_cache: any) => {
  const _component_i18n_t = _resolveComponent("i18n-t")
  const _component_StatusCard = _resolveComponent("StatusCard")

  return (_openBlock(), _createElementBlock("div", {
      "my-8": "",
      "px-3": "",
      "sm:px-8": "",
      flex: "~ col gap-4",
      relative: ""
    }, [ _createElementVNode("h2", {
        mxa: "",
        "text-xl": ""
      }, [ _createVNode(_component_i18n_t, { keypath: reportReason.value === 'dontlike' ? 'report.limiting' : 'report.reporting' }, {
          default: _withCtx(() => [
            _createElementVNode("b", _hoisted_1, "@" + _toDisplayString(__props.account.acct), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["keypath"]) ]), _createElementVNode("button", {
        ref_key: "dismissButton", ref: dismissButton,
        "btn-action-icon": "",
        absolute: "",
        "top--8": "",
        "right-0": "",
        m1: "",
        "aria-label": _ctx.$t('action.close'),
        onClick: _cache[0] || (_cache[0] = ($event: any) => (emit('close')))
      }, [ _hoisted_2 ], 8 /* PROPS */, ["aria-label"]), (step.value === 'selectCategory') ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createElementVNode("h1", _hoisted_3, _toDisplayString(__props.status ? _ctx.$t('report.whats_wrong_post') : _ctx.$t('report.whats_wrong_account')), 1 /* TEXT */), _createElementVNode("p", _hoisted_4, _toDisplayString(_ctx.$t('report.select_one')), 1 /* TEXT */), _createElementVNode("div", null, [ _withDirectives(_createElementVNode("input", {
              id: "dontlike",
              "onUpdate:modelValue": [($event: any) => ((reportReason).value = $event), ($event: any) => ((reportReason).value = $event)],
              type: "radio",
              value: "dontlike"
            }, null, 512 /* NEED_PATCH */), [ [_vModelRadio, reportReason.value] ]), _createElementVNode("label", _hoisted_5, _toDisplayString(_ctx.$t('report.dontlike')), 1 /* TEXT */), _createElementVNode("p", _hoisted_6, _toDisplayString(_ctx.$t('report.dontlike_desc')), 1 /* TEXT */) ]), _createElementVNode("div", null, [ _withDirectives(_createElementVNode("input", {
              id: "spam",
              "onUpdate:modelValue": [($event: any) => ((reportReason).value = $event), ($event: any) => ((reportReason).value = $event)],
              type: "radio",
              value: "spam"
            }, null, 512 /* NEED_PATCH */), [ [_vModelRadio, reportReason.value] ]), _createElementVNode("label", _hoisted_7, _toDisplayString(_ctx.$t('report.spam')), 1 /* TEXT */), _createElementVNode("p", _hoisted_8, _toDisplayString(_ctx.$t('report.spam_desc')), 1 /* TEXT */) ]), (serverRules.value.length > 0) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _withDirectives(_createElementVNode("input", {
                id: "violation",
                "onUpdate:modelValue": [($event: any) => ((reportReason).value = $event), ($event: any) => ((reportReason).value = $event)],
                type: "radio",
                value: "violation"
              }, null, 512 /* NEED_PATCH */), [ [_vModelRadio, reportReason.value] ]), _createElementVNode("label", _hoisted_9, _toDisplayString(_ctx.$t('report.violation')), 1 /* TEXT */), (reportReason.value === 'violation') ? (_openBlock(), _createElementBlock("p", {
                  key: 0,
                  "pl-6": "",
                  "pt-2": "",
                  "text-primary": "",
                  "font-bold": ""
                }, _toDisplayString(_ctx.$t('report.select_many')), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createElementVNode("ul", { "pl-6": "" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(serverRules.value, (rule) => {
                  return (_openBlock(), _createElementBlock("li", {
                    key: rule.id,
                    "pt-2": ""
                  }, [
                    _withDirectives(_createElementVNode("input", {
                      id: rule.id,
                      "onUpdate:modelValue": [($event: any) => ((selectedRuleIds).value = $event), ($event: any) => ((selectedRuleIds).value = $event)],
                      type: "checkbox",
                      value: rule.id,
                      disabled: reportReason.value !== 'violation'
                    }, null, 8 /* PROPS */, ["id", "value", "disabled"]), [
                      [_vModelCheckbox, selectedRuleIds.value]
                    ]),
                    _createElementVNode("label", {
                      "pl-2": "",
                      for: rule.id
                    }, _toDisplayString(rule.text), 9 /* TEXT, PROPS */, ["for"])
                  ]))
                }), 128 /* KEYED_FRAGMENT */)) ]) ])) : _createCommentVNode("v-if", true), _createElementVNode("div", null, [ _withDirectives(_createElementVNode("input", {
              id: "other",
              "onUpdate:modelValue": [($event: any) => ((reportReason).value = $event), ($event: any) => ((reportReason).value = $event)],
              type: "radio",
              value: "other"
            }, null, 512 /* NEED_PATCH */), [ [_vModelRadio, reportReason.value] ]), _createElementVNode("label", _hoisted_10, _toDisplayString(_ctx.$t('report.other')), 1 /* TEXT */), _createElementVNode("p", _hoisted_11, _toDisplayString(_ctx.$t('report.other_desc')), 1 /* TEXT */) ]), (reportReason.value && reportReason.value !== 'dontlike') ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createElementVNode("h3", _hoisted_12, _toDisplayString(_ctx.$t('report.anything_else')), 1 /* TEXT */), _withDirectives(_createElementVNode("textarea", {
                "onUpdate:modelValue": [($event: any) => ((additionalComments).value = $event), ($event: any) => ((additionalComments).value = $event)],
                "w-full": "",
                "h-20": "",
                "p-3": "",
                border: "",
                placeholder: _ctx.$t('report.additional_comments')
              }, null, 8 /* PROPS */, ["placeholder"]), [ [_vModelText, additionalComments.value] ]), (_ctx.getServerName(__props.account) && _ctx.getServerName(__props.account) !== _ctx.currentServer) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createElementVNode("h3", _hoisted_13, _toDisplayString(_ctx.$t('report.another_server')), 1 /* TEXT */), _createElementVNode("p", _hoisted_14, _toDisplayString(_ctx.$t('report.forward_question')), 1 /* TEXT */), _withDirectives(_createElementVNode("input", {
                    id: "forward",
                    "onUpdate:modelValue": [($event: any) => ((forwardReport).value = $event), ($event: any) => ((forwardReport).value = $event)],
                    type: "checkbox",
                    value: "rule.id"
                  }, null, 512 /* NEED_PATCH */), [ [_vModelCheckbox, forwardReport.value] ]), _createElementVNode("label", {
                    "pl-2": "",
                    for: "forward"
                  }, [ _createElementVNode("b", null, _toDisplayString(_ctx.$t('report.forward', [_ctx.getServerName(__props.account)])), 1 /* TEXT */) ]) ])) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true), _createElementVNode("button", {
            "btn-solid": "",
            mxa: "",
            "mt-10": "",
            disabled: !reportReason.value || (reportReason.value === 'violation' && selectedRuleIds.value.length < 1),
            onClick: _cache[1] || (_cache[1] = ($event: any) => (categoryChosen()))
          }, _toDisplayString(_ctx.$t('action.next')), 9 /* TEXT, PROPS */, ["disabled"]) ], 64 /* STABLE_FRAGMENT */)) : (step.value === 'selectStatuses') ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ _createElementVNode("h1", _hoisted_15, _toDisplayString(__props.status ? _ctx.$t('report.select_posts_other') : _ctx.$t('report.select_posts')), 1 /* TEXT */), _createElementVNode("p", _hoisted_16, _toDisplayString(_ctx.$t('report.select_many')), 1 /* TEXT */), _createElementVNode("table", null, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(availableStatuses.value, (availableStatus) => {
                return (_openBlock(), _createElementBlock("tr", { key: availableStatus.id }, [
                  _createElementVNode("td", null, [
                    _withDirectives(_createElementVNode("input", {
                      id: availableStatus.id,
                      "onUpdate:modelValue": [($event: any) => ((selectedStatusIds).value = $event), ($event: any) => ((selectedStatusIds).value = $event)],
                      type: "checkbox",
                      value: availableStatus.id
                    }, null, 8 /* PROPS */, ["id", "value"]), [
                      [_vModelCheckbox, selectedStatusIds.value]
                    ])
                  ]),
                  _createElementVNode("td", null, [
                    _createElementVNode("label", { for: availableStatus.id }, [
                      _createVNode(_component_StatusCard, {
                        status: availableStatus,
                        actions: false,
                        "pointer-events-none": ""
                      }, null, 8 /* PROPS */, ["status", "actions"])
                    ], 8 /* PROPS */, ["for"])
                  ])
                ]))
              }), 128 /* KEYED_FRAGMENT */)) ]), _createElementVNode("button", {
              "btn-solid": "",
              mxa: "",
              "mt-5": "",
              onClick: _cache[2] || (_cache[2] = ($event: any) => (submitReport()))
            }, _toDisplayString(_ctx.$t('report.submit')), 1 /* TEXT */) ], 64 /* STABLE_FRAGMENT */)) : (step.value === 'furtherActions') ? (_openBlock(), _createElementBlock(_Fragment, { key: 2 }, [ _createElementVNode("h1", _hoisted_17, _toDisplayString(reportReason.value === 'dontlike' ? _ctx.$t('report.further_actions.limit.title') : _ctx.$t('report.further_actions.report.title')), 1 /* TEXT */), _createElementVNode("p", _hoisted_18, _toDisplayString(reportReason.value === 'dontlike' ? _ctx.$t('report.further_actions.limit.description') : _ctx.$t('report.further_actions.report.description')), 1 /* TEXT */), (_unref(useRelationship)(__props.account).value?.following) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createElementVNode("button", {
                  "btn-outline": "",
                  mxa: "",
                  "mt-4": "",
                  "mb-2": "",
                  onClick: _cache[3] || (_cache[3] = ($event: any) => (unfollow()))
                }, [ _createVNode(_component_i18n_t, { keypath: "menu.unfollow_account" }, {
                    default: _withCtx(() => [
                      _createElementVNode("b", null, "@" + _toDisplayString(__props.account.acct), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }) ]), _hoisted_19, _createTextVNode("\n        "), _toDisplayString(_ctx.$t('report.unfollow_desc')) ])) : _createCommentVNode("v-if", true), (!_unref(useRelationship)(__props.account).value?.muting) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createElementVNode("button", {
                  "btn-outline": "",
                  mxa: "",
                  "mt-4": "",
                  "mb-2": "",
                  onClick: _cache[4] || (_cache[4] = ($event: any) => (mute()))
                }, [ _createVNode(_component_i18n_t, { keypath: "menu.mute_account" }, {
                    default: _withCtx(() => [
                      _createElementVNode("b", null, "@" + _toDisplayString(__props.account.acct), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }) ]), _hoisted_20, _createTextVNode("\n        "), _toDisplayString(_ctx.$t('report.mute_desc')) ])) : _createCommentVNode("v-if", true), (!_unref(useRelationship)(__props.account).value?.blocking) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createElementVNode("button", {
                  "btn-outline": "",
                  mxa: "",
                  "mt-4": "",
                  "mb-2": "",
                  onClick: _cache[5] || (_cache[5] = ($event: any) => (block()))
                }, [ _createVNode(_component_i18n_t, { keypath: "menu.block_account" }, {
                    default: _withCtx(() => [
                      _createElementVNode("b", null, "@" + _toDisplayString(__props.account.acct), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }) ]), _hoisted_21, _createTextVNode("\n        "), _toDisplayString(_ctx.$t('report.block_desc')) ])) : _createCommentVNode("v-if", true), _createElementVNode("button", {
              "btn-solid": "",
              mxa: "",
              "mt-10": "",
              onClick: _cache[6] || (_cache[6] = ($event: any) => (emit('close')))
            }, _toDisplayString(_ctx.$t('action.done')), 1 /* TEXT */) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ]))
}
}

})
