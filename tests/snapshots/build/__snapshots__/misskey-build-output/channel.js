import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-star" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-star" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-users ti-fw" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil ti-fw" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-user-star ti-fw" })
const _hoisted_6 = { style: "margin-left: 4px;" }
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pin ti-fw", style: "margin-right: 0.5em;" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-search" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil" })
import { computed, watch, ref, markRaw, shallowRef } from 'vue'
import * as Misskey from 'misskey-js'
import { url } from '@@/js/config.js'
import { useInterval } from '@@/js/use-interval.js'
import type { PageHeaderItem } from '@/types/page-header.js'
import MkPostForm from '@/components/MkPostForm.vue'
import MkStreamingNotesTimeline from '@/components/MkStreamingNotesTimeline.vue'
import XChannelFollowButton from '@/components/MkChannelFollowButton.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { $i, iAmModerator } from '@/i.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { deviceKind } from '@/utility/device-kind.js'
import MkNotesTimeline from '@/components/MkNotesTimeline.vue'
import { favoritedChannelsCache } from '@/cache.js'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import { prefer } from '@/preferences.js'
import MkNote from '@/components/MkNote.vue'
import MkInfo from '@/components/MkInfo.vue'
import MkFoldableSection from '@/components/MkFoldableSection.vue'
import { isSupportShare } from '@/utility/navigator.js'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'
import { notesSearchAvailable } from '@/utility/check-permissions.js'
import { miLocalStorage } from '@/local-storage.js'
import { useRouter } from '@/router.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'channel',
  props: {
    channelId: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const router = useRouter();
const tab = ref('overview');
const channel = ref<Misskey.entities.Channel | null>(null);
const favorited = ref(false);
const searchQuery = ref('');
const searchPaginator = shallowRef();
const searchKey = ref('');
const featuredPaginator = markRaw(new Paginator('notes/featured', {
	limit: 10,
	computedParams: computed(() => ({
		channelId: props.channelId,
	})),
}));
useInterval(() => {
	if (channel.value == null) return;
	miLocalStorage.setItemAsJson(`channelLastReadedAt:${channel.value.id}`, Date.now());
}, 3000, {
	immediate: true,
	afterMounted: true,
});
watch(() => props.channelId, async () => {
	const _channel = await misskeyApi('channels/show', {
		channelId: props.channelId,
	});
	favorited.value = _channel.isFavorited ?? false;
	if (favorited.value || _channel.isFollowing) {
		tab.value = 'timeline';
	}
	if ((favorited.value || _channel.isFollowing) && _channel.lastNotedAt) {
		const lastReadedAt: number = miLocalStorage.getItemAsJson(`channelLastReadedAt:${_channel.id}`) ?? 0;
		const lastNotedAt = Date.parse(_channel.lastNotedAt);
		if (lastNotedAt > lastReadedAt) {
			miLocalStorage.setItemAsJson(`channelLastReadedAt:${_channel.id}`, lastNotedAt);
		}
	}
	channel.value = _channel;
}, { immediate: true });
function edit() {
	router.push('/channels/:channelId/edit', {
		params: {
			channelId: props.channelId,
		},
	});
}
function openPostForm() {
	os.post({
		channel: channel.value,
	});
}
function favorite() {
	if (!channel.value) return;
	os.apiWithDialog('channels/favorite', {
		channelId: channel.value.id,
	}).then(() => {
		favorited.value = true;
		favoritedChannelsCache.delete();
	});
}
async function unfavorite() {
	if (!channel.value) return;
	const confirm = await os.confirm({
		type: 'warning',
		text: i18n.ts.unfavoriteConfirm,
	});
	if (confirm.canceled) return;
	os.apiWithDialog('channels/unfavorite', {
		channelId: channel.value.id,
	}).then(() => {
		favorited.value = false;
		favoritedChannelsCache.delete();
	});
}
async function mute() {
	if (!channel.value) return;
	const _channel = channel.value;
	const { canceled, result: period } = await os.select({
		title: i18n.ts.mutePeriod,
		items: [{
			value: 'indefinitely', label: i18n.ts.indefinitely,
		}, {
			value: 'tenMinutes', label: i18n.ts.tenMinutes,
		}, {
			value: 'oneHour', label: i18n.ts.oneHour,
		}, {
			value: 'oneDay', label: i18n.ts.oneDay,
		}, {
			value: 'oneWeek', label: i18n.ts.oneWeek,
		}],
		default: 'indefinitely',
	});
	if (canceled) return;
	const expiresAt = period === 'indefinitely' ? null
		: period === 'tenMinutes' ? Date.now() + (1000 * 60 * 10)
		: period === 'oneHour' ? Date.now() + (1000 * 60 * 60)
		: period === 'oneDay' ? Date.now() + (1000 * 60 * 60 * 24)
		: period === 'oneWeek' ? Date.now() + (1000 * 60 * 60 * 24 * 7)
		: null;
	os.apiWithDialog('channels/mute/create', {
		channelId: _channel.id,
		expiresAt,
	}).then(() => {
		_channel.isMuting = true;
	});
}
async function unmute() {
	if (!channel.value) return;
	const _channel = channel.value;
	os.apiWithDialog('channels/mute/delete', {
		channelId: _channel.id,
	}).then(() => {
		_channel.isMuting = false;
	});
}
async function search() {
	if (!channel.value) return;
	const query = searchQuery.value.toString().trim();
	if (query == null) return;
	searchPaginator.value = markRaw(new Paginator('notes/search', {
		limit: 10,
		params: {
			query: query,
			channelId: channel.value.id,
		},
	}));
	searchKey.value = query;
}
const headerActions = computed(() => {
	if (channel.value) {
		const headerItems: PageHeaderItem[] = [];

		headerItems.push({
			icon: 'ti ti-link',
			text: i18n.ts.copyUrl,
			handler: async (): Promise<void> => {
				if (!channel.value) {
					console.warn('failed to copy channel URL. channel.value is null.');
					return;
				}
				copyToClipboard(`${url}/channels/${channel.value.id}`);
			},
		});

		if (isSupportShare()) {
			headerItems.push({
				icon: 'ti ti-share',
				text: i18n.ts.share,
				handler: async (): Promise<void> => {
					if (!channel.value) {
						console.warn('failed to share channel. channel.value is null.');
						return;
					}

					navigator.share({
						title: channel.value.name,
						text: channel.value.description ?? undefined,
						url: `${url}/channels/${channel.value.id}`,
					});
				},
			});
		}

		if (!channel.value.isMuting) {
			headerItems.push({
				icon: 'ti ti-volume',
				text: i18n.ts.mute,
				handler: async (): Promise<void> => {
					await mute();
				},
			});
		} else {
			headerItems.push({
				icon: 'ti ti-volume-off',
				text: i18n.ts.unmute,
				handler: async (): Promise<void> => {
					await unmute();
				},
			});
		}

		if (($i && $i.id === channel.value.userId) || iAmModerator) {
			headerItems.push({
				icon: 'ti ti-settings',
				text: i18n.ts.edit,
				handler: edit,
			});
		}

		return headerItems.length > 0 ? headerItems : null;
	} else {
		return null;
	}
});
const headerTabs = computed(() => [{
	key: 'overview',
	title: i18n.ts.overview,
	icon: 'ti ti-info-circle',
}, {
	key: 'timeline',
	title: i18n.ts.timeline,
	icon: 'ti ti-home',
}, {
	key: 'featured',
	title: i18n.ts.featured,
	icon: 'ti ti-bolt',
}, {
	key: 'search',
	title: i18n.ts.search,
	icon: 'ti ti-search',
}]);
definePage(() => ({
	title: channel.value ? channel.value.name : i18n.ts.channel,
	icon: 'ti ti-device-tv',
}));

return (_ctx: any,_cache: any) => {
  const _component_I18n = _resolveComponent("I18n")
  const _component_Mfm = _resolveComponent("Mfm")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value,
      swipable: true,
      tab: tab.value,
      "onUpdate:tab": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
    }, {
      footer: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.footer)
        }, [
          _createElementVNode("div", {
            class: "_spacer",
            style: "--MI_SPACER-w: 700px; --MI_SPACER-min: 16px; --MI_SPACER-max: 16px;"
          }, [
            _createElementVNode("div", { class: "_buttonsCenter" }, [
              _createVNode(MkButton, {
                inline: "",
                rounded: "",
                primary: "",
                gradate: "",
                onClick: _cache[1] || (_cache[1] = ($event: any) => (openPostForm()))
              }, {
                default: _withCtx(() => [
                  _hoisted_9,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.postToTheChannel), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ])
          ])
        ])
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 700px;"
        }, [
          (channel.value && tab.value === 'overview')
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "_gaps"
            }, [
              _createElementVNode("div", {
                class: _normalizeClass(["_panel", _ctx.$style.bannerContainer])
              }, [
                _createVNode(XChannelFollowButton, {
                  channel: channel.value,
                  full: true,
                  class: _normalizeClass(_ctx.$style.subscribe)
                }, null, 8 /* PROPS */, ["channel", "full"]),
                (favorited.value)
                  ? (_openBlock(), _createBlock(MkButton, {
                    key: 0,
                    asLike: "",
                    rounded: "",
                    primary: "",
                    class: _normalizeClass(["button", _ctx.$style.favorite]),
                    onClick: _cache[2] || (_cache[2] = ($event: any) => (unfavorite()))
                  }, {
                    default: _withCtx(() => [
                      _hoisted_1
                    ]),
                    _: 1 /* STABLE */
                  }))
                  : (_openBlock(), _createBlock(MkButton, {
                    key: 1,
                    asLike: "",
                    rounded: "",
                    class: _normalizeClass(["button", _ctx.$style.favorite]),
                    onClick: _cache[3] || (_cache[3] = ($event: any) => (favorite()))
                  }, {
                    default: _withCtx(() => [
                      _hoisted_2
                    ]),
                    _: 1 /* STABLE */
                  })),
                _createElementVNode("div", {
                  style: _normalizeStyle({ backgroundImage: channel.value.bannerUrl ? `url(${channel.value.bannerUrl})` : undefined }),
                  class: _normalizeClass(_ctx.$style.banner)
                }, [
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.bannerStatus)
                  }, [
                    _createElementVNode("div", null, [
                      _hoisted_3,
                      _createVNode(_component_I18n, {
                        src: _unref(i18n).ts._channel.usersCount,
                        tag: "span",
                        style: "margin-left: 4px;"
                      }, {
                        n: _withCtx(() => [
                          _createElementVNode("b", null, _toDisplayString(channel.value.usersCount), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["src"])
                    ]),
                    _createElementVNode("div", null, [
                      _hoisted_4,
                      _createVNode(_component_I18n, {
                        src: _unref(i18n).ts._channel.notesCount,
                        tag: "span",
                        style: "margin-left: 4px;"
                      }, {
                        n: _withCtx(() => [
                          _createElementVNode("b", null, _toDisplayString(channel.value.notesCount), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["src"])
                    ]),
                    (_unref($i) != null && channel.value != null && _unref($i).id === channel.value.userId)
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        style: "color: var(--MI_THEME-warn)"
                      }, [
                        _hoisted_5,
                        _createElementVNode("span", _hoisted_6, _toDisplayString(_unref(i18n).ts.youAreAdmin), 1 /* TEXT */)
                      ]))
                      : _createCommentVNode("v-if", true)
                  ]),
                  (channel.value.isSensitive)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.sensitiveIndicator)
                    }, _toDisplayString(_unref(i18n).ts.sensitive), 1 /* TEXT */))
                    : _createCommentVNode("v-if", true),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.bannerFade)
                  })
                ], 4 /* STYLE */),
                (channel.value.description)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.description)
                  }, [
                    _createVNode(_component_Mfm, {
                      text: channel.value.description,
                      isNote: false
                    }, null, 8 /* PROPS */, ["text", "isNote"])
                  ]))
                  : _createCommentVNode("v-if", true)
              ]),
              _createVNode(MkFoldableSection, null, {
                header: _withCtx(() => [
                  _hoisted_7,
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.pinnedNotes), 1 /* TEXT */)
                ]),
                default: _withCtx(() => [
                  (channel.value.pinnedNotes && channel.value.pinnedNotes.length > 0)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "_gaps"
                    }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(channel.value.pinnedNotes, (note) => {
                        return (_openBlock(), _createBlock(MkNote, {
                          key: note.id,
                          class: "_panel",
                          note: note
                        }, null, 8 /* PROPS */, ["note"]))
                      }), 128 /* KEYED_FRAGMENT */))
                    ]))
                    : _createCommentVNode("v-if", true)
                ]),
                _: 1 /* STABLE */
              })
            ]))
            : _createCommentVNode("v-if", true),
          (channel.value && tab.value === 'timeline')
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "_gaps"
            }, [
              (channel.value.isArchived)
                ? (_openBlock(), _createBlock(MkInfo, {
                  key: 0,
                  warn: ""
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.thisChannelArchived), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }))
                : _createCommentVNode("v-if", true),
              _createTextVNode("\n\n\t\t\t"),
              _createTextVNode("\n\t\t\t"),
              (_unref($i) && _unref(prefer).r.showFixedPostFormInChannel.value)
                ? (_openBlock(), _createBlock(MkPostForm, {
                  key: 0,
                  channel: channel.value,
                  class: "post-form _panel",
                  fixed: "",
                  autofocus: _unref(deviceKind) === 'desktop'
                }, null, 8 /* PROPS */, ["channel", "autofocus"]))
                : _createCommentVNode("v-if", true),
              _createVNode(MkStreamingNotesTimeline, {
                key: __props.channelId,
                src: "channel",
                channel: __props.channelId
              }, null, 8 /* PROPS */, ["channel"])
            ]))
            : (tab.value === 'featured')
              ? (_openBlock(), _createElementBlock("div", { key: 1 }, [
                _createVNode(MkNotesTimeline, { paginator: _unref(featuredPaginator) }, null, 8 /* PROPS */, ["paginator"])
              ]))
            : (tab.value === 'search')
              ? (_openBlock(), _createElementBlock("div", { key: 2 }, [
                (_unref(notesSearchAvailable))
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: "_gaps"
                  }, [
                    _createElementVNode("div", null, [
                      _createVNode(MkInput, {
                        onEnter: _cache[4] || (_cache[4] = ($event: any) => (search())),
                        modelValue: searchQuery.value,
                        "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((searchQuery).value = $event))
                      }, {
                        prefix: _withCtx(() => [
                          _hoisted_8
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["modelValue"]),
                      _createVNode(MkButton, {
                        primary: "",
                        rounded: "",
                        style: "margin-top: 8px;",
                        onClick: _cache[6] || (_cache[6] = ($event: any) => (search()))
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.search), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      })
                    ]),
                    (searchPaginator.value)
                      ? (_openBlock(), _createBlock(MkNotesTimeline, {
                        key: searchKey.value,
                        paginator: searchPaginator.value
                      }, null, 8 /* PROPS */, ["paginator"]))
                      : _createCommentVNode("v-if", true)
                  ]))
                  : (_openBlock(), _createElementBlock("div", { key: 1 }, [
                    _createVNode(MkInfo, { warn: "" }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.notesSearchNotAvailable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]))
              ]))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs", "swipable", "tab"]))
}
}

})
