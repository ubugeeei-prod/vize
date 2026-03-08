import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref, vModelText as _vModelText, vModelCheckbox as _vModelCheckbox, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = { "font-medium": "true" }
const _hoisted_2 = { "font-medium": "true" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("div", { "aria-hidden": "true", "i-ri:eraser-line": "true" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { block: "true", "i-ri:loader-2-fill": "true", "aria-hidden": "true" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", block: "true", "i-carbon:face-dizzy-filled": "true" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("div", { "aria-hidden": "true", "i-ri:error-warning-fill": "true" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", w: "1.75em", h: "1.75em", "i-ri:close-line": "true" })
import type { mastodon } from 'masto'
import { useForm } from 'slimeform'

export default /*@__PURE__*/_defineComponent({
  __name: 'appearance',
  setup(__props) {

definePageMeta({
  middleware: 'auth',
})
const { t } = useI18n()
useHydratedHead({
  title: () => `${t('settings.profile.appearance.title')} | ${t('nav.settings')}`,
})
const { client } = useMasto()
const avatarInput = ref<any>()
const headerInput = ref<any>()
const account = computed(() => currentUser.value?.account)
const onlineSrc = computed(() => ({
  avatar: account.value?.avatar || '',
  header: account.value?.header || '',
}))
const { form, reset, submitter, isDirty, isError } = useForm({
  form: () => {
    // For complex types of objects, a deep copy is required to ensure correct comparison of initial and modified values
    const fieldsAttributes = Array.from({ length: maxAccountFieldCount.value }, (_, i) => {
      const field = { ...account.value?.fields?.[i] || { name: '', value: '' } }

      field.value = convertMetadata(field.value)

      return field
    })
    return {
      displayName: account.value?.displayName ?? '',
      note: account.value?.source.note.replaceAll('\r', '') ?? '',

      avatar: null as null | File,
      header: null as null | File,

      fieldsAttributes,

      bot: account.value?.bot ?? false,
      locked: account.value?.locked ?? false,

      // These look more like account and privacy settings than appearance settings
      // discoverable: false,
      // locked: false,
    }
  },
})
const isCanSubmit = computed(() => !isError.value && isDirty.value)
const failedMessages = ref<string[]>([])
const { submit, submitting } = submitter(async ({ dirtyFields }) => {
  if (!isCanSubmit.value)
    return

  const res = await client.value.v1.accounts.updateCredentials(dirtyFields.value as mastodon.rest.v1.UpdateCredentialsParams)
    .then(account => ({ account }))
    .catch((error: Error) => ({ error }))

  if ('error' in res) {
    console.error(res.error)
    failedMessages.value.push(res.error.message)
    return
  }

  const server = currentUser.value!.server

  if (!res.account.acct.includes('@'))
    res.account.acct = `${res.account.acct}@${server}`

  cacheAccount(res.account, server, true)
  currentUser.value!.account = res.account
  reset()
})
async function refreshInfo() {
  if (!currentUser.value)
    return
  // Keep the information to be edited up to date
  await refreshAccountInfo()
  if (!isDirty)
    reset()
}
useDropZone(avatarInput, (files) => {
  if (files?.[0])
    form.avatar = files[0]
})
useDropZone(headerInput, (files) => {
  if (files?.[0])
    form.header = files[0]
})
onHydrated(refreshInfo)
onReactivated(refreshInfo)

return (_ctx: any,_cache: any) => {
  const _component_MainTitle = _resolveComponent("MainTitle")
  const _component_CommonInputImage = _resolveComponent("CommonInputImage")
  const _component_CommonCropImage = _resolveComponent("CommonCropImage")
  const _component_AccountDisplayName = _resolveComponent("AccountDisplayName")
  const _component_AccountLockIndicator = _resolveComponent("AccountLockIndicator")
  const _component_AccountBotIndicator = _resolveComponent("AccountBotIndicator")
  const _component_AccountHandle = _resolveComponent("AccountHandle")
  const _component_SettingsProfileMetadata = _resolveComponent("SettingsProfileMetadata")
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")
  const _component_CommonErrorMessage = _resolveComponent("CommonErrorMessage")
  const _component_MainContent = _resolveComponent("MainContent")

  return (_openBlock(), _createBlock(_component_MainContent, { back: "" }, {
      title: _withCtx(() => [
        _createVNode(_component_MainTitle, {
          as: "h1",
          secondary: ""
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_ctx.$t('settings.profile.appearance.title')), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })
      ]),
      default: _withCtx(() => [
        _createElementVNode("form", {
          "space-y-5": "",
          onSubmit: _cache[0] || (_cache[0] = _withModifiers((...args) => (submit && submit(...args)), ["prevent"]))
        }, [
          (account.value)
            ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
              _createElementVNode("div", {
                "of-hidden": "",
                bg: "gray-500/20",
                aspect: "3"
              }, [
                _createVNode(_component_CommonInputImage, {
                  ref_key: "headerInput", ref: headerInput,
                  original: onlineSrc.value.header,
                  "w-full": "",
                  "h-full": "",
                  modelValue: _unref(form).header,
                  "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((_unref(form).header) = $event))
                }, null, 8 /* PROPS */, ["original", "modelValue"])
              ]),
              _createVNode(_component_CommonCropImage, {
                "stencil-aspect-ratio": 3 / 1,
                modelValue: _unref(form).header,
                "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((_unref(form).header) = $event))
              }, null, 8 /* PROPS */, ["stencil-aspect-ratio", "modelValue"]),
              _createTextVNode("\n\n        "),
              _createTextVNode("\n        "),
              _createElementVNode("div", {
                "px-4": "",
                flex: "~ gap4"
              }, [
                _createVNode(_component_CommonInputImage, {
                  ref_key: "avatarInput", ref: avatarInput,
                  original: onlineSrc.value.avatar,
                  "mt--10": "",
                  "rounded-full": "",
                  border: "bg-base 4",
                  w: "sm:30 24",
                  "min-w": "sm:30 24",
                  h: "sm:30 24",
                  modelValue: _unref(form).avatar,
                  "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((_unref(form).avatar) = $event))
                }, null, 8 /* PROPS */, ["original", "modelValue"])
              ]),
              _createVNode(_component_CommonCropImage, {
                modelValue: _unref(form).avatar,
                "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((_unref(form).avatar) = $event))
              }, null, 8 /* PROPS */, ["modelValue"]),
              _createElementVNode("div", { px4: "" }, [
                _createElementVNode("div", {
                  flex: "",
                  "justify-between": ""
                }, [
                  _createVNode(_component_AccountDisplayName, {
                    account: { ...account.value, displayName: _unref(form).displayName },
                    "font-bold": "",
                    "sm:text-2xl": "",
                    "text-xl": ""
                  }, null, 8 /* PROPS */, ["account"]),
                  _createElementVNode("div", {
                    flex: "~ row",
                    "items-center": "",
                    gap2: ""
                  }, [
                    _createElementVNode("label", null, [
                      _createVNode(_component_AccountLockIndicator, {
                        "show-label": "",
                        px2: "",
                        py1: ""
                      }, {
                        prepend: _withCtx(() => [
                          _withDirectives(_createElementVNode("input", {
                            "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((_unref(form).locked) = $event)),
                            type: "checkbox",
                            "cursor-pointer": ""
                          }, null, 512 /* NEED_PATCH */), [
                            [_vModelCheckbox, _unref(form).locked]
                          ])
                        ]),
                        _: 1 /* STABLE */
                      })
                    ]),
                    _createElementVNode("label", null, [
                      _createVNode(_component_AccountBotIndicator, {
                        "show-label": "",
                        px2: "",
                        py1: ""
                      }, {
                        prepend: _withCtx(() => [
                          _withDirectives(_createElementVNode("input", {
                            "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((_unref(form).bot) = $event)),
                            type: "checkbox",
                            "cursor-pointer": ""
                          }, null, 512 /* NEED_PATCH */), [
                            [_vModelCheckbox, _unref(form).bot]
                          ])
                        ]),
                        _: 1 /* STABLE */
                      })
                    ])
                  ])
                ]),
                _createVNode(_component_AccountHandle, { account: account.value }, null, 8 /* PROPS */, ["account"])
              ])
            ]))
            : _createCommentVNode("v-if", true),
          _createElementVNode("div", {
            px4: "",
            py3: "",
            "space-y-5": ""
          }, [
            _createElementVNode("label", {
              "space-y-2": "",
              block: ""
            }, [
              _createElementVNode("p", _hoisted_1, _toDisplayString(_ctx.$t('settings.profile.appearance.display_name')), 1 /* TEXT */),
              _withDirectives(_createElementVNode("input", {
                "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => ((_unref(form).displayName) = $event)),
                type: "text",
                "input-base": ""
              }, null, 512 /* NEED_PATCH */), [
                [_vModelText, _unref(form).displayName]
              ])
            ]),
            _createTextVNode("\n\n        " + "\n        "),
            _createElementVNode("label", {
              "space-y-2": "",
              block: ""
            }, [
              _createElementVNode("p", _hoisted_2, _toDisplayString(_ctx.$t('settings.profile.appearance.bio')), 1 /* TEXT */),
              _withDirectives(_createElementVNode("textarea", {
                "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event: any) => ((_unref(form).note) = $event)),
                maxlength: "500",
                "min-h-10ex": "",
                "input-base": ""
              }, null, 512 /* NEED_PATCH */), [
                [_vModelText, _unref(form).note]
              ])
            ]),
            _createTextVNode("\n\n        " + "\n\n        "),
            _createVNode(_component_SettingsProfileMetadata, {
              modelValue: _unref(form),
              "onUpdate:modelValue": _cache[9] || (_cache[9] = ($event: any) => ((form).value = $event))
            }, null, 8 /* PROPS */, ["modelValue"]),
            _createTextVNode("\n\n        " + "\n        "),
            _createElementVNode("div", {
              flex: "~ gap2",
              "justify-end": ""
            }, [
              _createElementVNode("button", {
                type: "button",
                "btn-text": "",
                "text-sm": "",
                flex: "",
                "gap-x-2": "",
                "items-center": "",
                "text-red": "",
                onClick: _cache[10] || (_cache[10] = ($event: any) => (_unref(reset)()))
              }, [
                _hoisted_3,
                _createTextVNode("\n            " + _toDisplayString(_ctx.$t('action.reset')), 1 /* TEXT */)
              ]),
              (failedMessages.value.length === 0)
                ? (_openBlock(), _createElementBlock("button", {
                  key: 0,
                  type: "submit",
                  "btn-solid": "",
                  "rounded-full": "",
                  "text-sm": "",
                  flex: "",
                  "gap-x-2": "",
                  "items-center": "",
                  disabled: _unref(submitting) || !isCanSubmit.value
                }, [
                  (_unref(submitting))
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      "aria-hidden": "true",
                      block: "",
                      "animate-spin": "",
                      "preserve-3d": ""
                    }, [
                      _hoisted_4
                    ]))
                    : (_openBlock(), _createElementBlock("span", {
                      key: 1,
                      "aria-hidden": "true",
                      block: "",
                      "i-ri:save-line": ""
                    })),
                  _createTextVNode("\n            "),
                  _toDisplayString(_ctx.$t('action.save'))
                ]))
                : (_openBlock(), _createElementBlock("button", {
                  key: 1,
                  type: "submit",
                  "btn-danger": "",
                  "rounded-full": "",
                  "text-sm": "",
                  flex: "",
                  "gap-x-2": "",
                  "items-center": ""
                }, [
                  _hoisted_5,
                  _createElementVNode("span", null, _toDisplayString(_ctx.$t('state.save_failed')), 1 /* TEXT */)
                ]))
            ]),
            (failedMessages.value.length > 0)
              ? (_openBlock(), _createBlock(_component_CommonErrorMessage, {
                key: 0,
                "described-by": "save-failed"
              }, {
                default: _withCtx(() => [
                  _createElementVNode("header", {
                    id: "save-failed",
                    flex: "",
                    "justify-between": ""
                  }, [
                    _createElementVNode("div", {
                      flex: "",
                      "items-center": "",
                      "gap-x-2": "",
                      "font-bold": ""
                    }, [
                      _hoisted_6,
                      _createElementVNode("p", null, _toDisplayString(_ctx.$t('state.save_failed')), 1 /* TEXT */)
                    ]),
                    _createVNode(_component_CommonTooltip, {
                      placement: "bottom",
                      content: _ctx.$t('action.clear_save_failed')
                    }, {
                      default: _withCtx(() => [
                        _createElementVNode("button", {
                          flex: "",
                          "rounded-4": "",
                          p1: "",
                          "hover:bg-active": "",
                          "cursor-pointer": "",
                          "transition-100": "",
                          "aria-label": _ctx.$t('action.clear_save_failed'),
                          onClick: _cache[11] || (_cache[11] = ($event: any) => (failedMessages.value = []))
                        }, [
                          _hoisted_7
                        ], 8 /* PROPS */, ["aria-label"])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["content"])
                  ]),
                  _createElementVNode("ol", {
                    "ps-2": "",
                    "sm:ps-1": ""
                  }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(failedMessages.value, (error, i) => {
                      return (_openBlock(), _createElementBlock("li", {
                        key: i,
                        flex: "~ col sm:row",
                        "gap-y-1": "",
                        "sm:gap-x-2": ""
                      }, [
                        _createElementVNode("strong", null, _toDisplayString(i + 1) + ".", 1 /* TEXT */),
                        _createElementVNode("span", null, _toDisplayString(error), 1 /* TEXT */)
                      ]))
                    }), 128 /* KEYED_FRAGMENT */))
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true)
          ])
        ], 32 /* NEED_HYDRATION */)
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
