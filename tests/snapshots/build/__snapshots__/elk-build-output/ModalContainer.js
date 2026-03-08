import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, mergeProps as _mergeProps, withCtx as _withCtx, unref as _unref } from "vue"

import type { ConfirmDialogChoice } from '#shared/types'
import type { mastodon } from 'masto'
import { isCommandPanelOpen, isConfirmDialogOpen, isEditHistoryDialogOpen, isErrorDialogOpen, isKeyboardShortcutsDialogOpen, isMediaPreviewOpen, isPreviewHelpOpen, isPublishDialogOpen, isReactedByDialogOpen, isReportDialogOpen, isSigninDialogOpen } from '~/composables/dialog'

export default /*@__PURE__*/_defineComponent({
  __name: 'ModalContainer',
  setup(__props) {

const isMac = useIsMac()
// TODO: temporary, await for keybind system
// open search panel
// listen to ctrl+k on windows/linux or cmd+k on mac
// open command panel
// listen to ctrl+/ on windows/linux or cmd+/ on mac
// or shift+ctrl+k on windows/linux or shift+cmd+k on mac
useEventListener('keydown', (e: KeyboardEvent) => {
  if ((e.key === 'k' || e.key === 'л') && (isMac.value ? e.metaKey : e.ctrlKey)) {
    e.preventDefault()
    openCommandPanel(e.shiftKey)
  }
  if ((e.key === '/' || e.key === ',') && (isMac.value ? e.metaKey : e.ctrlKey)) {
    e.preventDefault()
    openCommandPanel(true)
  }
})
function handlePublished(status: mastodon.v1.Status) {
  lastPublishDialogStatus.value = status
  isPublishDialogOpen.value = false
}
function handlePublishClose() {
  lastPublishDialogStatus.value = null
}
function handleConfirmChoice(choice: ConfirmDialogChoice) {
  confirmDialogChoice.value = choice
  isConfirmDialogOpen.value = false
}
function handleReactedByClose() {
  isReactedByDialogOpen.value = false
}

return (_ctx: any,_cache: any) => {
  const _component_UserSignIn = _resolveComponent("UserSignIn")
  const _component_ModalDialog = _resolveComponent("ModalDialog")
  const _component_HelpPreview = _resolveComponent("HelpPreview")
  const _component_PublishWidgetList = _resolveComponent("PublishWidgetList")
  const _component_ModalMediaPreview = _resolveComponent("ModalMediaPreview")
  const _component_StatusEditPreview = _resolveComponent("StatusEditPreview")
  const _component_CommandPanel = _resolveComponent("CommandPanel")
  const _component_ModalConfirm = _resolveComponent("ModalConfirm")
  const _component_ModalError = _resolveComponent("ModalError")
  const _component_StatusReactedBy = _resolveComponent("StatusReactedBy")
  const _component_MagickeysKeyboardShortcuts = _resolveComponent("MagickeysKeyboardShortcuts")
  const _component_ReportModal = _resolveComponent("ReportModal")

  return (_ctx.isHydrated)
      ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createVNode(_component_ModalDialog, {
          "py-4": "",
          "px-8": "",
          "max-w-125": "",
          modelValue: _unref(isSigninDialogOpen),
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((isSigninDialogOpen).value = $event))
        }, {
          default: _withCtx(() => [
            _createVNode(_component_UserSignIn)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["modelValue"]), _createVNode(_component_ModalDialog, {
          "keep-alive": "",
          "max-w-125": "",
          modelValue: _unref(isPreviewHelpOpen),
          "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((isPreviewHelpOpen).value = $event))
        }, {
          default: _withCtx(() => [
            _createVNode(_component_HelpPreview, {
              onClose: _cache[2] || (_cache[2] = ($event: any) => (_ctx.closePreviewHelp()))
            })
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["modelValue"]), _createVNode(_component_ModalDialog, {
          "max-w-180": "",
          flex: "",
          "w-full": "",
          onClose: handlePublishClose,
          modelValue: _unref(isPublishDialogOpen),
          "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((isPublishDialogOpen).value = $event))
        }, {
          default: _withCtx(() => [
            (_ctx.dialogDraftKey)
              ? (_openBlock(), _createBlock(_component_PublishWidgetList, {
                key: 0,
                "draft-key": _ctx.dialogDraftKey,
                expanded: "",
                class: "flex-1",
                onPublished: handlePublished
              }, null, 8 /* PROPS */, ["draft-key"]))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["modelValue"]), _createVNode(_component_ModalDialog, {
          "model-value": _unref(isMediaPreviewOpen),
          "w-full": "",
          "max-w-full": "",
          "h-full": "",
          "max-h-full": "",
          "bg-transparent": "",
          "border-0": "",
          "shadow-none": "",
          "onUpdate:model-value": _cache[4] || (_cache[4] = (...args) => (_ctx.closeMediaPreview && _ctx.closeMediaPreview(...args)))
        }, {
          default: _withCtx(() => [
            (_unref(isMediaPreviewOpen))
              ? (_openBlock(), _createBlock(_component_ModalMediaPreview, {
                key: 0,
                onClose: _cache[5] || (_cache[5] = ($event: any) => (_ctx.closeMediaPreview()))
              }))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["model-value"]), _createVNode(_component_ModalDialog, {
          "focus-first-element": false,
          "max-w-125": "",
          modelValue: _unref(isEditHistoryDialogOpen),
          "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((isEditHistoryDialogOpen).value = $event))
        }, {
          default: _withCtx(() => [
            (_ctx.statusEdit)
              ? (_openBlock(), _createBlock(_component_StatusEditPreview, {
                key: 0,
                edit: _ctx.statusEdit
              }, null, 8 /* PROPS */, ["edit"]))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["focus-first-element", "modelValue"]), _createVNode(_component_ModalDialog, {
          "max-w-fit": "",
          flex: "",
          modelValue: _unref(isCommandPanelOpen),
          "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => ((isCommandPanelOpen).value = $event))
        }, {
          default: _withCtx(() => [
            _createVNode(_component_CommandPanel, {
              onClose: _cache[8] || (_cache[8] = ($event: any) => (_ctx.closeCommandPanel()))
            })
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["modelValue"]), _createVNode(_component_ModalDialog, {
          "py-4": "",
          "px-8": "",
          "max-w-125": "",
          modelValue: _unref(isConfirmDialogOpen),
          "onUpdate:modelValue": _cache[9] || (_cache[9] = ($event: any) => ((isConfirmDialogOpen).value = $event))
        }, {
          default: _withCtx(() => [
            (_ctx.confirmDialogLabel)
              ? (_openBlock(), _createBlock(_component_ModalConfirm, _mergeProps(_ctx.confirmDialogLabel, {
                key: 0,
                onChoice: handleConfirmChoice
              }), null, 16 /* FULL_PROPS */))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["modelValue"]), _createVNode(_component_ModalDialog, {
          "py-4": "",
          "px-8": "",
          "max-w-125": "",
          modelValue: _unref(isErrorDialogOpen),
          "onUpdate:modelValue": _cache[10] || (_cache[10] = ($event: any) => ((isErrorDialogOpen).value = $event))
        }, {
          default: _withCtx(() => [
            (_ctx.errorDialogData)
              ? (_openBlock(), _createBlock(_component_ModalError, _mergeProps(_ctx.errorDialogData, { key: 0 }), null, 16 /* FULL_PROPS */))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["modelValue"]), _createVNode(_component_ModalDialog, {
          "max-w-180": "",
          onClose: handleReactedByClose,
          modelValue: _unref(isReactedByDialogOpen),
          "onUpdate:modelValue": _cache[11] || (_cache[11] = ($event: any) => ((isReactedByDialogOpen).value = $event))
        }, {
          default: _withCtx(() => [
            _createVNode(_component_StatusReactedBy)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["modelValue"]), _createVNode(_component_ModalDialog, {
          "max-w-full": "",
          "sm:max-w-140": "",
          "md:max-w-170": "",
          "lg:max-w-220": "",
          "md:min-w-160": "",
          modelValue: _unref(isKeyboardShortcutsDialogOpen),
          "onUpdate:modelValue": _cache[12] || (_cache[12] = ($event: any) => ((isKeyboardShortcutsDialogOpen).value = $event))
        }, {
          default: _withCtx(() => [
            _createVNode(_component_MagickeysKeyboardShortcuts, {
              onClose: _cache[13] || (_cache[13] = ($event: any) => (_ctx.closeKeyboardShortcuts()))
            })
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["modelValue"]), _createVNode(_component_ModalDialog, {
          "keep-alive": "",
          "max-w-175": "",
          modelValue: _unref(isReportDialogOpen),
          "onUpdate:modelValue": _cache[14] || (_cache[14] = ($event: any) => ((isReportDialogOpen).value = $event))
        }, {
          default: _withCtx(() => [
            (_ctx.reportAccount)
              ? (_openBlock(), _createBlock(_component_ReportModal, {
                key: 0,
                account: _ctx.reportAccount,
                status: _ctx.reportStatus,
                onClose: _cache[15] || (_cache[15] = ($event: any) => (_ctx.closeReportDialog()))
              }, null, 8 /* PROPS */, ["account", "status"]))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["modelValue"]) ], 64 /* STABLE_FRAGMENT */))
      : _createCommentVNode("v-if", true)
}
}

})
