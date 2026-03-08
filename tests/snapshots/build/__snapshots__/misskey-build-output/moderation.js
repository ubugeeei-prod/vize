import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle", style: "color: var(--MI_THEME-warn);" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle", style: "color: var(--MI_THEME-warn);" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-lock-star" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-message-exclamation" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("br")
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-message-x" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("br")
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-user-x" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("br")
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye-off" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye-off" })
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye-off" })
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-ban" })
import { ref, computed } from 'vue'
import * as Misskey from 'misskey-js'
import XServerRules from './server-rules.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkInput from '@/components/MkInput.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { fetchInstance } from '@/instance.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { useMkSelect } from '@/composables/use-mkselect.js'
import MkButton from '@/components/MkButton.vue'
import FormLink from '@/components/form/link.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkSelect from '@/components/MkSelect.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'moderation',
  async setup(__props) {

let __temp: any, __restore: any

const meta = await misskeyApi('admin/meta');
const enableRegistration = ref(!meta.disableRegistration);
const emailRequiredForSignup = ref(meta.emailRequiredForSignup);
const {
	model: ugcVisibilityForVisitor,
	def: ugcVisibilityForVisitorDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts._serverSettings._userGeneratedContentsVisibilityForVisitor.all, value: 'all' },
		{ label: i18n.ts._serverSettings._userGeneratedContentsVisibilityForVisitor.localOnly, value: 'local' },
		{ label: i18n.ts._serverSettings._userGeneratedContentsVisibilityForVisitor.none, value: 'none' },
	],
	initialValue: meta.ugcVisibilityForVisitor,
});
const sensitiveWords = ref(meta.sensitiveWords.join('\n'));
const prohibitedWords = ref(meta.prohibitedWords.join('\n'));
const prohibitedWordsForNameOfUser = ref(meta.prohibitedWordsForNameOfUser.join('\n'));
const hiddenTags = ref(meta.hiddenTags.join('\n'));
const preservedUsernames = ref(meta.preservedUsernames.join('\n'));
const blockedHosts = ref(meta.blockedHosts.join('\n'));
const silencedHosts = ref(meta.silencedHosts?.join('\n') ?? '');
const mediaSilencedHosts = ref(meta.mediaSilencedHosts.join('\n'));
async function onChange_enableRegistration(value: boolean) {
	if (value) {
		const { canceled } = await os.confirm({
			type: 'warning',
			text: i18n.ts.acknowledgeNotesAndEnable,
		});
		if (canceled) return;
	}
	enableRegistration.value = value;
	os.apiWithDialog('admin/update-meta', {
		disableRegistration: !value,
	}).then(() => {
		fetchInstance(true);
	});
}
function onChange_emailRequiredForSignup(value: boolean) {
	os.apiWithDialog('admin/update-meta', {
		emailRequiredForSignup: value,
	}).then(() => {
		fetchInstance(true);
	});
}
function onChange_ugcVisibilityForVisitor(value: typeof ugcVisibilityForVisitor.value) {
	os.apiWithDialog('admin/update-meta', {
		ugcVisibilityForVisitor: value,
	}).then(() => {
		fetchInstance(true);
	});
}
function save_preservedUsernames() {
	os.apiWithDialog('admin/update-meta', {
		preservedUsernames: preservedUsernames.value.split('\n'),
	}).then(() => {
		fetchInstance(true);
	});
}
function save_sensitiveWords() {
	os.apiWithDialog('admin/update-meta', {
		sensitiveWords: sensitiveWords.value.split('\n'),
	}).then(() => {
		fetchInstance(true);
	});
}
function save_prohibitedWords() {
	os.apiWithDialog('admin/update-meta', {
		prohibitedWords: prohibitedWords.value.split('\n'),
	}).then(() => {
		fetchInstance(true);
	});
}
function save_prohibitedWordsForNameOfUser() {
	os.apiWithDialog('admin/update-meta', {
		prohibitedWordsForNameOfUser: prohibitedWordsForNameOfUser.value.split('\n'),
	}).then(() => {
		fetchInstance(true);
	});
}
function save_hiddenTags() {
	os.apiWithDialog('admin/update-meta', {
		hiddenTags: hiddenTags.value.split('\n'),
	}).then(() => {
		fetchInstance(true);
	});
}
function save_blockedHosts() {
	os.apiWithDialog('admin/update-meta', {
		blockedHosts: blockedHosts.value.split('\n') || [],
	}).then(() => {
		fetchInstance(true);
	});
}
function save_silencedHosts() {
	os.apiWithDialog('admin/update-meta', {
		silencedHosts: silencedHosts.value.split('\n') || [],
	}).then(() => {
		fetchInstance(true);
	});
}
function save_mediaSilencedHosts() {
	os.apiWithDialog('admin/update-meta', {
		mediaSilencedHosts: mediaSilencedHosts.value.split('\n') || [],
	}).then(() => {
		fetchInstance(true);
	});
}
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.moderation,
	icon: 'ti ti-shield',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_SearchText = _resolveComponent("SearchText")
  const _component_SearchMarker = _resolveComponent("SearchMarker")
  const _component_SearchIcon = _resolveComponent("SearchIcon")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, { tabs: headerTabs.value }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 700px; --MI_SPACER-min: 16px; --MI_SPACER-max: 32px;"
        }, [
          _createVNode(_component_SearchMarker, {
            path: "/admin/moderation",
            label: _unref(i18n).ts.moderation,
            keywords: ['moderation'],
            icon: "ti ti-shield",
            inlining: ['serverRules']
          }, {
            default: _withCtx(() => [
              _createElementVNode("div", { class: "_gaps_m" }, [
                _createVNode(_component_SearchMarker, { keywords: ['open', 'registration'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkSwitch, {
                      modelValue: enableRegistration.value,
                      "onUpdate:modelValue": onChange_enableRegistration
                    }, {
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSettings.openRegistration), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      caption: _withCtx(() => [
                        _createElementVNode("div", null, [
                          _createVNode(_component_SearchText, null, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSettings.thisSettingWillAutomaticallyOffWhenModeratorsInactive), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]),
                        _createElementVNode("div", null, [
                          _hoisted_1,
                          _createTextVNode(),
                          _createVNode(_component_SearchText, null, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSettings.openRegistrationWarning), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["modelValue"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(_component_SearchMarker, { keywords: ['email', 'required', 'signup'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkSwitch, {
                      onChange: onChange_emailRequiredForSignup,
                      modelValue: emailRequiredForSignup.value,
                      "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((emailRequiredForSignup).value = $event))
                    }, {
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.emailRequiredForSignup), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }),
                        _createTextVNode(" ("),
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.recommended), 1 /* TEXT */),
                        _createTextVNode(")")
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["modelValue"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(_component_SearchMarker, { keywords: ['ugc', 'content', 'visibility', 'visitor', 'guest'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkSelect, {
                      items: _unref(ugcVisibilityForVisitorDef),
                      "onUpdate:modelValue": [onChange_ugcVisibilityForVisitor, ($event: any) => ((ugcVisibilityForVisitor).value = $event)],
                      modelValue: _unref(ugcVisibilityForVisitor)
                    }, {
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSettings.userGeneratedContentsVisibilityForVisitor), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      caption: _withCtx(() => [
                        _createElementVNode("div", null, [
                          _createVNode(_component_SearchText, null, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSettings.userGeneratedContentsVisibilityForVisitor_description), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]),
                        _createElementVNode("div", null, [
                          _hoisted_2,
                          _createTextVNode(),
                          _createVNode(_component_SearchText, null, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSettings.userGeneratedContentsVisibilityForVisitor_description2), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["items", "modelValue"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(XServerRules),
                _createVNode(_component_SearchMarker, { keywords: ['preserved', 'usernames'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkFolder, null, {
                      icon: _withCtx(() => [
                        _createVNode(_component_SearchIcon, null, {
                          default: _withCtx(() => [
                            _hoisted_3
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.preservedUsernames), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      default: _withCtx(() => [
                        _createElementVNode("div", { class: "_gaps" }, [
                          _createVNode(MkTextarea, {
                            modelValue: preservedUsernames.value,
                            "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((preservedUsernames).value = $event))
                          }, {
                            caption: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.preservedUsernamesDescription), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["modelValue"]),
                          _createVNode(MkButton, {
                            primary: "",
                            onClick: save_preservedUsernames
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(_component_SearchMarker, { keywords: ['sensitive', 'words'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkFolder, null, {
                      icon: _withCtx(() => [
                        _createVNode(_component_SearchIcon, null, {
                          default: _withCtx(() => [
                            _hoisted_4
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.sensitiveWords), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      default: _withCtx(() => [
                        _createElementVNode("div", { class: "_gaps" }, [
                          _createVNode(MkTextarea, {
                            modelValue: sensitiveWords.value,
                            "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((sensitiveWords).value = $event))
                          }, {
                            caption: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.sensitiveWordsDescription), 1 /* TEXT */),
                              _hoisted_5,
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.sensitiveWordsDescription2), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["modelValue"]),
                          _createVNode(MkButton, {
                            primary: "",
                            onClick: save_sensitiveWords
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(_component_SearchMarker, { keywords: ['prohibited', 'words'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkFolder, null, {
                      icon: _withCtx(() => [
                        _createVNode(_component_SearchIcon, null, {
                          default: _withCtx(() => [
                            _hoisted_6
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.prohibitedWords), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      default: _withCtx(() => [
                        _createElementVNode("div", { class: "_gaps" }, [
                          _createVNode(MkTextarea, {
                            modelValue: prohibitedWords.value,
                            "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((prohibitedWords).value = $event))
                          }, {
                            caption: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.prohibitedWordsDescription), 1 /* TEXT */),
                              _hoisted_7,
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.prohibitedWordsDescription2), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["modelValue"]),
                          _createVNode(MkButton, {
                            primary: "",
                            onClick: save_prohibitedWords
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(_component_SearchMarker, { keywords: ['prohibited', 'name', 'user'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkFolder, null, {
                      icon: _withCtx(() => [
                        _createVNode(_component_SearchIcon, null, {
                          default: _withCtx(() => [
                            _hoisted_8
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.prohibitedWordsForNameOfUser), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      default: _withCtx(() => [
                        _createElementVNode("div", { class: "_gaps" }, [
                          _createVNode(MkTextarea, {
                            modelValue: prohibitedWordsForNameOfUser.value,
                            "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((prohibitedWordsForNameOfUser).value = $event))
                          }, {
                            caption: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.prohibitedWordsForNameOfUserDescription), 1 /* TEXT */),
                              _hoisted_9,
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.prohibitedWordsDescription2), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["modelValue"]),
                          _createVNode(MkButton, {
                            primary: "",
                            onClick: save_prohibitedWordsForNameOfUser
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(_component_SearchMarker, { keywords: ['hidden', 'tags', 'hashtags'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkFolder, null, {
                      icon: _withCtx(() => [
                        _createVNode(_component_SearchIcon, null, {
                          default: _withCtx(() => [
                            _hoisted_10
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.hiddenTags), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      default: _withCtx(() => [
                        _createElementVNode("div", { class: "_gaps" }, [
                          _createVNode(MkTextarea, {
                            modelValue: hiddenTags.value,
                            "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((hiddenTags).value = $event))
                          }, {
                            caption: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.hiddenTagsDescription), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["modelValue"]),
                          _createVNode(MkButton, {
                            primary: "",
                            onClick: save_hiddenTags
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(_component_SearchMarker, { keywords: ['silenced', 'servers', 'hosts'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkFolder, null, {
                      icon: _withCtx(() => [
                        _createVNode(_component_SearchIcon, null, {
                          default: _withCtx(() => [
                            _hoisted_11
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.silencedInstances), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      default: _withCtx(() => [
                        _createElementVNode("div", { class: "_gaps" }, [
                          _createVNode(MkTextarea, {
                            modelValue: silencedHosts.value,
                            "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((silencedHosts).value = $event))
                          }, {
                            caption: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.silencedInstancesDescription), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["modelValue"]),
                          _createVNode(MkButton, {
                            primary: "",
                            onClick: save_silencedHosts
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(_component_SearchMarker, { keywords: ['media', 'silenced', 'servers', 'hosts'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkFolder, null, {
                      icon: _withCtx(() => [
                        _createVNode(_component_SearchIcon, null, {
                          default: _withCtx(() => [
                            _hoisted_12
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.mediaSilencedInstances), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      default: _withCtx(() => [
                        _createElementVNode("div", { class: "_gaps" }, [
                          _createVNode(MkTextarea, {
                            modelValue: mediaSilencedHosts.value,
                            "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => ((mediaSilencedHosts).value = $event))
                          }, {
                            caption: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.mediaSilencedInstancesDescription), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["modelValue"]),
                          _createVNode(MkButton, {
                            primary: "",
                            onClick: save_mediaSilencedHosts
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(_component_SearchMarker, { keywords: ['blocked', 'servers', 'hosts'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkFolder, null, {
                      icon: _withCtx(() => [
                        _createVNode(_component_SearchIcon, null, {
                          default: _withCtx(() => [
                            _hoisted_13
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.blockedInstances), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      default: _withCtx(() => [
                        _createElementVNode("div", { class: "_gaps" }, [
                          _createVNode(MkTextarea, {
                            modelValue: blockedHosts.value,
                            "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event: any) => ((blockedHosts).value = $event))
                          }, {
                            caption: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.blockedInstancesDescription), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["modelValue"]),
                          _createVNode(MkButton, {
                            primary: "",
                            onClick: save_blockedHosts
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
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
          }, 8 /* PROPS */, ["label", "keywords", "inlining"])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["tabs"]))
}
}

})
