import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle", style: "color: var(--MI_THEME-warn);" })
import { ref, computed, watch } from 'vue'
import type { MkSelectItem } from '@/components/MkSelect.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkSelect from '@/components/MkSelect.vue'
import FormSection from '@/components/form/section.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { instance } from '@/instance.js'
import { ensureSignin } from '@/i.js'
import { definePage } from '@/page.js'
import FormSlot from '@/components/form/slot.vue'
import { formatDateTimeString } from '@/utility/format-time-string.js'
import { useMkSelect } from '@/composables/use-mkselect.js'
import MkInput from '@/components/MkInput.vue'
import * as os from '@/os.js'
import MkDisableSection from '@/components/MkDisableSection.vue'
import MkInfo from '@/components/MkInfo.vue'
import MkFeatureBanner from '@/components/MkFeatureBanner.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'privacy',
  setup(__props) {

const $i = ensureSignin();
const isLocked = ref($i.isLocked);
const autoAcceptFollowed = ref($i.autoAcceptFollowed);
const noCrawle = ref($i.noCrawle);
const preventAiLearning = ref($i.preventAiLearning);
const isExplorable = ref($i.isExplorable);
const requireSigninToViewContents = ref($i.requireSigninToViewContents ?? false);
const makeNotesFollowersOnlyBefore = ref($i.makeNotesFollowersOnlyBefore ?? null);
const makeNotesHiddenBefore = ref($i.makeNotesHiddenBefore ?? null);
const hideOnlineStatus = ref($i.hideOnlineStatus);
const publicReactions = ref($i.publicReactions);
const {
	model: followingVisibility,
	def: followingVisibilityDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts.public, value: 'public' },
		{ label: i18n.ts.followers, value: 'followers' },
		{ label: i18n.ts.private, value: 'private' },
	],
	initialValue: $i.followingVisibility,
});
const {
	model: followersVisibility,
	def: followersVisibilityDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts.public, value: 'public' },
		{ label: i18n.ts.followers, value: 'followers' },
		{ label: i18n.ts.private, value: 'private' },
	],
	initialValue: $i.followersVisibility,
});
const {
	model: chatScope,
	def: chatScopeDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts._chat._chatAllowedUsers.everyone, value: 'everyone' },
		{ label: i18n.ts._chat._chatAllowedUsers.followers, value: 'followers' },
		{ label: i18n.ts._chat._chatAllowedUsers.following, value: 'following' },
		{ label: i18n.ts._chat._chatAllowedUsers.mutual, value: 'mutual' },
		{ label: i18n.ts._chat._chatAllowedUsers.none, value: 'none' },
	],
	initialValue: $i.chatScope,
});
const makeNotesFollowersOnlyBefore_type = computed({
	get: () => {
		if (makeNotesFollowersOnlyBefore.value == null) {
			return null;
		} else if (makeNotesFollowersOnlyBefore.value >= 0) {
			return 'absolute';
		} else {
			return 'relative';
		}
	},
	set(value) {
		if (value === 'relative') {
			makeNotesFollowersOnlyBefore.value = -604800;
		} else if (value === 'absolute') {
			makeNotesFollowersOnlyBefore.value = Math.floor(Date.now() / 1000);
		} else {
			makeNotesFollowersOnlyBefore.value = null;
		}
	},
});
const makeNotesFollowersOnlyBefore_presets = [
	{ label: i18n.ts.oneHour, value: -3600 },
	{ label: i18n.ts.oneDay, value: -86400 },
	{ label: i18n.ts.threeDays, value: -259200 },
	{ label: i18n.ts.oneWeek, value: -604800 },
	{ label: i18n.ts.oneMonth, value: -2592000 },
	{ label: i18n.ts.threeMonths, value: -7776000 },
	{ label: i18n.ts.oneYear, value: -31104000 },
] satisfies MkSelectItem[];
const makeNotesFollowersOnlyBefore_isCustomMode = ref(
	makeNotesFollowersOnlyBefore.value != null &&
	makeNotesFollowersOnlyBefore.value < 0 &&
	!makeNotesFollowersOnlyBefore_presets.some((preset) => preset.value === makeNotesFollowersOnlyBefore.value),
);
const makeNotesFollowersOnlyBefore_selection = computed({
	get: () => makeNotesFollowersOnlyBefore_isCustomMode.value ? 'custom' : makeNotesFollowersOnlyBefore.value,
	set(value) {
		makeNotesFollowersOnlyBefore_isCustomMode.value = value === 'custom';
		if (value !== 'custom') makeNotesFollowersOnlyBefore.value = value;
	},
});
const makeNotesFollowersOnlyBefore_customMonths = computed({
	get: () => makeNotesFollowersOnlyBefore.value ? Math.abs(makeNotesFollowersOnlyBefore.value) / (30 * 24 * 60 * 60) : null,
	set(value) {
		if (value != null && value > 0) makeNotesFollowersOnlyBefore.value = -Math.abs(Math.floor(Number(value))) * 30 * 24 * 60 * 60;
	},
});
const makeNotesHiddenBefore_type = computed({
	get: () => {
		if (makeNotesHiddenBefore.value == null) {
			return null;
		} else if (makeNotesHiddenBefore.value >= 0) {
			return 'absolute';
		} else {
			return 'relative';
		}
	},
	set(value) {
		if (value === 'relative') {
			makeNotesHiddenBefore.value = -604800;
		} else if (value === 'absolute') {
			makeNotesHiddenBefore.value = Math.floor(Date.now() / 1000);
		} else {
			makeNotesHiddenBefore.value = null;
		}
	},
});
const makeNotesHiddenBefore_presets = [
	{ label: i18n.ts.oneHour, value: -3600 },
	{ label: i18n.ts.oneDay, value: -86400 },
	{ label: i18n.ts.threeDays, value: -259200 },
	{ label: i18n.ts.oneWeek, value: -604800 },
	{ label: i18n.ts.oneMonth, value: -2592000 },
	{ label: i18n.ts.threeMonths, value: -7776000 },
	{ label: i18n.ts.oneYear, value: -31104000 },
] satisfies MkSelectItem[];
const makeNotesHiddenBefore_isCustomMode = ref(
	makeNotesHiddenBefore.value != null &&
	makeNotesHiddenBefore.value < 0 &&
	!makeNotesHiddenBefore_presets.some((preset) => preset.value === makeNotesHiddenBefore.value),
);
const makeNotesHiddenBefore_selection = computed({
	get: () => makeNotesHiddenBefore_isCustomMode.value ? 'custom' : makeNotesHiddenBefore.value,
	set(value) {
		makeNotesHiddenBefore_isCustomMode.value = value === 'custom';
		if (value !== 'custom') makeNotesHiddenBefore.value = value;
	},
});
const makeNotesHiddenBefore_customMonths = computed({
	get: () => makeNotesHiddenBefore.value ? Math.abs(makeNotesHiddenBefore.value) / (30 * 24 * 60 * 60) : null,
	set(value) {
		if (value != null && value > 0) makeNotesHiddenBefore.value = -Math.abs(Math.floor(Number(value))) * 30 * 24 * 60 * 60;
	},
});
watch([makeNotesFollowersOnlyBefore, makeNotesHiddenBefore], () => {
	save();
});
async function update_requireSigninToViewContents(value: boolean) {
	if (value === true && instance.federation !== 'none') {
		const { canceled } = await os.confirm({
			type: 'warning',
			text: i18n.ts.acknowledgeNotesAndEnable,
		});
		if (canceled) return;
	}
	requireSigninToViewContents.value = value;
	save();
}
function save() {
	misskeyApi('i/update', {
		isLocked: !!isLocked.value,
		autoAcceptFollowed: !!autoAcceptFollowed.value,
		noCrawle: !!noCrawle.value,
		preventAiLearning: !!preventAiLearning.value,
		isExplorable: !!isExplorable.value,
		requireSigninToViewContents: !!requireSigninToViewContents.value,
		makeNotesFollowersOnlyBefore: makeNotesFollowersOnlyBefore.value,
		makeNotesHiddenBefore: makeNotesHiddenBefore.value,
		hideOnlineStatus: !!hideOnlineStatus.value,
		publicReactions: !!publicReactions.value,
		followingVisibility: followingVisibility.value,
		followersVisibility: followersVisibility.value,
		chatScope: chatScope.value,
	});
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.privacy,
	icon: 'ti ti-lock-open',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchText = _resolveComponent("SearchText")
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_SearchMarker = _resolveComponent("SearchMarker")

  return (_openBlock(), _createBlock(_component_SearchMarker, {
      path: "/settings/privacy",
      label: _unref(i18n).ts.privacy,
      keywords: ['privacy'],
      icon: "ti ti-lock-open"
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", { class: "_gaps_m" }, [
          _createVNode(MkFeatureBanner, {
            icon: "/client-assets/unlocked_3d.png",
            color: "#aeff00"
          }, {
            default: _withCtx(() => [
              _createVNode(_component_SearchText, null, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._settings.privacyBanner), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(_component_SearchMarker, { keywords: ['follow', 'lock'] }, {
            default: _withCtx(() => [
              _createVNode(MkSwitch, {
                "onUpdate:modelValue": [($event: any) => (save()), ($event: any) => ((isLocked).value = $event)],
                modelValue: isLocked.value
              }, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.makeFollowManuallyApprove), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                caption: _withCtx(() => [
                  _createVNode(_component_SearchText, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.lockedAccountInfo), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(MkDisableSection, { disabled: !isLocked.value }, {
            default: _withCtx(() => [
              _createVNode(_component_SearchMarker, { keywords: ['follow', 'auto', 'accept'] }, {
                default: _withCtx(() => [
                  _createVNode(MkSwitch, {
                    "onUpdate:modelValue": [($event: any) => (save()), ($event: any) => ((autoAcceptFollowed).value = $event)],
                    modelValue: autoAcceptFollowed.value
                  }, {
                    label: _withCtx(() => [
                      _createVNode(_component_SearchLabel, null, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.autoAcceptFollowed), 1 /* TEXT */)
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
          }, 8 /* PROPS */, ["disabled"]),
          _createVNode(_component_SearchMarker, { keywords: ['reaction', 'public'] }, {
            default: _withCtx(() => [
              _createVNode(MkSwitch, {
                "onUpdate:modelValue": [($event: any) => (save()), ($event: any) => ((publicReactions).value = $event)],
                modelValue: publicReactions.value
              }, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.makeReactionsPublic), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                caption: _withCtx(() => [
                  _createVNode(_component_SearchText, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.makeReactionsPublicDescription), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['following', 'visibility'] }, {
            default: _withCtx(() => [
              _createVNode(MkSelect, {
                items: _unref(followingVisibilityDef),
                "onUpdate:modelValue": [($event: any) => (save()), ($event: any) => ((followingVisibility).value = $event)],
                modelValue: _unref(followingVisibility)
              }, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.followingVisibility), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["items", "modelValue"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['follower', 'visibility'] }, {
            default: _withCtx(() => [
              _createVNode(MkSelect, {
                items: _unref(followersVisibilityDef),
                "onUpdate:modelValue": [($event: any) => (save()), ($event: any) => ((followersVisibility).value = $event)],
                modelValue: _unref(followersVisibility)
              }, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.followersVisibility), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["items", "modelValue"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['online', 'status'] }, {
            default: _withCtx(() => [
              _createVNode(MkSwitch, {
                "onUpdate:modelValue": [($event: any) => (save()), ($event: any) => ((hideOnlineStatus).value = $event)],
                modelValue: hideOnlineStatus.value
              }, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.hideOnlineStatus), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                caption: _withCtx(() => [
                  _createVNode(_component_SearchText, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.hideOnlineStatusDescription), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['crawle', 'index', 'search'] }, {
            default: _withCtx(() => [
              _createVNode(MkSwitch, {
                "onUpdate:modelValue": [($event: any) => (save()), ($event: any) => ((noCrawle).value = $event)],
                modelValue: noCrawle.value
              }, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.noCrawle), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                caption: _withCtx(() => [
                  _createVNode(_component_SearchText, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.noCrawleDescription), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['crawle', 'ai'] }, {
            default: _withCtx(() => [
              _createVNode(MkSwitch, {
                "onUpdate:modelValue": [($event: any) => (save()), ($event: any) => ((preventAiLearning).value = $event)],
                modelValue: preventAiLearning.value
              }, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.preventAiLearning), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                caption: _withCtx(() => [
                  _createVNode(_component_SearchText, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.preventAiLearningDescription), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['explore'] }, {
            default: _withCtx(() => [
              _createVNode(MkSwitch, {
                "onUpdate:modelValue": [($event: any) => (save()), ($event: any) => ((isExplorable).value = $event)],
                modelValue: isExplorable.value
              }, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.makeExplorable), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                caption: _withCtx(() => [
                  _createVNode(_component_SearchText, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.makeExplorableDescription), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['chat'] }, {
            default: _withCtx(() => [
              _createVNode(FormSection, null, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.directMessage), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps_m" }, [
                    (_unref($i).policies.chatAvailability === 'unavailable')
                      ? (_openBlock(), _createBlock(MkInfo, { key: 0 }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts._chat.chatNotAvailableForThisAccountOrServer), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }))
                      : _createCommentVNode("v-if", true),
                    _createVNode(_component_SearchMarker, { keywords: ['chat'] }, {
                      default: _withCtx(() => [
                        _createVNode(MkSelect, {
                          items: _unref(chatScopeDef),
                          "onUpdate:modelValue": [($event: any) => (save()), ($event: any) => ((chatScope).value = $event)],
                          modelValue: _unref(chatScope)
                        }, {
                          label: _withCtx(() => [
                            _createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts._chat.chatAllowedUsers), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          caption: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts._chat.chatAllowedUsers_note), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["items", "modelValue"])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["keywords"])
                  ])
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['lockdown'] }, {
            default: _withCtx(() => [
              _createVNode(FormSection, null, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.lockdown), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps_m" }, [
                    _createVNode(_component_SearchMarker, { keywords: ['login', 'signin'] }, {
                      default: _withCtx(() => [
                        _createVNode(MkSwitch, {
                          modelValue: requireSigninToViewContents.value,
                          "onUpdate:modelValue": update_requireSigninToViewContents
                        }, {
                          label: _withCtx(() => [
                            _createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts._accountSettings.requireSigninToViewContents), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          caption: _withCtx(() => [
                            _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._accountSettings.requireSigninToViewContentsDescription1), 1 /* TEXT */),
                            _createElementVNode("div", null, [
                              _hoisted_1,
                              _createTextVNode(" " + _toDisplayString(_unref(i18n).ts._accountSettings.requireSigninToViewContentsDescription2), 1 /* TEXT */)
                            ])
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["modelValue"])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: ['follower'] }, {
                      default: _withCtx(() => [
                        _createVNode(FormSlot, null, {
                          label: _withCtx(() => [
                            _createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts._accountSettings.makeNotesFollowersOnlyBefore), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          caption: _withCtx(() => [
                            _createElementVNode("div", null, [
                              _createVNode(_component_SearchText, null, {
                                default: _withCtx(() => [
                                  _createTextVNode(_toDisplayString(_unref(i18n).ts._accountSettings.makeNotesFollowersOnlyBeforeDescription), 1 /* TEXT */)
                                ]),
                                _: 1 /* STABLE */
                              })
                            ])
                          ]),
                          default: _withCtx(() => [
                            _createElementVNode("div", { class: "_gaps_s" }, [
                              _createVNode(MkSelect, {
                                items: [
  										{ label: _unref(i18n).ts.none, value: null },
  										{ label: _unref(i18n).ts._accountSettings.notesHavePassedSpecifiedPeriod, value: 'relative' },
  										{ label: _unref(i18n).ts._accountSettings.notesOlderThanSpecifiedDateAndTime, value: 'absolute' },
  									],
                                modelValue: makeNotesFollowersOnlyBefore_type.value,
                                "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((makeNotesFollowersOnlyBefore_type).value = $event))
                              }, null, 8 /* PROPS */, ["items", "modelValue"]),
                              (makeNotesFollowersOnlyBefore_type.value === 'relative')
                                ? (_openBlock(), _createBlock(MkSelect, {
                                  key: 0,
                                  items: [
  										..._unref(makeNotesFollowersOnlyBefore_presets),
  										{ label: _unref(i18n).ts.custom, value: 'custom' },
  									],
                                  modelValue: makeNotesFollowersOnlyBefore_selection.value,
                                  "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((makeNotesFollowersOnlyBefore_selection).value = $event))
                                }, null, 8 /* PROPS */, ["items", "modelValue"]))
                                : _createCommentVNode("v-if", true),
                              (makeNotesFollowersOnlyBefore_type.value === 'relative' && makeNotesFollowersOnlyBefore_isCustomMode.value)
                                ? (_openBlock(), _createBlock(MkInput, {
                                  key: 0,
                                  type: "number",
                                  min: 1,
                                  modelValue: makeNotesFollowersOnlyBefore_customMonths.value,
                                  "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((makeNotesFollowersOnlyBefore_customMonths).value = $event))
                                }, {
                                  suffix: _withCtx(() => [
                                    _createTextVNode(_toDisplayString(_unref(i18n).ts._time.month), 1 /* TEXT */)
                                  ]),
                                  _: 1 /* STABLE */
                                }, 8 /* PROPS */, ["min", "modelValue"]))
                                : _createCommentVNode("v-if", true),
                              (makeNotesFollowersOnlyBefore_type.value === 'absolute' && makeNotesFollowersOnlyBefore.value != null)
                                ? (_openBlock(), _createBlock(MkInput, {
                                  key: 0,
                                  modelValue: _unref(formatDateTimeString)(new Date(makeNotesFollowersOnlyBefore.value * 1000), 'yyyy-MM-dd'),
                                  type: "date",
                                  manualSave: true,
                                  "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => (makeNotesFollowersOnlyBefore.value = Math.floor(new Date($event).getTime() / 1000)))
                                }, null, 8 /* PROPS */, ["modelValue", "manualSave"]))
                                : _createCommentVNode("v-if", true)
                            ])
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: ['hidden'] }, {
                      default: _withCtx(() => [
                        _createVNode(FormSlot, null, {
                          label: _withCtx(() => [
                            _createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts._accountSettings.makeNotesHiddenBefore), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          caption: _withCtx(() => [
                            _createElementVNode("div", null, [
                              _createVNode(_component_SearchText, null, {
                                default: _withCtx(() => [
                                  _createTextVNode(_toDisplayString(_unref(i18n).ts._accountSettings.makeNotesHiddenBeforeDescription), 1 /* TEXT */)
                                ]),
                                _: 1 /* STABLE */
                              })
                            ])
                          ]),
                          default: _withCtx(() => [
                            _createElementVNode("div", { class: "_gaps_s" }, [
                              _createVNode(MkSelect, {
                                items: [
  										{ label: _unref(i18n).ts.none, value: null },
  										{ label: _unref(i18n).ts._accountSettings.notesHavePassedSpecifiedPeriod, value: 'relative' },
  										{ label: _unref(i18n).ts._accountSettings.notesOlderThanSpecifiedDateAndTime, value: 'absolute' },
  									],
                                modelValue: makeNotesHiddenBefore_type.value,
                                "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((makeNotesHiddenBefore_type).value = $event))
                              }, null, 8 /* PROPS */, ["items", "modelValue"]),
                              (makeNotesHiddenBefore_type.value === 'relative')
                                ? (_openBlock(), _createBlock(MkSelect, {
                                  key: 0,
                                  items: [
  										..._unref(makeNotesHiddenBefore_presets),
  										{ label: _unref(i18n).ts.custom, value: 'custom' },
  									],
                                  modelValue: makeNotesHiddenBefore_selection.value,
                                  "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((makeNotesHiddenBefore_selection).value = $event))
                                }, null, 8 /* PROPS */, ["items", "modelValue"]))
                                : _createCommentVNode("v-if", true),
                              (makeNotesHiddenBefore_type.value === 'relative' && makeNotesHiddenBefore_isCustomMode.value)
                                ? (_openBlock(), _createBlock(MkInput, {
                                  key: 0,
                                  type: "number",
                                  min: 1,
                                  modelValue: makeNotesHiddenBefore_customMonths.value,
                                  "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((makeNotesHiddenBefore_customMonths).value = $event))
                                }, {
                                  suffix: _withCtx(() => [
                                    _createTextVNode(_toDisplayString(_unref(i18n).ts._time.month), 1 /* TEXT */)
                                  ]),
                                  _: 1 /* STABLE */
                                }, 8 /* PROPS */, ["min", "modelValue"]))
                                : _createCommentVNode("v-if", true),
                              (makeNotesHiddenBefore_type.value === 'absolute' && makeNotesHiddenBefore.value != null)
                                ? (_openBlock(), _createBlock(MkInput, {
                                  key: 0,
                                  modelValue: _unref(formatDateTimeString)(new Date(makeNotesHiddenBefore.value * 1000), 'yyyy-MM-dd'),
                                  type: "date",
                                  manualSave: true,
                                  "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => (makeNotesHiddenBefore.value = Math.floor(new Date($event).getTime() / 1000)))
                                }, null, 8 /* PROPS */, ["modelValue", "manualSave"]))
                                : _createCommentVNode("v-if", true)
                            ])
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["keywords"]),
                    _createVNode(MkInfo, { warn: "" }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._accountSettings.mayNotEffectSomeSituations), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
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
    }, 8 /* PROPS */, ["label", "keywords"]))
}
}

})
