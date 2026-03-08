import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-star" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-star" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-users" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-upload" })
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-upload" })
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-users" })
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_17 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_18 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-upload" })
const _hoisted_19 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-upload" })
const _hoisted_20 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-user-off" })
const _hoisted_21 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_22 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_23 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-upload" })
const _hoisted_24 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-upload" })
const _hoisted_25 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-user-off" })
const _hoisted_26 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_27 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_28 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-upload" })
const _hoisted_29 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-upload" })
const _hoisted_30 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-antenna" })
const _hoisted_31 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_32 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_33 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-upload" })
const _hoisted_34 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-upload" })
import { ref, computed } from 'vue'
import MkButton from '@/components/MkButton.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { selectFile } from '@/utility/drive.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { $i } from '@/i.js'
import MkFeatureBanner from '@/components/MkFeatureBanner.vue'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'account-data',
  setup(__props) {

const excludeMutingUsers = ref(false);
const excludeInactiveUsers = ref(false);
const withReplies = ref(prefer.s.defaultFollowWithReplies);
const onExportSuccess = () => {
	os.alert({
		type: 'info',
		text: i18n.ts.exportRequested,
	});
};
const onImportSuccess = () => {
	os.alert({
		type: 'info',
		text: i18n.ts.importRequested,
	});
};
const onError = (ev: Error) => {
	os.alert({
		type: 'error',
		text: ev.message,
	});
};
const exportNotes = () => {
	misskeyApi('i/export-notes', {}).then(onExportSuccess).catch(onError);
};
const exportFavorites = () => {
	misskeyApi('i/export-favorites', {}).then(onExportSuccess).catch(onError);
};
const exportClips = () => {
	misskeyApi('i/export-clips', {}).then(onExportSuccess).catch(onError);
};
const exportFollowing = () => {
	misskeyApi('i/export-following', {
		excludeMuting: excludeMutingUsers.value,
		excludeInactive: excludeInactiveUsers.value,
	})
		.then(onExportSuccess).catch(onError);
};
const exportBlocking = () => {
	misskeyApi('i/export-blocking', {}).then(onExportSuccess).catch(onError);
};
const exportUserLists = () => {
	misskeyApi('i/export-user-lists', {}).then(onExportSuccess).catch(onError);
};
const exportMuting = () => {
	misskeyApi('i/export-mute', {}).then(onExportSuccess).catch(onError);
};
const exportAntennas = () => {
	misskeyApi('i/export-antennas', {}).then(onExportSuccess).catch(onError);
};
const importFollowing = async (ev: PointerEvent) => {
	const file = await selectFile({
		anchorElement: ev.currentTarget ?? ev.target,
		multiple: false,
	});
	misskeyApi('i/import-following', {
		fileId: file.id,
		withReplies: withReplies.value,
	}).then(onImportSuccess).catch(onError);
};
const importUserLists = async (ev: PointerEvent) => {
	const file = await selectFile({
		anchorElement: ev.currentTarget ?? ev.target,
		multiple: false,
	});
	misskeyApi('i/import-user-lists', { fileId: file.id }).then(onImportSuccess).catch(onError);
};
const importMuting = async (ev: PointerEvent) => {
	const file = await selectFile({
		anchorElement: ev.currentTarget ?? ev.target,
		multiple: false,
	});
	misskeyApi('i/import-muting', { fileId: file.id }).then(onImportSuccess).catch(onError);
};
const importBlocking = async (ev: PointerEvent) => {
	const file = await selectFile({
		anchorElement: ev.currentTarget ?? ev.target,
		multiple: false,
	});
	misskeyApi('i/import-blocking', { fileId: file.id }).then(onImportSuccess).catch(onError);
};
const importAntennas = async (ev: PointerEvent) => {
	const file = await selectFile({
		anchorElement: ev.currentTarget ?? ev.target,
		multiple: false,
	});
	misskeyApi('i/import-antennas', { fileId: file.id }).then(onImportSuccess).catch(onError);
};
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts._settings.accountData,
	icon: 'ti ti-package',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchText = _resolveComponent("SearchText")
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_SearchMarker = _resolveComponent("SearchMarker")

  return (_openBlock(), _createBlock(_component_SearchMarker, {
      path: "/settings/account-data",
      label: _unref(i18n).ts._settings.accountData,
      keywords: ['import', 'export', 'data', 'archive'],
      icon: "ti ti-package"
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", { class: "_gaps_m" }, [
          _createVNode(MkFeatureBanner, {
            icon: "/client-assets/package_3d.png",
            color: "#ff9100"
          }, {
            default: _withCtx(() => [
              _createVNode(_component_SearchText, null, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._settings.accountDataBanner), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }),
          _createElementVNode("div", { class: "_gaps_s" }, [
            _createVNode(_component_SearchMarker, { keywords: ['notes'] }, {
              default: _withCtx(() => [
                _createVNode(MkFolder, null, {
                  icon: _withCtx(() => [
                    _hoisted_1
                  ]),
                  label: _withCtx(() => [
                    _createVNode(_component_SearchLabel, null, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._exportOrImport.allNotes), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  default: _withCtx(() => [
                    _createVNode(MkFolder, { defaultOpen: true }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.export), 1 /* TEXT */)
                      ]),
                      icon: _withCtx(() => [
                        _hoisted_2
                      ]),
                      default: _withCtx(() => [
                        _createVNode(MkButton, {
                          primary: "",
                          class: _normalizeClass(_ctx.$style.button),
                          inline: "",
                          onClick: _cache[0] || (_cache[0] = ($event: any) => (exportNotes()))
                        }, {
                          default: _withCtx(() => [
                            _hoisted_3,
                            _createTextVNode(" "),
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.export), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["defaultOpen"])
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ['favorite', 'notes'] }, {
              default: _withCtx(() => [
                _createVNode(MkFolder, null, {
                  icon: _withCtx(() => [
                    _hoisted_4
                  ]),
                  label: _withCtx(() => [
                    _createVNode(_component_SearchLabel, null, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._exportOrImport.favoritedNotes), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  default: _withCtx(() => [
                    _createVNode(MkFolder, { defaultOpen: true }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.export), 1 /* TEXT */)
                      ]),
                      icon: _withCtx(() => [
                        _hoisted_5
                      ]),
                      default: _withCtx(() => [
                        _createVNode(MkButton, {
                          primary: "",
                          class: _normalizeClass(_ctx.$style.button),
                          inline: "",
                          onClick: _cache[1] || (_cache[1] = ($event: any) => (exportFavorites()))
                        }, {
                          default: _withCtx(() => [
                            _hoisted_6,
                            _createTextVNode(" "),
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.export), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["defaultOpen"])
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ['clip', 'notes'] }, {
              default: _withCtx(() => [
                _createVNode(MkFolder, null, {
                  icon: _withCtx(() => [
                    _hoisted_7
                  ]),
                  label: _withCtx(() => [
                    _createVNode(_component_SearchLabel, null, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._exportOrImport.clips), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  default: _withCtx(() => [
                    _createVNode(MkFolder, { defaultOpen: true }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.export), 1 /* TEXT */)
                      ]),
                      icon: _withCtx(() => [
                        _hoisted_8
                      ]),
                      default: _withCtx(() => [
                        _createVNode(MkButton, {
                          primary: "",
                          class: _normalizeClass(_ctx.$style.button),
                          inline: "",
                          onClick: _cache[2] || (_cache[2] = ($event: any) => (exportClips()))
                        }, {
                          default: _withCtx(() => [
                            _hoisted_9,
                            _createTextVNode(" "),
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.export), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["defaultOpen"])
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ['following', 'users'] }, {
              default: _withCtx(() => [
                _createVNode(MkFolder, null, {
                  icon: _withCtx(() => [
                    _hoisted_10
                  ]),
                  label: _withCtx(() => [
                    _createVNode(_component_SearchLabel, null, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._exportOrImport.followingList), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  default: _withCtx(() => [
                    _createElementVNode("div", { class: "_gaps_s" }, [
                      _createVNode(MkFolder, { defaultOpen: true }, {
                        label: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.export), 1 /* TEXT */)
                        ]),
                        icon: _withCtx(() => [
                          _hoisted_11
                        ]),
                        default: _withCtx(() => [
                          _createElementVNode("div", { class: "_gaps_s" }, [
                            _createVNode(MkSwitch, {
                              modelValue: excludeMutingUsers.value,
                              "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((excludeMutingUsers).value = $event))
                            }, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts._exportOrImport.excludeMutingUsers), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            }, 8 /* PROPS */, ["modelValue"]),
                            _createVNode(MkSwitch, {
                              modelValue: excludeInactiveUsers.value,
                              "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((excludeInactiveUsers).value = $event))
                            }, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts._exportOrImport.excludeInactiveUsers), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            }, 8 /* PROPS */, ["modelValue"]),
                            _createVNode(MkButton, {
                              primary: "",
                              class: _normalizeClass(_ctx.$style.button),
                              inline: "",
                              onClick: _cache[5] || (_cache[5] = ($event: any) => (exportFollowing()))
                            }, {
                              default: _withCtx(() => [
                                _hoisted_12,
                                _createTextVNode(" "),
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.export), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ])
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["defaultOpen"]),
                      (_unref($i) && !_unref($i).movedTo && _unref($i).policies.canImportFollowing)
                        ? (_openBlock(), _createBlock(MkFolder, {
                          key: 0,
                          defaultOpen: true
                        }, {
                          label: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.import), 1 /* TEXT */)
                          ]),
                          icon: _withCtx(() => [
                            _hoisted_13
                          ]),
                          default: _withCtx(() => [
                            _createVNode(MkSwitch, {
                              modelValue: withReplies.value,
                              "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((withReplies).value = $event))
                            }, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts._exportOrImport.withReplies), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            }, 8 /* PROPS */, ["modelValue"]),
                            _createVNode(MkButton, {
                              primary: "",
                              class: _normalizeClass(_ctx.$style.button),
                              inline: "",
                              onClick: _cache[7] || (_cache[7] = ($event: any) => (importFollowing($event)))
                            }, {
                              default: _withCtx(() => [
                                _hoisted_14,
                                _createTextVNode(" "),
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.import), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["defaultOpen"]))
                        : _createCommentVNode("v-if", true)
                    ])
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ['user', 'lists'] }, {
              default: _withCtx(() => [
                _createVNode(MkFolder, null, {
                  icon: _withCtx(() => [
                    _hoisted_15
                  ]),
                  label: _withCtx(() => [
                    _createVNode(_component_SearchLabel, null, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._exportOrImport.userLists), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  default: _withCtx(() => [
                    _createElementVNode("div", { class: "_gaps_s" }, [
                      _createVNode(MkFolder, { defaultOpen: true }, {
                        label: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.export), 1 /* TEXT */)
                        ]),
                        icon: _withCtx(() => [
                          _hoisted_16
                        ]),
                        default: _withCtx(() => [
                          _createVNode(MkButton, {
                            primary: "",
                            class: _normalizeClass(_ctx.$style.button),
                            inline: "",
                            onClick: _cache[8] || (_cache[8] = ($event: any) => (exportUserLists()))
                          }, {
                            default: _withCtx(() => [
                              _hoisted_17,
                              _createTextVNode(" "),
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.export), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["defaultOpen"]),
                      (_unref($i) && !_unref($i).movedTo && _unref($i).policies.canImportUserLists)
                        ? (_openBlock(), _createBlock(MkFolder, {
                          key: 0,
                          defaultOpen: true
                        }, {
                          label: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.import), 1 /* TEXT */)
                          ]),
                          icon: _withCtx(() => [
                            _hoisted_18
                          ]),
                          default: _withCtx(() => [
                            _createVNode(MkButton, {
                              primary: "",
                              class: _normalizeClass(_ctx.$style.button),
                              inline: "",
                              onClick: _cache[9] || (_cache[9] = ($event: any) => (importUserLists($event)))
                            }, {
                              default: _withCtx(() => [
                                _hoisted_19,
                                _createTextVNode(" "),
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.import), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["defaultOpen"]))
                        : _createCommentVNode("v-if", true)
                    ])
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ['mute', 'users'] }, {
              default: _withCtx(() => [
                _createVNode(MkFolder, null, {
                  icon: _withCtx(() => [
                    _hoisted_20
                  ]),
                  label: _withCtx(() => [
                    _createVNode(_component_SearchLabel, null, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._exportOrImport.muteList), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  default: _withCtx(() => [
                    _createElementVNode("div", { class: "_gaps_s" }, [
                      _createVNode(MkFolder, { defaultOpen: true }, {
                        label: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.export), 1 /* TEXT */)
                        ]),
                        icon: _withCtx(() => [
                          _hoisted_21
                        ]),
                        default: _withCtx(() => [
                          _createVNode(MkButton, {
                            primary: "",
                            class: _normalizeClass(_ctx.$style.button),
                            inline: "",
                            onClick: _cache[10] || (_cache[10] = ($event: any) => (exportMuting()))
                          }, {
                            default: _withCtx(() => [
                              _hoisted_22,
                              _createTextVNode(" "),
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.export), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["defaultOpen"]),
                      (_unref($i) && !_unref($i).movedTo && _unref($i).policies.canImportMuting)
                        ? (_openBlock(), _createBlock(MkFolder, {
                          key: 0,
                          defaultOpen: true
                        }, {
                          label: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.import), 1 /* TEXT */)
                          ]),
                          icon: _withCtx(() => [
                            _hoisted_23
                          ]),
                          default: _withCtx(() => [
                            _createVNode(MkButton, {
                              primary: "",
                              class: _normalizeClass(_ctx.$style.button),
                              inline: "",
                              onClick: _cache[11] || (_cache[11] = ($event: any) => (importMuting($event)))
                            }, {
                              default: _withCtx(() => [
                                _hoisted_24,
                                _createTextVNode(" "),
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.import), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["defaultOpen"]))
                        : _createCommentVNode("v-if", true)
                    ])
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ['block', 'users'] }, {
              default: _withCtx(() => [
                _createVNode(MkFolder, null, {
                  icon: _withCtx(() => [
                    _hoisted_25
                  ]),
                  label: _withCtx(() => [
                    _createVNode(_component_SearchLabel, null, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._exportOrImport.blockingList), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  default: _withCtx(() => [
                    _createElementVNode("div", { class: "_gaps_s" }, [
                      _createVNode(MkFolder, { defaultOpen: true }, {
                        label: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.export), 1 /* TEXT */)
                        ]),
                        icon: _withCtx(() => [
                          _hoisted_26
                        ]),
                        default: _withCtx(() => [
                          _createVNode(MkButton, {
                            primary: "",
                            class: _normalizeClass(_ctx.$style.button),
                            inline: "",
                            onClick: _cache[12] || (_cache[12] = ($event: any) => (exportBlocking()))
                          }, {
                            default: _withCtx(() => [
                              _hoisted_27,
                              _createTextVNode(" "),
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.export), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["defaultOpen"]),
                      (_unref($i) && !_unref($i).movedTo && _unref($i).policies.canImportBlocking)
                        ? (_openBlock(), _createBlock(MkFolder, {
                          key: 0,
                          defaultOpen: true
                        }, {
                          label: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.import), 1 /* TEXT */)
                          ]),
                          icon: _withCtx(() => [
                            _hoisted_28
                          ]),
                          default: _withCtx(() => [
                            _createVNode(MkButton, {
                              primary: "",
                              class: _normalizeClass(_ctx.$style.button),
                              inline: "",
                              onClick: _cache[13] || (_cache[13] = ($event: any) => (importBlocking($event)))
                            }, {
                              default: _withCtx(() => [
                                _hoisted_29,
                                _createTextVNode(" "),
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.import), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["defaultOpen"]))
                        : _createCommentVNode("v-if", true)
                    ])
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ['antennas'] }, {
              default: _withCtx(() => [
                _createVNode(MkFolder, null, {
                  icon: _withCtx(() => [
                    _hoisted_30
                  ]),
                  label: _withCtx(() => [
                    _createVNode(_component_SearchLabel, null, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.antennas), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  default: _withCtx(() => [
                    _createElementVNode("div", { class: "_gaps_s" }, [
                      _createVNode(MkFolder, { defaultOpen: true }, {
                        label: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.export), 1 /* TEXT */)
                        ]),
                        icon: _withCtx(() => [
                          _hoisted_31
                        ]),
                        default: _withCtx(() => [
                          _createVNode(MkButton, {
                            primary: "",
                            class: _normalizeClass(_ctx.$style.button),
                            inline: "",
                            onClick: _cache[14] || (_cache[14] = ($event: any) => (exportAntennas()))
                          }, {
                            default: _withCtx(() => [
                              _hoisted_32,
                              _createTextVNode(" "),
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.export), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["defaultOpen"]),
                      (_unref($i) && !_unref($i).movedTo && _unref($i).policies.canImportAntennas)
                        ? (_openBlock(), _createBlock(MkFolder, {
                          key: 0,
                          defaultOpen: true
                        }, {
                          label: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.import), 1 /* TEXT */)
                          ]),
                          icon: _withCtx(() => [
                            _hoisted_33
                          ]),
                          default: _withCtx(() => [
                            _createVNode(MkButton, {
                              primary: "",
                              class: _normalizeClass(_ctx.$style.button),
                              inline: "",
                              onClick: _cache[15] || (_cache[15] = ($event: any) => (importAntennas($event)))
                            }, {
                              default: _withCtx(() => [
                                _hoisted_34,
                                _createTextVNode(" "),
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.import), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["defaultOpen"]))
                        : _createCommentVNode("v-if", true)
                    ])
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["keywords"])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["label", "keywords"]))
}
}

})
