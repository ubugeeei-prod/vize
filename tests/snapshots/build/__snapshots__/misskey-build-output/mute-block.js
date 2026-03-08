import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-message-off" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-message-off" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-mood-off" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-planet-off" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-repeat-off" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye-off" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-ban" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
import { ref, computed, watch, markRaw } from 'vue'
import * as Misskey from 'misskey-js'
import XEmojiMute from './mute-block.emoji-mute.vue'
import XInstanceMute from './mute-block.instance-mute.vue'
import XWordMute from './mute-block.word-mute.vue'
import MkPagination from '@/components/MkPagination.vue'
import { userPage } from '@/filters/user.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import MkUserCardMini from '@/components/MkUserCardMini.vue'
import * as os from '@/os.js'
import { instance } from '@/instance.js'
import { ensureSignin } from '@/i.js'
import MkInfo from '@/components/MkInfo.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import { prefer } from '@/preferences.js'
import MkFeatureBanner from '@/components/MkFeatureBanner.vue'
import { Paginator } from '@/utility/paginator.js'
import { suggestReload } from '@/utility/reload-suggest.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'mute-block',
  setup(__props) {

const $i = ensureSignin();
const renoteMutingPaginator = markRaw(new Paginator('renote-mute/list', {
	limit: 10,
}));
const mutingPaginator = markRaw(new Paginator('mute/list', {
	limit: 10,
}));
const blockingPaginator = markRaw(new Paginator('blocking/list', {
	limit: 10,
}));
const expandedRenoteMuteItems = ref<string[]>([]);
const expandedMuteItems = ref<string[]>([]);
const expandedBlockItems = ref<string[]>([]);
const showSoftWordMutedWord = prefer.model('showSoftWordMutedWord');
watch([
	showSoftWordMutedWord,
], () => {
	suggestReload();
});
async function unrenoteMute(user: Misskey.entities.UserDetailed, ev: PointerEvent) {
	os.popupMenu([{
		text: i18n.ts.renoteUnmute,
		icon: 'ti ti-x',
		action: async () => {
			await os.apiWithDialog('renote-mute/delete', { userId: user.id });
			//role.users = role.users.filter(u => u.id !== user.id);
		},
	}], ev.currentTarget ?? ev.target);
}
async function unmute(user: Misskey.entities.UserDetailed, ev: PointerEvent) {
	os.popupMenu([{
		text: i18n.ts.unmute,
		icon: 'ti ti-x',
		action: async () => {
			await os.apiWithDialog('mute/delete', { userId: user.id });
			//role.users = role.users.filter(u => u.id !== user.id);
		},
	}], ev.currentTarget ?? ev.target);
}
async function unblock(user: Misskey.entities.UserDetailed, ev: PointerEvent) {
	os.popupMenu([{
		text: i18n.ts.unblock,
		icon: 'ti ti-x',
		action: async () => {
			await os.apiWithDialog('blocking/delete', { userId: user.id });
			//role.users = role.users.filter(u => u.id !== user.id);
		},
	}], ev.currentTarget ?? ev.target);
}
async function toggleRenoteMuteItem(item: { id: string }) {
	if (expandedRenoteMuteItems.value.includes(item.id)) {
		expandedRenoteMuteItems.value = expandedRenoteMuteItems.value.filter(x => x !== item.id);
	} else {
		expandedRenoteMuteItems.value.push(item.id);
	}
}
async function toggleMuteItem(item: { id: string }) {
	if (expandedMuteItems.value.includes(item.id)) {
		expandedMuteItems.value = expandedMuteItems.value.filter(x => x !== item.id);
	} else {
		expandedMuteItems.value.push(item.id);
	}
}
async function toggleBlockItem(item: { id: string }) {
	if (expandedBlockItems.value.includes(item.id)) {
		expandedBlockItems.value = expandedBlockItems.value.filter(x => x !== item.id);
	} else {
		expandedBlockItems.value.push(item.id);
	}
}
async function saveMutedWords(mutedWords: (string | string[])[]) {
	await os.apiWithDialog('i/update', { mutedWords });
}
async function saveHardMutedWords(hardMutedWords: (string | string[])[]) {
	await os.apiWithDialog('i/update', { hardMutedWords });
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.muteAndBlock,
	icon: 'ti ti-ban',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchText = _resolveComponent("SearchText")
  const _component_SearchMarker = _resolveComponent("SearchMarker")
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_MkResult = _resolveComponent("MkResult")
  const _component_MkA = _resolveComponent("MkA")
  const _component_MkTime = _resolveComponent("MkTime")

  return (_openBlock(), _createBlock(_component_SearchMarker, {
      path: "/settings/mute-block",
      label: _unref(i18n).ts.muteAndBlock,
      icon: "ti ti-ban",
      keywords: ['mute', 'block']
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", { class: "_gaps_m" }, [
          _createVNode(MkFeatureBanner, {
            icon: "/client-assets/prohibited_3d.png",
            color: "#ff2600"
          }, {
            default: _withCtx(() => [
              _createVNode(_component_SearchText, null, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._settings.muteAndBlockBanner), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }),
          _createElementVNode("div", { class: "_gaps_s" }, [
            _createVNode(_component_SearchMarker, {
              label: _unref(i18n).ts.wordMute,
              keywords: ['note', 'word', 'soft', 'mute', 'hide']
            }, {
              default: _withCtx(() => [
                _createVNode(MkFolder, null, {
                  icon: _withCtx(() => [
                    _hoisted_1
                  ]),
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.wordMute), 1 /* TEXT */)
                  ]),
                  default: _withCtx(() => [
                    _createElementVNode("div", { class: "_gaps_m" }, [
                      _createVNode(MkInfo, null, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.wordMuteDescription), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }),
                      _createVNode(_component_SearchMarker, {
                        label: _unref(i18n).ts.showMutedWord,
                        keywords: ['show']
                      }, {
                        default: _withCtx(() => [
                          _createVNode(MkSwitch, {
                            modelValue: _unref(showSoftWordMutedWord),
                            "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((showSoftWordMutedWord).value = $event))
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.showMutedWord), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["modelValue"])
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["label", "keywords"]),
                      _createVNode(XWordMute, {
                        muted: _unref($i).mutedWords,
                        onSave: saveMutedWords
                      }, null, 8 /* PROPS */, ["muted"])
                    ])
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["label", "keywords"]),
            _createVNode(_component_SearchMarker, {
              label: _unref(i18n).ts.hardWordMute,
              keywords: ['note', 'word', 'hard', 'mute', 'hide']
            }, {
              default: _withCtx(() => [
                _createVNode(MkFolder, null, {
                  icon: _withCtx(() => [
                    _hoisted_2
                  ]),
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.hardWordMute), 1 /* TEXT */)
                  ]),
                  default: _withCtx(() => [
                    _createElementVNode("div", { class: "_gaps_m" }, [
                      _createVNode(MkInfo, null, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.hardWordMuteDescription), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }),
                      _createVNode(XWordMute, {
                        muted: _unref($i).hardMutedWords,
                        onSave: saveHardMutedWords
                      }, null, 8 /* PROPS */, ["muted"])
                    ])
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["label", "keywords"]),
            _createVNode(_component_SearchMarker, {
              label: _unref(i18n).ts.emojiMute,
              keywords: ['emoji', 'mute', 'hide']
            }, {
              default: _withCtx(() => [
                _createVNode(MkFolder, null, {
                  icon: _withCtx(() => [
                    _hoisted_3
                  ]),
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.emojiMute), 1 /* TEXT */)
                  ]),
                  default: _withCtx(() => [
                    _createVNode(XEmojiMute)
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["label", "keywords"]),
            _createVNode(_component_SearchMarker, {
              label: _unref(i18n).ts.instanceMute,
              keywords: ['note', 'server', 'instance', 'host', 'federation', 'mute', 'hide']
            }, {
              default: _withCtx(() => [
                (_unref(instance).federation !== 'none')
                  ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                    icon: _withCtx(() => [
                      _hoisted_4
                    ]),
                    label: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.instanceMute), 1 /* TEXT */)
                    ]),
                    default: _withCtx(() => [
                      _createVNode(XInstanceMute)
                    ]),
                    _: 1 /* STABLE */
                  }))
                  : _createCommentVNode("v-if", true)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["label", "keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ['renote', 'mute', 'hide', 'user'] }, {
              default: _withCtx(() => [
                _createVNode(MkFolder, null, {
                  icon: _withCtx(() => [
                    _hoisted_5
                  ]),
                  label: _withCtx(() => [
                    _createVNode(_component_SearchLabel, null, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.mutedUsers), 1 /* TEXT */),
                        _createTextVNode(" ("),
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.renote), 1 /* TEXT */),
                        _createTextVNode(")")
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  default: _withCtx(() => [
                    _createVNode(MkPagination, {
                      paginator: _unref(renoteMutingPaginator),
                      withControl: ""
                    }, {
                      empty: _withCtx(() => [
                        _createVNode(_component_MkResult, {
                          type: "empty",
                          text: _unref(i18n).ts.noUsers
                        }, null, 8 /* PROPS */, ["text"])
                      ]),
                      default: _withCtx(({ items }) => [
                        _createElementVNode("div", { class: "_gaps_s" }, [
                          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
                            return (_openBlock(), _createElementBlock("div", {
                              key: item.mutee.id,
                              class: _normalizeClass([_ctx.$style.userItem, { [_ctx.$style.userItemOpend]: expandedRenoteMuteItems.value.includes(item.id) }])
                            }, [
                              _createElementVNode("div", {
                                class: _normalizeClass(_ctx.$style.userItemMain)
                              }, [
                                _createVNode(_component_MkA, {
                                  class: _normalizeClass(_ctx.$style.userItemMainBody),
                                  to: _unref(userPage)(item.mutee)
                                }, {
                                  default: _withCtx(() => [
                                    _createVNode(MkUserCardMini, { user: item.mutee }, null, 8 /* PROPS */, ["user"])
                                  ]),
                                  _: 2 /* DYNAMIC */
                                }, 8 /* PROPS */, ["to"]),
                                _createElementVNode("button", {
                                  class: _normalizeClass(["_button", _ctx.$style.userToggle]),
                                  onClick: _cache[1] || (_cache[1] = ($event: any) => (toggleRenoteMuteItem(item)))
                                }, [
                                  _createElementVNode("i", {
                                    class: _normalizeClass(["ti ti-chevron-down", _ctx.$style.chevron])
                                  })
                                ]),
                                _createElementVNode("button", {
                                  class: _normalizeClass(["_button", _ctx.$style.remove]),
                                  onClick: _cache[2] || (_cache[2] = ($event: any) => (unrenoteMute(item.mutee, $event)))
                                }, [
                                  _hoisted_6
                                ])
                              ]),
                              (expandedRenoteMuteItems.value.includes(item.id))
                                ? (_openBlock(), _createElementBlock("div", {
                                  key: 0,
                                  class: _normalizeClass(_ctx.$style.userItemSub)
                                }, [
                                  _createElementVNode("div", null, [
                                    _createTextVNode("Muted at: "),
                                    _createVNode(_component_MkTime, {
                                      time: item.createdAt,
                                      mode: "detail"
                                    }, null, 8 /* PROPS */, ["time"])
                                  ])
                                ]))
                                : _createCommentVNode("v-if", true)
                            ], 2 /* CLASS */))
                          }), 128 /* KEYED_FRAGMENT */))
                        ])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["paginator"])
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["keywords"]),
            _createVNode(_component_SearchMarker, {
              label: _unref(i18n).ts.mutedUsers,
              keywords: ['note', 'mute', 'hide', 'user']
            }, {
              default: _withCtx(() => [
                _createVNode(MkFolder, null, {
                  icon: _withCtx(() => [
                    _hoisted_7
                  ]),
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.mutedUsers), 1 /* TEXT */)
                  ]),
                  default: _withCtx(() => [
                    _createVNode(MkPagination, {
                      paginator: _unref(mutingPaginator),
                      withControl: ""
                    }, {
                      empty: _withCtx(() => [
                        _createVNode(_component_MkResult, {
                          type: "empty",
                          text: _unref(i18n).ts.noUsers
                        }, null, 8 /* PROPS */, ["text"])
                      ]),
                      default: _withCtx(({ items }) => [
                        _createElementVNode("div", { class: "_gaps_s" }, [
                          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
                            return (_openBlock(), _createElementBlock("div", {
                              key: item.mutee.id,
                              class: _normalizeClass([_ctx.$style.userItem, { [_ctx.$style.userItemOpend]: expandedMuteItems.value.includes(item.id) }])
                            }, [
                              _createElementVNode("div", {
                                class: _normalizeClass(_ctx.$style.userItemMain)
                              }, [
                                _createVNode(_component_MkA, {
                                  class: _normalizeClass(_ctx.$style.userItemMainBody),
                                  to: _unref(userPage)(item.mutee)
                                }, {
                                  default: _withCtx(() => [
                                    _createVNode(MkUserCardMini, { user: item.mutee }, null, 8 /* PROPS */, ["user"])
                                  ]),
                                  _: 2 /* DYNAMIC */
                                }, 8 /* PROPS */, ["to"]),
                                _createElementVNode("button", {
                                  class: _normalizeClass(["_button", _ctx.$style.userToggle]),
                                  onClick: _cache[3] || (_cache[3] = ($event: any) => (toggleMuteItem(item)))
                                }, [
                                  _createElementVNode("i", {
                                    class: _normalizeClass(["ti ti-chevron-down", _ctx.$style.chevron])
                                  })
                                ]),
                                _createElementVNode("button", {
                                  class: _normalizeClass(["_button", _ctx.$style.remove]),
                                  onClick: _cache[4] || (_cache[4] = ($event: any) => (unmute(item.mutee, $event)))
                                }, [
                                  _hoisted_8
                                ])
                              ]),
                              (expandedMuteItems.value.includes(item.id))
                                ? (_openBlock(), _createElementBlock("div", {
                                  key: 0,
                                  class: _normalizeClass(_ctx.$style.userItemSub)
                                }, [
                                  _createElementVNode("div", null, [
                                    _createTextVNode("Muted at: "),
                                    _createVNode(_component_MkTime, {
                                      time: item.createdAt,
                                      mode: "detail"
                                    }, null, 8 /* PROPS */, ["time"])
                                  ]),
                                  (item.expiresAt)
                                    ? (_openBlock(), _createElementBlock("div", { key: 0 }, "Period: " + _toDisplayString(new Date(item.expiresAt).toLocaleString()), 1 /* TEXT */))
                                    : (_openBlock(), _createElementBlock("div", { key: 1 }, "Period: " + _toDisplayString(_unref(i18n).ts.indefinitely), 1 /* TEXT */))
                                ]))
                                : _createCommentVNode("v-if", true)
                            ], 2 /* CLASS */))
                          }), 128 /* KEYED_FRAGMENT */))
                        ])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["paginator"])
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["label", "keywords"]),
            _createVNode(_component_SearchMarker, {
              label: _unref(i18n).ts.blockedUsers,
              keywords: ['block', 'user']
            }, {
              default: _withCtx(() => [
                _createVNode(MkFolder, null, {
                  icon: _withCtx(() => [
                    _hoisted_9
                  ]),
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.blockedUsers), 1 /* TEXT */)
                  ]),
                  default: _withCtx(() => [
                    _createVNode(MkPagination, {
                      paginator: _unref(blockingPaginator),
                      withControl: ""
                    }, {
                      empty: _withCtx(() => [
                        _createVNode(_component_MkResult, {
                          type: "empty",
                          text: _unref(i18n).ts.noUsers
                        }, null, 8 /* PROPS */, ["text"])
                      ]),
                      default: _withCtx(({ items }) => [
                        _createElementVNode("div", { class: "_gaps_s" }, [
                          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
                            return (_openBlock(), _createElementBlock("div", {
                              key: item.blockee.id,
                              class: _normalizeClass([_ctx.$style.userItem, { [_ctx.$style.userItemOpend]: expandedBlockItems.value.includes(item.id) }])
                            }, [
                              _createElementVNode("div", {
                                class: _normalizeClass(_ctx.$style.userItemMain)
                              }, [
                                _createVNode(_component_MkA, {
                                  class: _normalizeClass(_ctx.$style.userItemMainBody),
                                  to: _unref(userPage)(item.blockee)
                                }, {
                                  default: _withCtx(() => [
                                    _createVNode(MkUserCardMini, { user: item.blockee }, null, 8 /* PROPS */, ["user"])
                                  ]),
                                  _: 2 /* DYNAMIC */
                                }, 8 /* PROPS */, ["to"]),
                                _createElementVNode("button", {
                                  class: _normalizeClass(["_button", _ctx.$style.userToggle]),
                                  onClick: _cache[5] || (_cache[5] = ($event: any) => (toggleBlockItem(item)))
                                }, [
                                  _createElementVNode("i", {
                                    class: _normalizeClass(["ti ti-chevron-down", _ctx.$style.chevron])
                                  })
                                ]),
                                _createElementVNode("button", {
                                  class: _normalizeClass(["_button", _ctx.$style.remove]),
                                  onClick: _cache[6] || (_cache[6] = ($event: any) => (unblock(item.blockee, $event)))
                                }, [
                                  _hoisted_10
                                ])
                              ]),
                              (expandedBlockItems.value.includes(item.id))
                                ? (_openBlock(), _createElementBlock("div", {
                                  key: 0,
                                  class: _normalizeClass(_ctx.$style.userItemSub)
                                }, [
                                  _createElementVNode("div", null, [
                                    _createTextVNode("Blocked at: "),
                                    _createVNode(_component_MkTime, {
                                      time: item.createdAt,
                                      mode: "detail"
                                    }, null, 8 /* PROPS */, ["time"])
                                  ])
                                ]))
                                : _createCommentVNode("v-if", true)
                            ], 2 /* CLASS */))
                          }), 128 /* KEYED_FRAGMENT */))
                        ])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["paginator"])
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["label", "keywords"])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["label", "keywords"]))
}
}

})
