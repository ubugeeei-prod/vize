import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, createSlots as _createSlots, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-mail" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check", style: "color: var(--MI_THEME-success);" })
import { onMounted, ref, watch, computed } from 'vue'
import FormSection from '@/components/form/section.vue'
import MkInfo from '@/components/MkInfo.vue'
import MkInput from '@/components/MkInput.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkDisableSection from '@/components/MkDisableSection.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { ensureSignin } from '@/i.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { instance } from '@/instance.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'email',
  setup(__props) {

const $i = ensureSignin();
const emailAddress = ref($i.email ?? '');
function onChangeReceiveAnnouncementEmail(v: boolean) {
	misskeyApi('i/update', {
		receiveAnnouncementEmail: v,
	});
}
async function saveEmailAddress() {
	const auth = await os.authenticateDialog();
	if (auth.canceled) return;
	os.apiWithDialog('i/update-email', {
		password: auth.result.password,
		token: auth.result.token,
		email: emailAddress.value,
	});
}
const emailNotification_mention = ref($i.emailNotificationTypes.includes('mention'));
const emailNotification_reply = ref($i.emailNotificationTypes.includes('reply'));
const emailNotification_quote = ref($i.emailNotificationTypes.includes('quote'));
const emailNotification_follow = ref($i.emailNotificationTypes.includes('follow'));
const emailNotification_receiveFollowRequest = ref($i.emailNotificationTypes.includes('receiveFollowRequest'));
const saveNotificationSettings = () => {
	misskeyApi('i/update', {
		emailNotificationTypes: [
			...[emailNotification_mention.value ? 'mention' : null],
			...[emailNotification_reply.value ? 'reply' : null],
			...[emailNotification_quote.value ? 'quote' : null],
			...[emailNotification_follow.value ? 'follow' : null],
			...[emailNotification_receiveFollowRequest.value ? 'receiveFollowRequest' : null],
		].filter(x => x != null),
	});
};
watch([emailNotification_mention, emailNotification_reply, emailNotification_quote, emailNotification_follow, emailNotification_receiveFollowRequest], () => {
	saveNotificationSettings();
});
onMounted(() => {
	watch(emailAddress, () => {
		saveEmailAddress();
	});
});
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.email,
	icon: 'ti ti-mail',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_SearchMarker = _resolveComponent("SearchMarker")

  return (_openBlock(), _createBlock(_component_SearchMarker, {
      path: "/settings/email",
      label: _unref(i18n).ts.email,
      keywords: ['email'],
      icon: "ti ti-mail"
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", { class: "_gaps_m" }, [
          (!_unref(instance).enableEmail)
            ? (_openBlock(), _createBlock(MkInfo, { key: 0 }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.emailNotSupported), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }))
            : _createCommentVNode("v-if", true),
          _createVNode(MkDisableSection, { disabled: !_unref(instance).enableEmail }, {
            default: _withCtx(() => [
              _createElementVNode("div", { class: "_gaps_m" }, [
                _createVNode(_component_SearchMarker, { keywords: ['email', 'address'] }, {
                  default: _withCtx(() => [
                    _createVNode(FormSection, { first: "" }, {
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.emailAddress), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      default: _withCtx(() => [
                        _createVNode(MkInput, {
                          type: "email",
                          manualSave: "",
                          modelValue: emailAddress.value,
                          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((emailAddress).value = $event))
                        }, _createSlots({ _: 2 /* DYNAMIC */ }, [
                          {
                            name: "prefix",
                            fn: _withCtx(() => [
                              _hoisted_1
                            ])
                          },
                          (_unref($i).email && !_unref($i).emailVerified)
                            ? {
                              name: "caption",
                              fn: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.verificationEmailSent), 1 /* TEXT */)
                              ]),
                              key: "0"
                            }
                          : (emailAddress.value === _unref($i).email && _unref($i).emailVerified)
                            ? {
                              name: "caption",
                              fn: _withCtx(() => [
                                _hoisted_2,
                                _createTextVNode(" "),
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.emailVerified), 1 /* TEXT */)
                              ]),
                              key: "1"
                            }
                          : undefined
                        ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["modelValue"])
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(FormSection, null, {
                  default: _withCtx(() => [
                    _createVNode(_component_SearchMarker, { keywords: ['announcement', 'email'] }, {
                      default: _withCtx(() => [
                        _createVNode(MkSwitch, {
                          modelValue: _unref($i).receiveAnnouncementEmail,
                          "onUpdate:modelValue": onChangeReceiveAnnouncementEmail
                        }, {
                          label: _withCtx(() => [
                            _createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.receiveAnnouncementFromInstance), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["modelValue"])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["keywords"])
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(_component_SearchMarker, { keywords: ['notification', 'email'] }, {
                  default: _withCtx(() => [
                    _createVNode(FormSection, null, {
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.emailNotification), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      default: _withCtx(() => [
                        _createElementVNode("div", { class: "_gaps_s" }, [
                          _createVNode(MkSwitch, {
                            modelValue: emailNotification_mention.value,
                            "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((emailNotification_mention).value = $event))
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._notification._types.mention), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["modelValue"]),
                          _createVNode(MkSwitch, {
                            modelValue: emailNotification_reply.value,
                            "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((emailNotification_reply).value = $event))
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._notification._types.reply), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["modelValue"]),
                          _createVNode(MkSwitch, {
                            modelValue: emailNotification_quote.value,
                            "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((emailNotification_quote).value = $event))
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._notification._types.quote), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["modelValue"]),
                          _createVNode(MkSwitch, {
                            modelValue: emailNotification_follow.value,
                            "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((emailNotification_follow).value = $event))
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._notification._types.follow), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["modelValue"]),
                          _createVNode(MkSwitch, {
                            modelValue: emailNotification_receiveFollowRequest.value,
                            "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((emailNotification_receiveFollowRequest).value = $event))
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._notification._types.receiveFollowRequest), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["modelValue"])
                        ])
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"])
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["disabled"])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["label", "keywords"]))
}
}

})
