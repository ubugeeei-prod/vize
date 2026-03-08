import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-search" })
const _hoisted_2 = { class: "name _monospace" }
const _hoisted_3 = { class: "info" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-search" })
const _hoisted_5 = { class: "name _monospace" }
const _hoisted_6 = { class: "info" }
import * as Misskey from 'misskey-js'
import { computed, markRaw, ref } from 'vue'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import MkPagination from '@/components/MkPagination.vue'
import MkRemoteEmojiEditDialog from '@/components/MkRemoteEmojiEditDialog.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import FormSplit from '@/components/form/split.vue'
import { selectFile } from '@/utility/drive.js'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { getProxiedImageUrl } from '@/utility/media-proxy.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { Paginator } from '@/utility/paginator.js'

type RemoteEmoji = Misskey.entities.AdminEmojiListRemoteResponse[number] & { host: string };

export default /*@__PURE__*/_defineComponent({
  __name: 'custom-emojis-manager',
  setup(__props) {

const tab = ref('local');
const query = ref<string | null>(null);
const queryRemote = ref<string | null>(null);
const host = ref<string | null>(null);
const selectMode = ref(false);
const selectedEmojis = ref<string[]>([]);
const paginator = markRaw(new Paginator('admin/emoji/list', {
	limit: 30,
	computedParams: computed(() => ({
		query: (query.value && query.value !== '') ? query.value : null,
	})),
}));
const remotePaginator = markRaw(new Paginator('admin/emoji/list-remote', {
	limit: 30,
	computedParams: computed(() => ({
		query: (queryRemote.value && queryRemote.value !== '') ? queryRemote.value : null,
		host: (host.value && host.value !== '') ? host.value : null,
	})),
}));
const selectAll = () => {
	if (selectedEmojis.value.length > 0) {
		selectedEmojis.value = [];
	} else {
		selectedEmojis.value = paginator.items.value.map(item => item.id);
	}
};
const toggleSelect = (emoji: Misskey.entities.EmojiDetailed) => {
	if (selectedEmojis.value.includes(emoji.id)) {
		selectedEmojis.value = selectedEmojis.value.filter(x => x !== emoji.id);
	} else {
		selectedEmojis.value.push(emoji.id);
	}
};
const add = async () => {
	const { dispose } = await os.popupAsyncWithDialog(import('./emoji-edit-dialog.vue').then(x => x.default), {
	}, {
		done: result => {
			if (result.created) {
				const nowIso = (new Date()).toISOString();
				paginator.prepend({
					...result.created,
					createdAt: nowIso,
				});
			}
		},
		closed: () => dispose(),
	});
};
const edit = async (emoji: Misskey.entities.EmojiDetailed) => {
	const { dispose } = await os.popupAsyncWithDialog(import('./emoji-edit-dialog.vue').then(x => x.default), {
		emoji: emoji,
	}, {
		done: result => {
			if (result.updated) {
				paginator.updateItem(result.updated.id, (oldEmoji) => ({
					...oldEmoji,
					...result.updated,
				}));
			} else if (result.deleted) {
				paginator.removeItem(emoji.id);
			}
		},
		closed: () => dispose(),
	});
};
const detailRemoteEmoji = (emoji: {
	id: string,
	name: string,
	host: string,
	license: string | null,
	url: string
}) => {
	const { dispose } = os.popup(MkRemoteEmojiEditDialog, {
		emoji: emoji,
	}, {
		done: () => {
			dispose();
		},
		closed: () => {
			dispose();
		},
	});
};
const importEmoji = (emojiId: string) => {
	os.apiWithDialog('admin/emoji/copy', {
		emojiId: emojiId,
	});
};
const remoteMenu = (emoji: {
	id: string,
	name: string,
	host: string,
	license: string | null,
	url: string
}, ev: PointerEvent) => {
	os.popupMenu([{
		type: 'label',
		text: ':' + emoji.name + ':',
	}, {
		text: i18n.ts.details,
		icon: 'ti ti-info-circle',
		action: () => { detailRemoteEmoji(emoji); },
	}, {
		text: i18n.ts.import,
		icon: 'ti ti-plus',
		action: () => { importEmoji(emoji.id); },
	}], ev.currentTarget ?? ev.target);
};
const menu = (ev: PointerEvent) => {
	os.popupMenu([{
		icon: 'ti ti-download',
		text: i18n.ts.export,
		action: async () => {
			misskeyApi('export-custom-emojis', {
			})
				.then(() => {
					os.alert({
						type: 'info',
						text: i18n.ts.exportRequested,
					});
				}).catch((err) => {
					os.alert({
						type: 'error',
						text: err.message,
					});
				});
		},
	}, {
		icon: 'ti ti-upload',
		text: i18n.ts.import,
		action: async () => {
			const file = await selectFile({
				anchorElement: ev.currentTarget ?? ev.target,
				multiple: false,
			});
			misskeyApi('admin/emoji/import-zip', {
				fileId: file.id,
			})
				.then(() => {
					os.alert({
						type: 'info',
						text: i18n.ts.importRequested,
					});
				}).catch((err) => {
					os.alert({
						type: 'error',
						text: err.message,
					});
				});
		},
	}], ev.currentTarget ?? ev.target);
};
const setCategoryBulk = async () => {
	const { canceled, result } = await os.inputText({
		title: 'Category',
	});
	if (canceled) return;
	await os.apiWithDialog('admin/emoji/set-category-bulk', {
		ids: selectedEmojis.value,
		category: result,
	});
	paginator.reload();
};
const setLicenseBulk = async () => {
	const { canceled, result } = await os.inputText({
		title: 'License',
	});
	if (canceled) return;
	await os.apiWithDialog('admin/emoji/set-license-bulk', {
		ids: selectedEmojis.value,
		license: result,
	});
	paginator.reload();
};
const addTagBulk = async () => {
	const { canceled, result } = await os.inputText({
		title: 'Tag',
	});
	if (canceled || result == null) return;
	await os.apiWithDialog('admin/emoji/add-aliases-bulk', {
		ids: selectedEmojis.value,
		aliases: result.split(' '),
	});
	paginator.reload();
};
const removeTagBulk = async () => {
	const { canceled, result } = await os.inputText({
		title: 'Tag',
	});
	if (canceled || result == null) return;
	await os.apiWithDialog('admin/emoji/remove-aliases-bulk', {
		ids: selectedEmojis.value,
		aliases: result.split(' '),
	});
	paginator.reload();
};
const setTagBulk = async () => {
	const { canceled, result } = await os.inputText({
		title: 'Tag',
	});
	if (canceled || result == null) return;
	await os.apiWithDialog('admin/emoji/set-aliases-bulk', {
		ids: selectedEmojis.value,
		aliases: result.split(' '),
	});
	paginator.reload();
};
const delBulk = async () => {
	const { canceled } = await os.confirm({
		type: 'warning',
		text: i18n.ts.deleteConfirm,
	});
	if (canceled) return;
	await os.apiWithDialog('admin/emoji/delete-bulk', {
		ids: selectedEmojis.value,
	});
	paginator.reload();
};
const headerActions = computed(() => [{
	asFullButton: true,
	icon: 'ti ti-plus',
	text: i18n.ts.addEmoji,
	handler: add,
}, {
	icon: 'ti ti-dots',
	text: i18n.ts.more,
	handler: menu,
}]);
const headerTabs = computed(() => [{
	key: 'local',
	title: i18n.ts.local,
}, {
	key: 'remote',
	title: i18n.ts.remote,
}]);
definePage(() => ({
	title: i18n.ts.customEmojis,
	icon: 'ti ti-icons',
}));

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value,
      tab: tab.value,
      "onUpdate:tab": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 900px;"
        }, [
          _createElementVNode("div", { class: "ogwlenmc" }, [
            (tab.value === 'local')
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: "local"
              }, [
                _createVNode(MkInput, {
                  debounce: true,
                  type: "search",
                  autocapitalize: "off",
                  modelValue: query.value,
                  "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((query).value = $event))
                }, {
                  prefix: _withCtx(() => [
                    _hoisted_1
                  ]),
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.search), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["debounce", "modelValue"]),
                _createVNode(MkSwitch, {
                  style: "margin: 8px 0;",
                  modelValue: selectMode.value,
                  "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((selectMode).value = $event))
                }, {
                  label: _withCtx(() => [
                    _createTextVNode("Select mode")
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["modelValue"]),
                (selectMode.value)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: "_buttons"
                  }, [
                    _createVNode(MkButton, {
                      inline: "",
                      onClick: selectAll
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode("Select all")
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(MkButton, {
                      inline: "",
                      onClick: setCategoryBulk
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode("Set category")
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(MkButton, {
                      inline: "",
                      onClick: setTagBulk
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode("Set tag")
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(MkButton, {
                      inline: "",
                      onClick: addTagBulk
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode("Add tag")
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(MkButton, {
                      inline: "",
                      onClick: removeTagBulk
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode("Remove tag")
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(MkButton, {
                      inline: "",
                      onClick: setLicenseBulk
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode("Set License")
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(MkButton, {
                      inline: "",
                      danger: "",
                      onClick: delBulk
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode("Delete")
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]))
                  : _createCommentVNode("v-if", true),
                _createVNode(MkPagination, {
                  ref: "emojisPaginationComponent",
                  paginator: _unref(paginator)
                }, {
                  empty: _withCtx(() => [
                    _createElementVNode("span", null, _toDisplayString(_unref(i18n).ts.noCustomEmojis), 1 /* TEXT */)
                  ]),
                  default: _withCtx(({items}) => [
                    _createElementVNode("div", { class: "ldhfsamy" }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (emoji) => {
                        return (_openBlock(), _createElementBlock("button", {
                          key: emoji.id,
                          class: _normalizeClass(["emoji _panel _button", { selected: selectedEmojis.value.includes(emoji.id) }]),
                          onClick: _cache[3] || (_cache[3] = ($event: any) => (selectMode.value ? toggleSelect(emoji) : edit(emoji)))
                        }, [
                          _createElementVNode("img", {
                            src: emoji.url,
                            class: "img",
                            alt: emoji.name
                          }, null, 8 /* PROPS */, ["src", "alt"]),
                          _createElementVNode("div", { class: "body" }, [
                            _createElementVNode("div", _hoisted_2, _toDisplayString(emoji.name), 1 /* TEXT */),
                            _createElementVNode("div", _hoisted_3, _toDisplayString(emoji.category), 1 /* TEXT */)
                          ])
                        ], 2 /* CLASS */))
                      }), 128 /* KEYED_FRAGMENT */))
                    ])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["paginator"])
              ]))
              : (tab.value === 'remote')
                ? (_openBlock(), _createElementBlock("div", {
                  key: 1,
                  class: "remote"
                }, [
                  _createVNode(FormSplit, null, {
                    default: _withCtx(() => [
                      _createVNode(MkInput, {
                        debounce: true,
                        type: "search",
                        autocapitalize: "off",
                        modelValue: queryRemote.value,
                        "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((queryRemote).value = $event))
                      }, {
                        prefix: _withCtx(() => [
                          _hoisted_4
                        ]),
                        label: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.search), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["debounce", "modelValue"]),
                      _createVNode(MkInput, {
                        debounce: true,
                        modelValue: host.value,
                        "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((host).value = $event))
                      }, {
                        label: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.host), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["debounce", "modelValue"])
                    ]),
                    _: 1 /* STABLE */
                  }),
                  _createVNode(MkPagination, { paginator: _unref(remotePaginator) }, {
                    empty: _withCtx(() => [
                      _createElementVNode("span", null, _toDisplayString(_unref(i18n).ts.noCustomEmojis), 1 /* TEXT */)
                    ]),
                    default: _withCtx(({items}) => [
                      _createElementVNode("div", { class: "ldhfsamy" }, [
                        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (emoji) => {
                          return (_openBlock(), _createElementBlock("div", {
                            key: emoji.id,
                            class: "emoji _panel _button",
                            onClick: _cache[6] || (_cache[6] = ($event: any) => (remoteMenu(emoji, $event)))
                          }, [
                            _createElementVNode("img", {
                              src: _unref(getProxiedImageUrl)(emoji.url, 'emoji'),
                              class: "img",
                              alt: emoji.name
                            }, null, 8 /* PROPS */, ["src", "alt"]),
                            _createElementVNode("div", { class: "body" }, [
                              _createElementVNode("div", _hoisted_5, _toDisplayString(emoji.name), 1 /* TEXT */),
                              _createElementVNode("div", _hoisted_6, _toDisplayString(emoji.host), 1 /* TEXT */)
                            ])
                          ]))
                        }), 128 /* KEYED_FRAGMENT */))
                      ])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["paginator"])
                ]))
              : _createCommentVNode("v-if", true)
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs", "tab"]))
}
}

})
