import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-info-circle" })
const _hoisted_2 = { class: "_monospace" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-badges" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-badges" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plane" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-flask" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-code" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("hr")
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-adjustments" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("hr")
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-bulb" })
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-bulb-off" })
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("hr")
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("hr")
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-refresh" })
import { computed, watch } from 'vue'
import XMigration from './migration.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import FormLink from '@/components/form/link.vue'
import MkFolder from '@/components/MkFolder.vue'
import FormInfo from '@/components/MkInfo.vue'
import MkKeyValue from '@/components/MkKeyValue.vue'
import MkButton from '@/components/MkButton.vue'
import FormSlot from '@/components/form/slot.vue'
import * as os from '@/os.js'
import { enableStoragePersistence, storagePersisted, storagePersistenceSupported } from '@/utility/storage.js'
import { ensureSignin } from '@/i.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import FormSection from '@/components/form/section.vue'
import { prefer } from '@/preferences.js'
import MkRolePreview from '@/components/MkRolePreview.vue'
import { signout } from '@/signout.js'
import { migrateOldSettings } from '@/pref-migrate.js'
import { hideAllTips as _hideAllTips, resetAllTips as _resetAllTips } from '@/tips.js'
import { suggestReload } from '@/utility/reload-suggest.js'
import { cloudBackup } from '@/preferences/utility.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'other',
  setup(__props) {

const $i = ensureSignin();
const reportError = prefer.model('reportError');
const enableCondensedLine = prefer.model('enableCondensedLine');
const skipNoteRender = prefer.model('skipNoteRender');
const devMode = prefer.model('devMode');
const stackingRouterView = prefer.model('experimental.stackingRouterView');
const enableFolderPageView = prefer.model('experimental.enableFolderPageView');
const enableHapticFeedback = prefer.model('experimental.enableHapticFeedback');
const enableWebTranslatorApi = prefer.model('experimental.enableWebTranslatorApi');
watch(skipNoteRender, () => {
	suggestReload();
});
async function deleteAccount() {
	{
		const { canceled } = await os.confirm({
			type: 'warning',
			text: i18n.ts.deleteAccountConfirm,
		});
		if (canceled) return;
	}
	const auth = await os.authenticateDialog();
	if (auth.canceled) return;
	await os.apiWithDialog('i/delete-account', {
		password: auth.result.password,
		token: auth.result.token,
	});
	await os.alert({
		title: i18n.ts._accountDelete.started,
	});
	await signout();
}
function migrate() {
	migrateOldSettings();
}
function resetAllTips() {
	_resetAllTips();
	os.success();
}
function hideAllTips() {
	_hideAllTips();
	os.success();
}
function readAllChatMessages() {
	os.apiWithDialog('chat/read-all', {});
}
async function forceCloudBackup() {
	await cloudBackup();
	os.success();
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.other,
	icon: 'ti ti-dots',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchIcon = _resolveComponent("SearchIcon")
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_SearchMarker = _resolveComponent("SearchMarker")
  const _component_SearchText = _resolveComponent("SearchText")

  return (_openBlock(), _createBlock(_component_SearchMarker, {
      path: "/settings/other",
      label: _unref(i18n).ts.other,
      keywords: ['other'],
      icon: "ti ti-dots"
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", { class: "_gaps_m" }, [
          _createElementVNode("div", { class: "_gaps_s" }, [
            _createVNode(_component_SearchMarker, { keywords: ['account', 'info'] }, {
              default: _withCtx(() => [
                _createVNode(MkFolder, null, {
                  icon: _withCtx(() => [
                    _createVNode(_component_SearchIcon, null, {
                      default: _withCtx(() => [
                        _hoisted_1
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  label: _withCtx(() => [
                    _createVNode(_component_SearchLabel, null, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.accountInfo), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  default: _withCtx(() => [
                    _createElementVNode("div", { class: "_gaps_m" }, [
                      _createVNode(MkKeyValue, null, {
                        key: _withCtx(() => [
                          _createTextVNode("ID")
                        ]),
                        value: _withCtx(() => [
                          _createElementVNode("span", _hoisted_2, _toDisplayString(_unref($i).id), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }),
                      _createVNode(MkKeyValue, null, {
                        key: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.registeredDate), 1 /* TEXT */)
                        ]),
                        value: _withCtx(() => [
                          _createVNode(_component_MkTime, {
                            time: _unref($i).createdAt,
                            mode: "detail"
                          }, null, 8 /* PROPS */, ["time"])
                        ]),
                        _: 1 /* STABLE */
                      }),
                      _createVNode(_component_SearchMarker, { keywords: ['role', 'policy'] }, {
                        default: _withCtx(() => [
                          _createVNode(MkFolder, null, {
                            icon: _withCtx(() => [
                              _hoisted_3
                            ]),
                            label: _withCtx(() => [
                              _createVNode(_component_SearchLabel, null, {
                                default: _withCtx(() => [
                                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role.policies), 1 /* TEXT */)
                                ]),
                                _: 1 /* STABLE */
                              })
                            ]),
                            default: _withCtx(() => [
                              _createElementVNode("div", { class: "_gaps_s" }, [
                                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(Object.keys(_unref($i).policies), (policy) => {
                                  return (_openBlock(), _createElementBlock("div", { key: policy }, _toDisplayString(policy) + " ... " + _toDisplayString(_unref($i).policies[policy]), 1 /* TEXT */))
                                }), 128 /* KEYED_FRAGMENT */))
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
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ['roles'] }, {
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
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.rolesAssignedToMe), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  default: _withCtx(() => [
                    _createElementVNode("div", { class: "_gaps_s" }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref($i).roles, (role) => {
                        return (_openBlock(), _createBlock(MkRolePreview, {
                          key: role.id,
                          role: role,
                          forModeration: false
                        }, null, 8 /* PROPS */, ["role", "forModeration"]))
                      }), 128 /* KEYED_FRAGMENT */))
                    ])
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ['account', 'move', 'migration'] }, {
              default: _withCtx(() => [
                _createVNode(MkFolder, null, {
                  icon: _withCtx(() => [
                    _createVNode(_component_SearchIcon, null, {
                      default: _withCtx(() => [
                        _hoisted_5
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  label: _withCtx(() => [
                    _createVNode(_component_SearchLabel, null, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.accountMigration), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  default: _withCtx(() => [
                    _createVNode(XMigration)
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ['account', 'close', 'delete'] }, {
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
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.closeAccount), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  default: _withCtx(() => [
                    _createElementVNode("div", { class: "_gaps_m" }, [
                      _createVNode(FormInfo, { warn: "" }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts._accountDelete.mayTakeTime), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }),
                      _createVNode(FormInfo, null, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts._accountDelete.sendEmail), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }),
                      (!_unref($i).isDeleted)
                        ? (_openBlock(), _createBlock(MkButton, {
                          key: 0,
                          danger: "",
                          onClick: deleteAccount
                        }, {
                          default: _withCtx(() => [
                            _createVNode(_component_SearchText, null, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts._accountDelete.requestAccountDelete), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          _: 1 /* STABLE */
                        }))
                        : (_openBlock(), _createBlock(MkButton, {
                          key: 1,
                          disabled: ""
                        }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts._accountDelete.inProgress), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }))
                    ])
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ['experimental', 'feature', 'flags'] }, {
              default: _withCtx(() => [
                _createVNode(MkFolder, null, {
                  icon: _withCtx(() => [
                    _createVNode(_component_SearchIcon, null, {
                      default: _withCtx(() => [
                        _hoisted_7
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  label: _withCtx(() => [
                    _createVNode(_component_SearchLabel, null, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.experimentalFeatures), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  default: _withCtx(() => [
                    _createElementVNode("div", { class: "_gaps_m" }, [
                      _createVNode(MkSwitch, {
                        modelValue: _unref(enableCondensedLine),
                        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((enableCondensedLine).value = $event))
                      }, {
                        label: _withCtx(() => [
                          _createTextVNode("Enable condensed line")
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["modelValue"]),
                      _createVNode(MkSwitch, {
                        modelValue: _unref(skipNoteRender),
                        "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((skipNoteRender).value = $event))
                      }, {
                        label: _withCtx(() => [
                          _createTextVNode("Enable note render skipping")
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["modelValue"]),
                      _createVNode(MkSwitch, {
                        modelValue: _unref(stackingRouterView),
                        "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((stackingRouterView).value = $event))
                      }, {
                        label: _withCtx(() => [
                          _createTextVNode("Enable stacking router view")
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["modelValue"]),
                      _createVNode(MkSwitch, {
                        modelValue: _unref(enableFolderPageView),
                        "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((enableFolderPageView).value = $event))
                      }, {
                        label: _withCtx(() => [
                          _createTextVNode("Enable folder page view")
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["modelValue"]),
                      _createVNode(MkSwitch, {
                        modelValue: _unref(enableHapticFeedback),
                        "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((enableHapticFeedback).value = $event))
                      }, {
                        label: _withCtx(() => [
                          _createTextVNode("Enable haptic feedback")
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["modelValue"]),
                      _createVNode(MkSwitch, {
                        modelValue: _unref(enableWebTranslatorApi),
                        "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((enableWebTranslatorApi).value = $event))
                      }, {
                        label: _withCtx(() => [
                          _createTextVNode("Enable in-browser translator API")
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["modelValue"])
                    ])
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ['developer', 'mode', 'debug'] }, {
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
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.developer), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  default: _withCtx(() => [
                    _createElementVNode("div", { class: "_gaps_m" }, [
                      _createVNode(MkSwitch, {
                        modelValue: _unref(devMode),
                        "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((devMode).value = $event))
                      }, {
                        label: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.devMode), 1 /* TEXT */)
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
          ]),
          _hoisted_9,
          _createVNode(FormLink, { to: "/registry" }, {
            icon: _withCtx(() => [
              _hoisted_10
            ]),
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.registry), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }),
          _hoisted_11,
          _createVNode(MkButton, { onClick: resetAllTips }, {
            default: _withCtx(() => [
              _hoisted_12,
              _createTextVNode(" "),
              _createTextVNode(_toDisplayString(_unref(i18n).ts.redisplayAllTips), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(MkButton, { onClick: hideAllTips }, {
            default: _withCtx(() => [
              _hoisted_13,
              _createTextVNode(" "),
              _createTextVNode(_toDisplayString(_unref(i18n).ts.hideAllTips), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }),
          _hoisted_14,
          (_unref($i).policies.chatAvailability !== 'unavailable')
            ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
              _createVNode(MkButton, { onClick: readAllChatMessages }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.readAllChatMessages), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }),
              _hoisted_15
            ], 64 /* STABLE_FRAGMENT */))
            : _createCommentVNode("v-if", true),
          (_unref(storagePersistenceSupported) && !_unref(storagePersisted))
            ? (_openBlock(), _createBlock(MkButton, {
              key: 0,
              onClick: _cache[7] || (_cache[7] = (...args) => (enableStoragePersistence && enableStoragePersistence(...args)))
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._settings.settingsPersistence_title), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }))
            : _createCommentVNode("v-if", true),
          _createVNode(MkButton, { onClick: forceCloudBackup }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._preferencesBackup.forceBackup), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(FormSlot, null, {
            caption: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.migrateOldSettings_description), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createVNode(MkButton, {
                danger: "",
                onClick: migrate
              }, {
                default: _withCtx(() => [
                  _hoisted_16,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.migrateOldSettings), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          })
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["label", "keywords"]))
}
}

})
