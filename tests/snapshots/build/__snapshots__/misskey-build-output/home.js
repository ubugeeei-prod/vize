import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, vModelText as _vModelText } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { class: "fade" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-lock" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-robot" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-edit" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-lock" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-robot" })
const _hoisted_8 = { class: "messageHeader" }
const _hoisted_9 = { class: "heading" }
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-map-pin ti-fw" })
const _hoisted_11 = { class: "value" }
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-cake ti-fw" })
const _hoisted_13 = { class: "value" }
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-calendar ti-fw" })
import { defineAsyncComponent, computed, onMounted, onUnmounted, onActivated, onDeactivated, nextTick, watch, ref, useTemplateRef } from 'vue'
import * as Misskey from 'misskey-js'
import { getScrollContainer } from '@@/js/scroll.js'
import MkNote from '@/components/MkNote.vue'
import MkFollowButton from '@/components/MkFollowButton.vue'
import MkAccountMoved from '@/components/MkAccountMoved.vue'
import MkFukidashi from '@/components/MkFukidashi.vue'
import MkRemoteCaution from '@/components/MkRemoteCaution.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import MkOmit from '@/components/MkOmit.vue'
import MkInfo from '@/components/MkInfo.vue'
import MkButton from '@/components/MkButton.vue'
import { getUserMenu } from '@/utility/get-user-menu.js'
import number from '@/filters/number.js'
import { userPage } from '@/filters/user.js'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { $i, iAmModerator } from '@/i.js'
import { dateString } from '@/filters/date.js'
import { confetti } from '@/utility/confetti.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { isFollowingVisibleForMe, isFollowersVisibleForMe } from '@/utility/isFfVisibleForMe.js'
import { useRouter } from '@/router.js'
import { getStaticImageUrl } from '@/utility/media-proxy.js'
import MkSparkle from '@/components/MkSparkle.vue'
import { prefer } from '@/preferences.js'
import MkPullToRefresh from '@/components/MkPullToRefresh.vue'
import { isBirthday } from '@/utility/is-birthday.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'home',
  props: {
    user: { type: null, required: true },
    disableNotes: { type: Boolean, required: false, default: false }
  },
  emits: ["showMoreFiles"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
function calcAge(birthdate: string): number {
	const date = new Date(birthdate);
	const now = new Date();
	let yearDiff = now.getFullYear() - date.getFullYear();
	const monthDiff = now.getMonth() - date.getMonth();
	const pastDate = now.getDate() < date.getDate();
	if (monthDiff < 0 || (monthDiff === 0 && pastDate)) {
		yearDiff--;
	}
	return yearDiff;
}
const XFiles = defineAsyncComponent(() => import('./index.files.vue'));
const XActivity = defineAsyncComponent(() => import('./index.activity.vue'));
const XTimeline = defineAsyncComponent(() => import('./index.timeline.vue'));
const router = useRouter();
const user = ref(props.user);
const narrow = ref<null | boolean>(null);
const rootEl = useTemplateRef('rootEl');
const bannerEl = useTemplateRef('bannerEl');
const memoTextareaEl = useTemplateRef('memoTextareaEl');
const memoDraft = ref(props.user.memo);
const isEditingMemo = ref(false);
const moderationNote = ref(props.user.moderationNote ?? '');
const editModerationNote = ref(false);
watch(moderationNote, async () => {
	await misskeyApi('admin/update-user-note', { userId: props.user.id, text: moderationNote.value });
});
const style = computed(() => {
	if (props.user.bannerUrl == null) return {};
	if (prefer.s.disableShowingAnimatedImages) {
		return {
			backgroundImage: `url(${ getStaticImageUrl(props.user.bannerUrl) })`,
		};
	} else {
		return {
			backgroundImage: `url(${ props.user.bannerUrl })`,
		};
	};
});
const age = computed(() => {
	return props.user.birthday ? calcAge(props.user.birthday) : NaN;
});
function menu(ev: PointerEvent) {
	const { menu, cleanup } = getUserMenu(user.value, router);
	os.popupMenu(menu, ev.currentTarget ?? ev.target).finally(cleanup);
}
function showMemoTextarea() {
	isEditingMemo.value = true;
	nextTick(() => {
		memoTextareaEl.value?.focus();
	});
}
function adjustMemoTextarea() {
	if (!memoTextareaEl.value) return;
	memoTextareaEl.value.style.height = '0px';
	memoTextareaEl.value.style.height = `${memoTextareaEl.value.scrollHeight}px`;
}
async function updateMemo() {
	await misskeyApi('users/update-memo', {
		memo: memoDraft.value,
		userId: props.user.id,
	});
	isEditingMemo.value = false;
}
watch([props.user], () => {
	memoDraft.value = props.user.memo;
});
async function reload() {
	// TODO
}
let bannerParallaxResizeObserver: ResizeObserver | null = null;
function calcBannerParallax() {
	if (!bannerEl.value || !CSS.supports('view-timeline-inset', 'auto 100px')) return;
	const elRect = bannerEl.value.getBoundingClientRect();
	const scrollEl = getScrollContainer(bannerEl.value);
	const scrollPosition = scrollEl?.scrollTop ?? window.scrollY;
	const scrollContainerHeight = scrollEl?.clientHeight ?? window.innerHeight;
	const scrollContainerTop = scrollEl?.getBoundingClientRect().top ?? 0;
	const top = scrollPosition + elRect.top - scrollContainerTop;
	const bottom = scrollContainerHeight - top;
	bannerEl.value.style.setProperty('--bannerParallaxInset', `auto ${bottom}px`);
}
function initCalcBannerParallax() {
	const scrollEl = bannerEl.value ? getScrollContainer(bannerEl.value) : null;
	if (scrollEl != null && CSS.supports('view-timeline-inset', 'auto 100px')) {
		bannerParallaxResizeObserver = new ResizeObserver(() => {
			calcBannerParallax();
		});
		bannerParallaxResizeObserver.observe(scrollEl);
	}
}
function disposeBannerParallaxResizeObserver() {
	if (bannerParallaxResizeObserver) {
		bannerParallaxResizeObserver.disconnect();
		bannerParallaxResizeObserver = null;
	}
}
onMounted(() => {
	narrow.value = rootEl.value!.clientWidth < 1000;
	if (isBirthday(user.value)) {
		confetti({
			duration: 1000 * 4,
		});
	}
	nextTick(() => {
		calcBannerParallax();
		adjustMemoTextarea();
	});
	initCalcBannerParallax();
});
onActivated(() => {
	if (bannerEl.value) {
		calcBannerParallax();
		initCalcBannerParallax();
	}
});
onUnmounted(disposeBannerParallaxResizeObserver);
onDeactivated(disposeBannerParallaxResizeObserver);

return (_ctx: any,_cache: any) => {
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_MkAcct = _resolveComponent("MkAcct")
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_Mfm = _resolveComponent("Mfm")
  const _component_MkA = _resolveComponent("MkA")
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_MkLazy = _resolveComponent("MkLazy")
  const _directive_tooltip = _resolveDirective("tooltip")
  const _directive_adaptive_bg = _resolveDirective("adaptive-bg")

  return (_openBlock(), _createBlock(_resolveDynamicComponent(_unref(prefer).s.enablePullToRefresh ? MkPullToRefresh : 'div'), { refresher: () => reload() }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: _normalizeStyle({ '--MI_SPACER-w': narrow.value ? '800px' : '1100px' })
        }, [
          _createElementVNode("div", {
            ref_key: "rootEl", ref: rootEl,
            class: _normalizeClass(["ftskorzw", { wide: !narrow.value }]),
            style: "container-type: inline-size;"
          }, [
            _createElementVNode("div", { class: "main _gaps" }, [
              _createElementVNode("div", { class: "profile _gaps" }, [
                (user.value.movedTo)
                  ? (_openBlock(), _createBlock(MkAccountMoved, {
                    key: 0,
                    movedTo: user.value.movedTo
                  }, null, 8 /* PROPS */, ["movedTo"]))
                  : _createCommentVNode("v-if", true),
                (user.value.host != null)
                  ? (_openBlock(), _createBlock(MkRemoteCaution, {
                    key: 0,
                    href: user.value.url ?? user.value.uri
                  }, null, 8 /* PROPS */, ["href"]))
                  : _createCommentVNode("v-if", true),
                (user.value.host == null && user.value.username.includes('.'))
                  ? (_openBlock(), _createBlock(MkInfo, { key: 0 }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.isSystemAccount), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }))
                  : _createCommentVNode("v-if", true),
                _createElementVNode("div", {
                  key: user.value.id,
                  class: "main _panel"
                }, [
                  _createElementVNode("div", {
                    ref_key: "bannerEl", ref: bannerEl,
                    class: "banner-container"
                  }, [
                    _createElementVNode("div", {
                      class: "banner",
                      style: _normalizeStyle(style.value)
                    }, null, 4 /* STYLE */),
                    _hoisted_1,
                    _createElementVNode("div", { class: "title" }, [
                      _createVNode(_component_MkUserName, {
                        class: "name",
                        user: user.value,
                        nowrap: true
                      }, null, 8 /* PROPS */, ["user", "nowrap"]),
                      _createElementVNode("div", { class: "bottom" }, [
                        _createElementVNode("span", { class: "username" }, [
                          _createVNode(_component_MkAcct, {
                            user: user.value,
                            detail: true
                          }, null, 8 /* PROPS */, ["user", "detail"])
                        ]),
                        (user.value.isLocked)
                          ? (_openBlock(), _createElementBlock("span", { key: 0 }, [
                            _hoisted_2
                          ]))
                          : _createCommentVNode("v-if", true),
                        (user.value.isBot)
                          ? (_openBlock(), _createElementBlock("span", { key: 0 }, [
                            _hoisted_3
                          ]))
                          : _createCommentVNode("v-if", true),
                        (_unref($i) && !isEditingMemo.value && !memoDraft.value)
                          ? (_openBlock(), _createElementBlock("button", {
                            key: 0,
                            class: "_button add-note-button",
                            onClick: showMemoTextarea
                          }, [
                            _hoisted_4,
                            _createTextVNode(),
                            _toDisplayString(_unref(i18n).ts.addMemo)
                          ]))
                          : _createCommentVNode("v-if", true)
                      ])
                    ]),
                    (_unref($i) && _unref($i).id != user.value.id && user.value.isFollowed)
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: "followed"
                      }, _toDisplayString(_unref(i18n).ts.followsYou), 1 /* TEXT */))
                      : _createCommentVNode("v-if", true),
                    _createElementVNode("div", { class: "actions" }, [
                      _createElementVNode("button", {
                        class: "menu _button",
                        onClick: menu
                      }, [
                        _hoisted_5
                      ]),
                      (_unref($i)?.id != user.value.id)
                        ? (_openBlock(), _createBlock(MkFollowButton, {
                          key: 0,
                          inline: true,
                          transparent: false,
                          full: true,
                          class: "koudoku",
                          user: user.value,
                          "onUpdate:user": _cache[0] || (_cache[0] = ($event: any) => ((user).value = $event))
                        }, null, 8 /* PROPS */, ["inline", "transparent", "full", "user"]))
                        : _createCommentVNode("v-if", true)
                    ])
                  ], 512 /* NEED_PATCH */),
                  _createVNode(_component_MkAvatar, {
                    class: "avatar",
                    user: user.value,
                    indicator: ""
                  }, null, 8 /* PROPS */, ["user"]),
                  _createElementVNode("div", { class: "title" }, [
                    _createVNode(_component_MkUserName, {
                      user: user.value,
                      nowrap: false,
                      class: "name"
                    }, null, 8 /* PROPS */, ["user", "nowrap"]),
                    _createElementVNode("div", { class: "bottom" }, [
                      _createElementVNode("span", { class: "username" }, [
                        _createVNode(_component_MkAcct, {
                          user: user.value,
                          detail: true
                        }, null, 8 /* PROPS */, ["user", "detail"])
                      ]),
                      (user.value.isLocked)
                        ? (_openBlock(), _createElementBlock("span", { key: 0 }, [
                          _hoisted_6
                        ]))
                        : _createCommentVNode("v-if", true),
                      (user.value.isBot)
                        ? (_openBlock(), _createElementBlock("span", { key: 0 }, [
                          _hoisted_7
                        ]))
                        : _createCommentVNode("v-if", true)
                    ])
                  ]),
                  (user.value.followedMessage != null)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "followedMessage"
                    }, [
                      _createVNode(MkFukidashi, {
                        class: "fukidashi",
                        tail: narrow.value ? 'none' : 'left',
                        negativeMargin: ""
                      }, {
                        default: _withCtx(() => [
                          _createElementVNode("div", _hoisted_8, _toDisplayString(_unref(i18n).ts.messageToFollower), 1 /* TEXT */),
                          _createElementVNode("div", null, [
                            _createVNode(MkSparkle, null, {
                              default: _withCtx(() => [
                                _createVNode(_component_Mfm, {
                                  plain: true,
                                  text: user.value.followedMessage,
                                  author: user.value,
                                  class: "_selectable"
                                }, null, 8 /* PROPS */, ["plain", "text", "author"])
                              ]),
                              _: 1 /* STABLE */
                            })
                          ])
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["tail"])
                    ]))
                    : _createCommentVNode("v-if", true),
                  (user.value.roles.length > 0)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "roles"
                    }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(user.value.roles, (role) => {
                        return (_openBlock(), _createElementBlock("span", {
                          key: role.id,
                          class: "role",
                          style: _normalizeStyle({ '--color': role.color ?? '' })
                        }, [
                          _createVNode(_component_MkA, { to: `/roles/${role.id}` }, {
                            default: _withCtx(() => [
                              (role.iconUrl)
                                ? (_openBlock(), _createElementBlock("img", {
                                  key: 0,
                                  style: "height: 1.3em; vertical-align: -22%;",
                                  src: role.iconUrl
                                }))
                                : _createCommentVNode("v-if", true),
                              _createTextVNode("\n\t\t\t\t\t\t\t\t\t"),
                              _createTextVNode(_toDisplayString(role.name), 1 /* TEXT */)
                            ]),
                            _: 2 /* DYNAMIC */
                          }, 8 /* PROPS */, ["to"])
                        ], 4 /* STYLE */))
                      }), 128 /* KEYED_FRAGMENT */))
                    ]))
                    : _createCommentVNode("v-if", true),
                  (_unref(iAmModerator))
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "moderationNote"
                    }, [
                      (editModerationNote.value || (moderationNote.value != null && moderationNote.value !== ''))
                        ? (_openBlock(), _createBlock(MkTextarea, {
                          key: 0,
                          manualSave: "",
                          modelValue: moderationNote.value,
                          "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((moderationNote).value = $event))
                        }, {
                          label: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.moderationNote), 1 /* TEXT */)
                          ]),
                          caption: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.moderationNoteDescription), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["modelValue"]))
                        : (_openBlock(), _createElementBlock("div", { key: 1 }, [
                          _createVNode(MkButton, {
                            small: "",
                            onClick: _cache[2] || (_cache[2] = ($event: any) => (editModerationNote.value = true))
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.addModerationNote), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]))
                    ]))
                    : _createCommentVNode("v-if", true),
                  (isEditingMemo.value || memoDraft.value)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: _normalizeClass(["memo", {'no-memo': !memoDraft.value}])
                    }, [
                      _createElementVNode("div", _hoisted_9, _toDisplayString(_unref(i18n).ts.memo), 1 /* TEXT */),
                      _withDirectives(_createElementVNode("textarea", {
                        ref_key: "memoTextareaEl", ref: memoTextareaEl,
                        "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((memoDraft).value = $event)),
                        rows: "1",
                        onFocus: _cache[4] || (_cache[4] = ($event: any) => (isEditingMemo.value = true)),
                        onBlur: updateMemo,
                        onInput: adjustMemoTextarea
                      }, null, 32 /* NEED_HYDRATION */), [
                        [_vModelText, memoDraft.value]
                      ])
                    ]))
                    : _createCommentVNode("v-if", true),
                  _createElementVNode("div", { class: "description" }, [
                    _createVNode(MkOmit, null, {
                      default: _withCtx(() => [
                        (user.value.description)
                          ? (_openBlock(), _createBlock(_component_Mfm, {
                            key: 0,
                            text: user.value.description,
                            isNote: false,
                            author: user.value,
                            class: "_selectable"
                          }, null, 8 /* PROPS */, ["text", "isNote", "author"]))
                          : (_openBlock(), _createElementBlock("p", {
                            key: 1,
                            class: "empty"
                          }, _toDisplayString(_unref(i18n).ts.noAccountDescription), 1 /* TEXT */))
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  _createElementVNode("div", { class: "fields system" }, [
                    (user.value.location)
                      ? (_openBlock(), _createElementBlock("dl", {
                        key: 0,
                        class: "field"
                      }, [
                        _createElementVNode("dt", { class: "name" }, [
                          _hoisted_10,
                          _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.location), 1 /* TEXT */)
                        ]),
                        _createElementVNode("dd", _hoisted_11, _toDisplayString(user.value.location), 1 /* TEXT */)
                      ]))
                      : _createCommentVNode("v-if", true),
                    (user.value.birthday)
                      ? (_openBlock(), _createElementBlock("dl", {
                        key: 0,
                        class: "field"
                      }, [
                        _createElementVNode("dt", { class: "name" }, [
                          _hoisted_12,
                          _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.birthday), 1 /* TEXT */)
                        ]),
                        _createElementVNode("dd", _hoisted_13, _toDisplayString(user.value.birthday.replace('-', '/').replace('-', '/')) + " (" + _toDisplayString(_unref(i18n).tsx.yearsOld({ age: age.value })) + ")", 1 /* TEXT */)
                      ]))
                      : _createCommentVNode("v-if", true),
                    _createElementVNode("dl", { class: "field" }, [
                      _createElementVNode("dt", { class: "name" }, [
                        _hoisted_14,
                        _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.registeredDate), 1 /* TEXT */)
                      ]),
                      _createElementVNode("dd", { class: "value" }, [
                        _createTextVNode(_toDisplayString(_unref(dateString)(user.value.createdAt)) + " (", 1 /* TEXT */),
                        _createVNode(_component_MkTime, { time: user.value.createdAt }, null, 8 /* PROPS */, ["time"]),
                        _createTextVNode(")")
                      ])
                    ])
                  ]),
                  (user.value.fields.length > 0)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "fields"
                    }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(user.value.fields, (field, i) => {
                        return (_openBlock(), _createElementBlock("dl", {
                          key: i,
                          class: "field"
                        }, [
                          _createElementVNode("dt", { class: "name" }, [
                            _createVNode(_component_Mfm, {
                              text: field.name,
                              author: user.value,
                              plain: true,
                              colored: false,
                              class: "_selectable"
                            }, null, 8 /* PROPS */, ["text", "author", "plain", "colored"])
                          ]),
                          _createElementVNode("dd", { class: "value" }, [
                            _createVNode(_component_Mfm, {
                              text: field.value,
                              author: user.value,
                              colored: false,
                              class: "_selectable"
                            }, null, 8 /* PROPS */, ["text", "author", "colored"]),
                            (user.value.verifiedLinks.includes(field.value))
                              ? (_openBlock(), _createElementBlock("i", {
                                key: 0,
                                class: _normalizeClass(["ti ti-circle-check", _ctx.$style.verifiedLink])
                              }))
                              : _createCommentVNode("v-if", true)
                          ])
                        ]))
                      }), 128 /* KEYED_FRAGMENT */))
                    ]))
                    : _createCommentVNode("v-if", true),
                  _createElementVNode("div", { class: "status" }, [
                    _createVNode(_component_MkA, { to: _unref(userPage)(user.value, 'notes') }, {
                      default: _withCtx(() => [
                        _createElementVNode("b", null, _toDisplayString(number(user.value.notesCount)), 1 /* TEXT */),
                        _createElementVNode("span", null, _toDisplayString(_unref(i18n).ts.notes), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["to"]),
                    (_unref(isFollowingVisibleForMe)(user.value))
                      ? (_openBlock(), _createBlock(_component_MkA, {
                        key: 0,
                        to: _unref(userPage)(user.value, 'following')
                      }, {
                        default: _withCtx(() => [
                          _createElementVNode("b", null, _toDisplayString(number(user.value.followingCount)), 1 /* TEXT */),
                          _createElementVNode("span", null, _toDisplayString(_unref(i18n).ts.following), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["to"]))
                      : _createCommentVNode("v-if", true),
                    (_unref(isFollowersVisibleForMe)(user.value))
                      ? (_openBlock(), _createBlock(_component_MkA, {
                        key: 0,
                        to: _unref(userPage)(user.value, 'followers')
                      }, {
                        default: _withCtx(() => [
                          _createElementVNode("b", null, _toDisplayString(number(user.value.followersCount)), 1 /* TEXT */),
                          _createElementVNode("span", null, _toDisplayString(_unref(i18n).ts.followers), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["to"]))
                      : _createCommentVNode("v-if", true)
                  ])
                ])
              ]),
              _createElementVNode("div", { class: "contents _gaps" }, [
                (user.value.pinnedNotes.length > 0)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: "_gaps"
                  }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(user.value.pinnedNotes, (note) => {
                      return (_openBlock(), _createBlock(MkNote, {
                        key: note.id,
                        class: "note _panel",
                        note: note,
                        pinned: true
                      }, null, 8 /* PROPS */, ["note", "pinned"]))
                    }), 128 /* KEYED_FRAGMENT */))
                  ]))
                  : (_unref($i) && _unref($i).id === user.value.id)
                    ? (_openBlock(), _createBlock(MkInfo, { key: 1 }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.userPagePinTip), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }))
                  : _createCommentVNode("v-if", true),
                (narrow.value)
                  ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                    _createVNode(_component_MkLazy, null, {
                      default: _withCtx(() => [
                        _createVNode(XFiles, {
                          key: user.value.id,
                          user: user.value,
                          onShowMore: _cache[5] || (_cache[5] = ($event: any) => (emit('showMoreFiles')))
                        }, null, 8 /* PROPS */, ["user"])
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(_component_MkLazy, null, {
                      default: _withCtx(() => [
                        _createVNode(XActivity, {
                          key: user.value.id,
                          user: user.value
                        }, null, 8 /* PROPS */, ["user"])
                      ]),
                      _: 1 /* STABLE */
                    })
                  ], 64 /* STABLE_FRAGMENT */))
                  : _createCommentVNode("v-if", true),
                (!__props.disableNotes)
                  ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                    _createVNode(_component_MkLazy, null, {
                      default: _withCtx(() => [
                        _createVNode(XTimeline, { user: user.value }, null, 8 /* PROPS */, ["user"])
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]))
                  : _createCommentVNode("v-if", true)
              ])
            ]),
            (!narrow.value)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: "sub _gaps",
                style: "container-type: inline-size;"
              }, [
                _createVNode(XFiles, {
                  key: user.value.id,
                  user: user.value,
                  onShowMore: _cache[6] || (_cache[6] = ($event: any) => (emit('showMoreFiles')))
                }, null, 8 /* PROPS */, ["user"]),
                _createVNode(XActivity, {
                  key: user.value.id,
                  user: user.value
                }, null, 8 /* PROPS */, ["user"])
              ]))
              : _createCommentVNode("v-if", true)
          ], 2 /* CLASS */)
        ], 4 /* STYLE */)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["refresher"]))
}
}

})
