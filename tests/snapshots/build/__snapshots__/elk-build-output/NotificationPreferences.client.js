import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = { id: "pn-settings", px6: "true", py4: "true", mt2: "true", "font-bold": "true", "text-xl": "true", flex: "~ gap-1", "items-center": "true" }
const _hoisted_2 = { id: "pn-instructions", "text-sm": "true", pb2: "true", "aria-hidden": "true" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { block: "true", "i-ri:loader-2-fill": "true", "aria-hidden": "true" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", class: "block i-material-symbols:undo-rounded" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("span", { border: "b base 2px", class: "bg-$c-text-secondary" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("span", { block: "true", "i-ri:loader-2-fill": "true", "aria-hidden": "true" })
const _hoisted_7 = { id: "n-unsupported" }

export default /*@__PURE__*/_defineComponent({
  __name: 'NotificationPreferences.client',
  props: {
    show: { type: Boolean, required: false }
  },
  setup(__props: any) {

const {
  pushNotificationData,
  saveEnabled,
  undoChanges,
  hiddenNotification,
  isSubscribed,
  isSupported,
  notificationPermission,
  updateSubscription,
  subscribe,
  unsubscribe,
} = usePushManager()
const { t } = useI18n()
const pwaEnabled = useAppConfig().pwaEnabled
const busy = ref<boolean>(false)
const animateSave = ref<boolean>(false)
const animateSubscription = ref<boolean>(false)
const animateRemoveSubscription = ref<boolean>(false)
const subscribeError = ref<string>('')
const showSubscribeError = ref<boolean>(false)
function hideNotification() {
  const key = currentUser.value?.account?.acct
  if (key)
    hiddenNotification.value[key] = true
}
const showWarning = computed(() => {
  if (!pwaEnabled)
    return false

  return isSupported
    && (!isSubscribed.value || !notificationPermission.value || notificationPermission.value === 'prompt')
    && !(hiddenNotification.value[currentUser.value?.account?.acct ?? ''])
})
async function saveSettings() {
  if (busy.value)
    return
  busy.value = true
  await nextTick()
  animateSave.value = true
  try {
    await updateSubscription()
  }
  catch (err) {
    // todo: handle error
    console.error(err)
  }
  finally {
    busy.value = false
    animateSave.value = false
  }
}
async function doSubscribe() {
  if (busy.value)
    return
  busy.value = true
  await nextTick()
  animateSubscription.value = true
  try {
    const result = await subscribe()
    if (result !== 'subscribed') {
      subscribeError.value = t(`settings.notifications.push_notifications.subscription_error.${result === 'notification-denied' ? 'permission_denied' : 'request_error'}`)
      showSubscribeError.value = true
    }
  }
  catch (err) {
    if (err instanceof PushSubscriptionError) {
      subscribeError.value = t(`settings.notifications.push_notifications.subscription_error.${err.code}`)
    }
    else {
      console.error(err)
      subscribeError.value = t('settings.notifications.push_notifications.subscription_error.request_error')
    }
    showSubscribeError.value = true
  }
  finally {
    busy.value = false
    animateSubscription.value = false
  }
}
async function removeSubscription() {
  if (busy.value)
    return
  busy.value = true
  await nextTick()
  animateRemoveSubscription.value = true
  try {
    await unsubscribe()
  }
  catch (err) {
    console.error(err)
  }
  finally {
    busy.value = false
    animateRemoveSubscription.value = false
  }
}
onActivated(() => (busy.value = false))

return (_ctx: any,_cache: any) => {
  const _component_CommonCheckbox = _resolveComponent("CommonCheckbox")
  const _component_CommonRadio = _resolveComponent("CommonRadio")
  const _component_NotificationSubscribePushNotificationError = _resolveComponent("NotificationSubscribePushNotificationError")
  const _component_NotificationEnablePushNotification = _resolveComponent("NotificationEnablePushNotification")

  return (_unref(pwaEnabled) && (showWarning.value || __props.show))
      ? (_openBlock(), _createElementBlock("section", {
        key: 0,
        "aria-labelledby": "pn-s"
      }, [ _createVNode(_Transition, { name: "slide-down" }, {
          default: _withCtx(() => [
            (__props.show)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                flex: "~ col",
                border: "b base"
              }, [
                _createElementVNode("h3", _hoisted_1, _toDisplayString(_ctx.$t('settings.notifications.push_notifications.label')), 1 /* TEXT */),
                (_unref(isSupported))
                  ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                    (_unref(isSubscribed))
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        flex: "~ col"
                      }, [
                        _createElementVNode("form", {
                          flex: "~ col",
                          "gap-y-2": "",
                          px6: "",
                          pb4: "",
                          onSubmit: _withModifiers(saveSettings, ["prevent"])
                        }, [
                          _createElementVNode("p", _hoisted_2, _toDisplayString(_ctx.$t('settings.notifications.push_notifications.instructions')), 1 /* TEXT */),
                          _createElementVNode("fieldset", {
                            flex: "~ col",
                            "gap-y-1": "",
                            "py-1": ""
                          }, [
                            _createElementVNode("legend", null, _toDisplayString(_ctx.$t('settings.notifications.push_notifications.alerts.title')), 1 /* TEXT */),
                            _createVNode(_component_CommonCheckbox, {
                              hover: "",
                              label: _ctx.$t('settings.notifications.push_notifications.alerts.follow'),
                              modelValue: _unref(pushNotificationData).follow,
                              "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((_unref(pushNotificationData).follow) = $event))
                            }, null, 8 /* PROPS */, ["label", "modelValue"]),
                            _createVNode(_component_CommonCheckbox, {
                              hover: "",
                              label: _ctx.$t('settings.notifications.push_notifications.alerts.favourite'),
                              modelValue: _unref(pushNotificationData).favourite,
                              "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((_unref(pushNotificationData).favourite) = $event))
                            }, null, 8 /* PROPS */, ["label", "modelValue"]),
                            _createVNode(_component_CommonCheckbox, {
                              hover: "",
                              label: _ctx.$t('settings.notifications.push_notifications.alerts.reblog'),
                              modelValue: _unref(pushNotificationData).reblog,
                              "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((_unref(pushNotificationData).reblog) = $event))
                            }, null, 8 /* PROPS */, ["label", "modelValue"]),
                            _createVNode(_component_CommonCheckbox, {
                              hover: "",
                              label: _ctx.$t('settings.notifications.push_notifications.alerts.mention'),
                              modelValue: _unref(pushNotificationData).mention,
                              "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((_unref(pushNotificationData).mention) = $event))
                            }, null, 8 /* PROPS */, ["label", "modelValue"]),
                            _createVNode(_component_CommonCheckbox, {
                              hover: "",
                              label: _ctx.$t('settings.notifications.push_notifications.alerts.poll'),
                              modelValue: _unref(pushNotificationData).poll,
                              "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((_unref(pushNotificationData).poll) = $event))
                            }, null, 8 /* PROPS */, ["label", "modelValue"])
                          ]),
                          _createElementVNode("fieldset", {
                            flex: "~ col",
                            "gap-y-1": "",
                            "py-1": ""
                          }, [
                            _createElementVNode("legend", null, _toDisplayString(_ctx.$t('settings.notifications.push_notifications.policy.title')), 1 /* TEXT */),
                            _createVNode(_component_CommonRadio, {
                              hover: "",
                              value: "all",
                              label: _ctx.$t('settings.notifications.push_notifications.policy.all'),
                              modelValue: _unref(pushNotificationData).policy,
                              "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((_unref(pushNotificationData).policy) = $event))
                            }, null, 8 /* PROPS */, ["label", "modelValue"]),
                            _createVNode(_component_CommonRadio, {
                              hover: "",
                              value: "followed",
                              label: _ctx.$t('settings.notifications.push_notifications.policy.followed'),
                              modelValue: _unref(pushNotificationData).policy,
                              "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((_unref(pushNotificationData).policy) = $event))
                            }, null, 8 /* PROPS */, ["label", "modelValue"]),
                            _createVNode(_component_CommonRadio, {
                              hover: "",
                              value: "follower",
                              label: _ctx.$t('settings.notifications.push_notifications.policy.follower'),
                              modelValue: _unref(pushNotificationData).policy,
                              "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => ((_unref(pushNotificationData).policy) = $event))
                            }, null, 8 /* PROPS */, ["label", "modelValue"]),
                            _createVNode(_component_CommonRadio, {
                              hover: "",
                              value: "none",
                              label: _ctx.$t('settings.notifications.push_notifications.policy.none'),
                              modelValue: _unref(pushNotificationData).policy,
                              "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event: any) => ((_unref(pushNotificationData).policy) = $event))
                            }, null, 8 /* PROPS */, ["label", "modelValue"])
                          ]),
                          _createElementVNode("div", {
                            flex: "~ col",
                            "gap-y-4": "",
                            "gap-x-2": "",
                            "py-1": "",
                            sm: "~ justify-between flex-row"
                          }, [
                            _createElementVNode("button", {
                              "btn-solid": "",
                              "font-bold": "",
                              py2: "",
                              "full-w": "",
                              "sm-wa": "",
                              flex: "~ gap2 center",
                              class: _normalizeClass(busy.value || !_unref(saveEnabled) ? 'border-transparent' : null),
                              disabled: busy.value || !_unref(saveEnabled)
                            }, [
                              (busy.value && animateSave.value)
                                ? (_openBlock(), _createElementBlock("span", {
                                  key: 0,
                                  "aria-hidden": "true",
                                  block: "",
                                  "animate-spin": "",
                                  "preserve-3d": ""
                                }, [
                                  _hoisted_3
                                ]))
                                : (_openBlock(), _createElementBlock("span", {
                                  key: 1,
                                  block: "",
                                  "aria-hidden": "true",
                                  "i-ri:save-2-fill": ""
                                })),
                              _createTextVNode("\n                  " + _toDisplayString(_ctx.$t('settings.notifications.push_notifications.save_settings')), 1 /* TEXT */)
                            ], 10 /* CLASS, PROPS */, ["disabled"]),
                            _createElementVNode("button", {
                              "btn-outline": "",
                              "font-bold": "",
                              py2: "",
                              "full-w": "",
                              "sm-wa": "",
                              flex: "~ gap2 center",
                              type: "button",
                              class: _normalizeClass(busy.value || !_unref(saveEnabled) ? 'border-transparent' : null),
                              disabled: busy.value || !_unref(saveEnabled),
                              onClick: _cache[9] || (_cache[9] = (...args) => (undoChanges && undoChanges(...args)))
                            }, [
                              _hoisted_4,
                              _createTextVNode("\n                  " + _toDisplayString(_ctx.$t('settings.notifications.push_notifications.undo_settings')), 1 /* TEXT */)
                            ], 10 /* CLASS, PROPS */, ["disabled"])
                          ])
                        ], 32 /* NEED_HYDRATION */),
                        _createElementVNode("form", {
                          flex: "~ col",
                          "mt-4": "",
                          onSubmit: _withModifiers(removeSubscription, ["prevent"])
                        }, [
                          _hoisted_5,
                          _createElementVNode("button", {
                            "btn-outline": "",
                            "rounded-full": "",
                            "font-bold": "",
                            "py-4": "",
                            flex: "~ gap2 center",
                            m5: "",
                            class: _normalizeClass(busy.value ? 'border-transparent' : null),
                            disabled: busy.value
                          }, [
                            (busy.value && animateRemoveSubscription.value)
                              ? (_openBlock(), _createElementBlock("span", {
                                key: 0,
                                "aria-hidden": "true",
                                block: "",
                                "animate-spin": "",
                                "preserve-3d": ""
                              }, [
                                _hoisted_6
                              ]))
                              : (_openBlock(), _createElementBlock("span", {
                                key: 1,
                                block: "",
                                "aria-hidden": "true",
                                "i-material-symbols:cancel-rounded": ""
                              })),
                            _createTextVNode("\n                " + _toDisplayString(_ctx.$t('settings.notifications.push_notifications.unsubscribe')), 1 /* TEXT */)
                          ], 10 /* CLASS, PROPS */, ["disabled"])
                        ], 32 /* NEED_HYDRATION */)
                      ]))
                      : (_openBlock(), _createBlock(_component_NotificationEnablePushNotification, {
                        key: 1,
                        animate: animateSubscription.value,
                        busy: busy.value,
                        onHide: hideNotification,
                        onSubscribe: doSubscribe
                      }, {
                        error: _withCtx(() => [
                          _createVNode(_Transition, { name: "slide-down" }, {
                            default: _withCtx(() => [
                              _createVNode(_component_NotificationSubscribePushNotificationError, {
                                message: subscribeError.value,
                                modelValue: showSubscribeError.value,
                                "onUpdate:modelValue": _cache[10] || (_cache[10] = ($event: any) => ((showSubscribeError).value = $event))
                              }, null, 8 /* PROPS */, ["message", "modelValue"])
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["animate", "busy"]))
                  ], 64 /* STABLE_FRAGMENT */))
                  : (_openBlock(), _createElementBlock("div", {
                    key: 1,
                    px6: "",
                    pb4: "",
                    role: "alert",
                    "aria-labelledby": "n-unsupported"
                  }, [
                    _createElementVNode("p", _hoisted_7, _toDisplayString(_ctx.$t('settings.notifications.push_notifications.unsupported')), 1 /* TEXT */)
                  ]))
              ]))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }), (showWarning.value && !__props.show) ? (_openBlock(), _createBlock(_component_NotificationEnablePushNotification, {
            key: 0,
            "closeable-header": "",
            px5: "",
            py4: "",
            animate: animateSubscription.value,
            busy: busy.value,
            onHide: hideNotification,
            onSubscribe: doSubscribe
          }, {
            error: _withCtx(() => [
              _createVNode(_Transition, { name: "slide-down" }, {
                default: _withCtx(() => [
                  _createVNode(_component_NotificationSubscribePushNotificationError, {
                    message: subscribeError.value,
                    modelValue: showSubscribeError.value,
                    "onUpdate:modelValue": _cache[11] || (_cache[11] = ($event: any) => ((showSubscribeError).value = $event))
                  }, null, 8 /* PROPS */, ["message", "modelValue"])
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["animate", "busy"])) : _createCommentVNode("v-if", true) ]))
      : _createCommentVNode("v-if", true)
}
}

})
