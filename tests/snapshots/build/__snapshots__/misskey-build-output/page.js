import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil ti-fw" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-share ti-fw" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-heart-off" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-heart" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil ti-fw" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-link ti-fw" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-share ti-fw" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots ti-fw" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-clock" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-clock-edit" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-clock" })
import { computed, watch, ref, defineAsyncComponent, markRaw } from 'vue'
import * as Misskey from 'misskey-js'
import { url } from '@@/js/config.js'
import type { MenuItem } from '@/types/menu.js'
import XPage from '@/components/page/page.vue'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import MkMediaImage from '@/components/MkMediaImage.vue'
import MkImgWithBlurhash from '@/components/MkImgWithBlurhash.vue'
import MkFollowButton from '@/components/MkFollowButton.vue'
import MkContainer from '@/components/MkContainer.vue'
import MkPagination from '@/components/MkPagination.vue'
import MkPagePreview from '@/components/MkPagePreview.vue'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { deepClone } from '@/utility/clone.js'
import { $i } from '@/i.js'
import { isSupportShare } from '@/utility/navigator.js'
import { instance } from '@/instance.js'
import { getStaticImageUrl } from '@/utility/media-proxy.js'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'
import { useRouter } from '@/router.js'
import { prefer } from '@/preferences.js'
import { getPluginHandlers } from '@/plugin.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'page',
  props: {
    pageName: { type: String, required: true },
    username: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const router = useRouter();
const page = ref<Misskey.entities.Page | null>(null);
const error = ref<any>(null);
const otherPostsPaginator = markRaw(new Paginator('users/pages', {
	limit: 6,
	computedParams: computed(() => page.value ? ({
		userId: page.value.user.id,
	}) : undefined),
}));
const path = computed(() => props.username + '/' + props.pageName);
function fetchPage() {
	page.value = null;
	misskeyApi('pages/show', {
		name: props.pageName,
		username: props.username,
	}).then(async _page => {
		page.value = _page;
		// plugin
		const pageViewInterruptors = getPluginHandlers('page_view_interruptor');
		if (pageViewInterruptors.length > 0) {
			let result = deepClone(_page);
			for (const interruptor of pageViewInterruptors) {
				result = await interruptor.handler(result);
			}
			page.value = result;
		}
	}).catch(err => {
		error.value = err;
	});
}
function share(ev: PointerEvent) {
	if (!page.value) return;
	const menuItems: MenuItem[] = [];
	menuItems.push({
		text: i18n.ts.shareWithNote,
		icon: 'ti ti-pencil',
		action: shareWithNote,
	});
	if (isSupportShare()) {
		menuItems.push({
			text: i18n.ts.share,
			icon: 'ti ti-share',
			action: shareWithNavigator,
		});
	}
	os.popupMenu(menuItems, ev.currentTarget ?? ev.target);
}
function copyLink() {
	if (!page.value) return;
	copyToClipboard(`${url}/@${page.value.user.username}/pages/${page.value.name}`);
}
function shareWithNote() {
	if (!page.value) return;
	os.post({
		initialText: `${page.value.title || page.value.name}\n${url}/@${page.value.user.username}/pages/${page.value.name}`,
		instant: true,
	});
}
function shareWithNavigator() {
	if (!page.value) return;
	navigator.share({
		title: page.value.title ?? page.value.name,
		text: page.value.summary ?? undefined,
		url: `${url}/@${page.value.user.username}/pages/${page.value.name}`,
	});
}
function like() {
	if (!page.value) return;
	os.apiWithDialog('pages/like', {
		pageId: page.value.id,
	}).then(() => {
		page.value!.isLiked = true;
		page.value!.likedCount++;
	});
}
async function unlike() {
	if (!page.value) return;
	const confirm = await os.confirm({
		type: 'warning',
		text: i18n.ts.unlikeConfirm,
	});
	if (confirm.canceled) return;
	os.apiWithDialog('pages/unlike', {
		pageId: page.value.id,
	}).then(() => {
		page.value!.isLiked = false;
		page.value!.likedCount--;
	});
}
function pin(pin: boolean) {
	if (!page.value) return;
	os.apiWithDialog('i/update', {
		pinnedPageId: pin ? page.value.id : null,
	});
}
async function reportAbuse() {
	if (!page.value) return;
	const pageUrl = `${url}/@${props.username}/pages/${props.pageName}`;
	const { dispose } = await os.popupAsyncWithDialog(import('@/components/MkAbuseReportWindow.vue').then(x => x.default), {
		user: page.value.user,
		initialComment: `Page: ${pageUrl}\n-----\n`,
	}, {
		closed: () => dispose(),
	});
}
function showMenu(ev: PointerEvent) {
	if (!page.value) return;
	const menuItems: MenuItem[] = [];
	if ($i && $i.id === page.value.userId) {
		menuItems.push({
			icon: 'ti ti-pencil',
			text: i18n.ts.edit,
			action: () => router.push('/pages/edit/:initPageId', {
				params: {
					initPageId: page.value!.id,
				},
			}),
		});
		if ($i.pinnedPageId === page.value.id) {
			menuItems.push({
				icon: 'ti ti-pinned-off',
				text: i18n.ts.unpin,
				action: () => pin(false),
			});
		} else {
			menuItems.push({
				icon: 'ti ti-pin',
				text: i18n.ts.pin,
				action: () => pin(true),
			});
		}
	} else if ($i && $i.id !== page.value.userId) {
		menuItems.push({
			icon: 'ti ti-exclamation-circle',
			text: i18n.ts.reportAbuse,
			action: reportAbuse,
		});
		if ($i.isModerator || $i.isAdmin) {
			menuItems.push({
				type: 'divider',
			}, {
				icon: 'ti ti-trash',
				text: i18n.ts.delete,
				danger: true,
				action: () => os.confirm({
					type: 'warning',
					text: i18n.ts.deleteConfirm,
				}).then(({ canceled }) => {
					if (canceled || !page.value) return;
					os.apiWithDialog('pages/delete', { pageId: page.value.id });
				}),
			});
		}
	}
	os.popupMenu(menuItems, ev.currentTarget ?? ev.target);
}
watch(() => path.value, fetchPage, { immediate: true });
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: page.value ? page.value.title || page.value.name : i18n.ts.pages,
	...page.value ? {
		avatar: page.value.user,
		path: `/@${page.value.user.username}/pages/${page.value.name}`,
		share: {
			title: page.value.title || page.value.name,
			text: page.value.summary,
		},
	} : {},
}));

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_MkA = _resolveComponent("MkA")
  const _component_MkAcct = _resolveComponent("MkAcct")
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_MkAd = _resolveComponent("MkAd")
  const _component_MkError = _resolveComponent("MkError")
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")
  const _directive_tooltip = _resolveDirective("tooltip")
  const _directive_click_anime = _resolveDirective("click-anime")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 800px;"
        }, [
          _createVNode(_Transition, {
            enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.fadeEnterActive : '',
            leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.fadeLeaveActive : '',
            enterFromClass: _unref(prefer).s.animation ? _ctx.$style.fadeEnterFrom : '',
            leaveToClass: _unref(prefer).s.animation ? _ctx.$style.fadeLeaveTo : '',
            mode: "out-in"
          }, {
            default: _withCtx(() => [
              (page.value)
                ? (_openBlock(), _createElementBlock("div", {
                  key: page.value.id,
                  class: "_gaps"
                }, [
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.pageMain)
                  }, [
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.pageBanner)
                    }, [
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.pageBannerBgRoot)
                      }, [
                        (page.value.eyeCatchingImageId)
                          ? (_openBlock(), _createBlock(MkImgWithBlurhash, {
                            key: 0,
                            class: _normalizeClass(_ctx.$style.pageBannerBg),
                            hash: page.value.eyeCatchingImage?.blurhash,
                            cover: true,
                            forceBlurhash: true
                          }, null, 8 /* PROPS */, ["hash", "cover", "forceBlurhash"]))
                          : (_unref(instance).backgroundImageUrl || _unref(instance).bannerUrl)
                            ? (_openBlock(), _createElementBlock("img", {
                              key: 1,
                              class: _normalizeClass([_ctx.$style.pageBannerBg, _ctx.$style.pageBannerBgFallback1]),
                              src: _unref(getStaticImageUrl)(_unref(instance).backgroundImageUrl ?? _unref(instance).bannerUrl)
                            }))
                          : (_openBlock(), _createElementBlock("div", {
                            key: 2,
                            class: _normalizeClass([_ctx.$style.pageBannerBg, _ctx.$style.pageBannerBgFallback2])
                          }))
                      ]),
                      (page.value.eyeCatchingImageId)
                        ? (_openBlock(), _createElementBlock("div", {
                          key: 0,
                          class: _normalizeClass(_ctx.$style.pageBannerImage)
                        }, [
                          _createVNode(MkMediaImage, {
                            image: page.value.eyeCatchingImage,
                            cover: true,
                            disableImageLink: true,
                            class: _normalizeClass(_ctx.$style.thumbnail)
                          }, null, 8 /* PROPS */, ["image", "cover", "disableImageLink"])
                        ]))
                        : _createCommentVNode("v-if", true),
                      _createElementVNode("div", {
                        class: _normalizeClass(["_gaps_s", _ctx.$style.pageBannerTitle])
                      }, [
                        _createElementVNode("h1", null, _toDisplayString(page.value.title || page.value.name), 1 /* TEXT */),
                        _createElementVNode("div", {
                          class: _normalizeClass(_ctx.$style.pageBannerTitleSub)
                        }, [
                          (page.value.user)
                            ? (_openBlock(), _createElementBlock("div", {
                              key: 0,
                              class: _normalizeClass(_ctx.$style.pageBannerTitleUser)
                            }, [
                              _createVNode(_component_MkAvatar, {
                                user: page.value.user,
                                class: _normalizeClass(_ctx.$style.avatar),
                                indicator: "",
                                link: "",
                                preview: ""
                              }, null, 8 /* PROPS */, ["user"]),
                              _createTextVNode(),
                              _createVNode(_component_MkA, { to: `/@${__props.username}` }, {
                                default: _withCtx(() => [
                                  _createVNode(_component_MkUserName, {
                                    user: page.value.user,
                                    nowrap: false
                                  }, null, 8 /* PROPS */, ["user", "nowrap"])
                                ]),
                                _: 1 /* STABLE */
                              }, 8 /* PROPS */, ["to"])
                            ]))
                            : _createCommentVNode("v-if", true),
                          _createElementVNode("div", {
                            class: _normalizeClass(_ctx.$style.pageBannerTitleSubActions)
                          }, [
                            (page.value.userId === _unref($i)?.id)
                              ? (_openBlock(), _createBlock(_component_MkA, {
                                key: 0,
                                to: `/pages/edit/${page.value.id}`,
                                class: _normalizeClass(["_button", _ctx.$style.generalActionButton])
                              }, {
                                default: _withCtx(() => [
                                  _hoisted_1
                                ]),
                                _: 1 /* STABLE */
                              }, 8 /* PROPS */, ["to"]))
                              : _createCommentVNode("v-if", true),
                            _createElementVNode("button", {
                              class: _normalizeClass(["_button", _ctx.$style.generalActionButton]),
                              onClick: share
                            }, [
                              _hoisted_2
                            ])
                          ])
                        ])
                      ])
                    ]),
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.pageContent)
                    }, [
                      _createVNode(XPage, { page: page.value }, null, 8 /* PROPS */, ["page"])
                    ]),
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.pageActions)
                    }, [
                      _createElementVNode("div", null, [
                        (page.value.isLiked)
                          ? (_openBlock(), _createBlock(MkButton, {
                            key: 0,
                            class: "button",
                            asLike: "",
                            primary: "",
                            onClick: _cache[0] || (_cache[0] = ($event: any) => (unlike()))
                          }, {
                            default: _withCtx(() => [
                              _hoisted_3,
                              (page.value.likedCount > 0)
                                ? (_openBlock(), _createElementBlock("span", {
                                  key: 0,
                                  class: "count"
                                }, _toDisplayString(page.value.likedCount), 1 /* TEXT */))
                                : _createCommentVNode("v-if", true)
                            ]),
                            _: 1 /* STABLE */
                          }))
                          : (_openBlock(), _createBlock(MkButton, {
                            key: 1,
                            class: "button",
                            asLike: "",
                            onClick: _cache[1] || (_cache[1] = ($event: any) => (like()))
                          }, {
                            default: _withCtx(() => [
                              _hoisted_4,
                              (page.value.likedCount > 0)
                                ? (_openBlock(), _createElementBlock("span", {
                                  key: 0,
                                  class: "count"
                                }, _toDisplayString(page.value.likedCount), 1 /* TEXT */))
                                : _createCommentVNode("v-if", true)
                            ]),
                            _: 1 /* STABLE */
                          }))
                      ]),
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.other)
                      }, [
                        (page.value.userId === _unref($i)?.id)
                          ? (_openBlock(), _createBlock(_component_MkA, {
                            key: 0,
                            to: `/pages/edit/${page.value.id}`,
                            class: _normalizeClass(["_button", _ctx.$style.generalActionButton])
                          }, {
                            default: _withCtx(() => [
                              _hoisted_5
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["to"]))
                          : _createCommentVNode("v-if", true),
                        _createElementVNode("button", {
                          class: _normalizeClass(["_button", _ctx.$style.generalActionButton]),
                          onClick: copyLink
                        }, [
                          _hoisted_6
                        ]),
                        _createElementVNode("button", {
                          class: _normalizeClass(["_button", _ctx.$style.generalActionButton]),
                          onClick: share
                        }, [
                          _hoisted_7
                        ]),
                        (_unref($i))
                          ? (_openBlock(), _createElementBlock("button", {
                            key: 0,
                            class: _normalizeClass(["_button", _ctx.$style.generalActionButton]),
                            onClick: showMenu
                          }, [
                            _hoisted_8
                          ]))
                          : _createCommentVNode("v-if", true)
                      ])
                    ]),
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.pageUser)
                    }, [
                      _createVNode(_component_MkAvatar, {
                        user: page.value.user,
                        class: _normalizeClass(_ctx.$style.avatar),
                        link: "",
                        preview: ""
                      }, null, 8 /* PROPS */, ["user"]),
                      _createVNode(_component_MkA, { to: `/@${__props.username}` }, {
                        default: _withCtx(() => [
                          _createVNode(_component_MkUserName, {
                            user: page.value.user,
                            class: _normalizeClass(_ctx.$style.name)
                          }, null, 8 /* PROPS */, ["user"]),
                          _createVNode(_component_MkAcct, {
                            user: page.value.user,
                            class: _normalizeClass(_ctx.$style.acct)
                          }, null, 8 /* PROPS */, ["user"])
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["to"])
                    ]),
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.pageDate)
                    }, [
                      _createElementVNode("div", null, [
                        _hoisted_9,
                        _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.createdAt) + ": ", 1 /* TEXT */),
                        _createVNode(_component_MkTime, {
                          time: page.value.createdAt,
                          mode: "detail"
                        }, null, 8 /* PROPS */, ["time"])
                      ]),
                      (page.value.createdAt != page.value.updatedAt)
                        ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                          _hoisted_10,
                          _createTextVNode(),
                          _toDisplayString(_unref(i18n).ts.updatedAt),
                          _createTextVNode(": "),
                          _createVNode(_component_MkTime, {
                            time: page.value.updatedAt,
                            mode: "detail"
                          }, null, 8 /* PROPS */, ["time"])
                        ]))
                        : _createCommentVNode("v-if", true)
                    ])
                  ]),
                  _createVNode(_component_MkAd, { preferForms: ['horizontal', 'horizontal-big'] }, null, 8 /* PROPS */, ["preferForms"]),
                  _createVNode(MkContainer, {
                    "max-height": 300,
                    foldable: true,
                    class: "other"
                  }, {
                    icon: _withCtx(() => [
                      _hoisted_11
                    ]),
                    header: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.recentPosts), 1 /* TEXT */)
                    ]),
                    default: _withCtx(() => [
                      _createVNode(MkPagination, {
                        paginator: _unref(otherPostsPaginator),
                        class: _normalizeClass(["_gaps", _ctx.$style.relatedPagesRoot])
                      }, {
                        default: _withCtx(({items}) => [
                          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (page) => {
                            return (_openBlock(), _createBlock(MkPagePreview, {
                              key: page.value.id,
                              page: page.value,
                              class: _normalizeClass(_ctx.$style.relatedPagesItem)
                            }, null, 8 /* PROPS */, ["page"]))
                          }), 128 /* KEYED_FRAGMENT */))
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["paginator"])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["max-height", "foldable"])
                ]))
                : (error.value)
                  ? (_openBlock(), _createBlock(_component_MkError, {
                    key: 1,
                    onRetry: _cache[2] || (_cache[2] = ($event: any) => (fetchPage()))
                  }))
                : (_openBlock(), _createBlock(_component_MkLoading, { key: 2 }))
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass"])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
