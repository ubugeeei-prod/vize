import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "title" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-clock" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-heart-off" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-heart" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil ti-fw" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-repeat ti-fw" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-link ti-fw" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-share ti-fw" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots ti-fw" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-clock" })
import { computed, watch, ref, defineAsyncComponent, markRaw } from 'vue'
import * as Misskey from 'misskey-js'
import { url } from '@@/js/config.js'
import type { MenuItem } from '@/types/menu.js'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import MkContainer from '@/components/MkContainer.vue'
import MkPagination from '@/components/MkPagination.vue'
import MkGalleryPostPreview from '@/components/MkGalleryPostPreview.vue'
import MkFollowButton from '@/components/MkFollowButton.vue'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { prefer } from '@/preferences.js'
import { $i } from '@/i.js'
import { isSupportShare } from '@/utility/navigator.js'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'
import { useRouter } from '@/router.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'post',
  props: {
    postId: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const router = useRouter();
const post = ref<Misskey.entities.GalleryPost | null>(null);
const error = ref<any>(null);
const otherPostsPaginator = markRaw(new Paginator('users/gallery/posts', {
	limit: 6,
	computedParams: computed(() => ({
		userId: post.value!.user.id,
	})),
}));
function fetchPost() {
	post.value = null;
	misskeyApi('gallery/posts/show', {
		postId: props.postId,
	}).then(_post => {
		post.value = _post;
	}).catch(_error => {
		error.value = _error;
	});
}
function copyLink() {
	if (!post.value) return;
	copyToClipboard(`${url}/gallery/${post.value.id}`);
}
function share() {
	if (!post.value) return;
	navigator.share({
		title: post.value.title,
		text: post.value.description ?? undefined,
		url: `${url}/gallery/${post.value.id}`,
	});
}
function shareWithNote() {
	if (!post.value) return;
	os.post({
		initialText: `${post.value.title} ${url}/gallery/${post.value.id}`,
	});
}
function like() {
	if (!post.value) return;
	os.apiWithDialog('gallery/posts/like', {
		postId: props.postId,
	}).then(() => {
		post.value!.isLiked = true;
		post.value!.likedCount++;
	});
}
async function unlike() {
	if (!post.value) return;
	const confirm = await os.confirm({
		type: 'warning',
		text: i18n.ts.unlikeConfirm,
	});
	if (confirm.canceled) return;
	os.apiWithDialog('gallery/posts/unlike', {
		postId: props.postId,
	}).then(() => {
		post.value!.isLiked = false;
		post.value!.likedCount--;
	});
}
function edit() {
	router.push('/gallery/:postId/edit', {
		params: {
			postId: props.postId,
		},
	});
}
async function reportAbuse() {
	if (!post.value) return;
	const pageUrl = `${url}/gallery/${post.value.id}`;
	const { dispose } = await os.popupAsyncWithDialog(import('@/components/MkAbuseReportWindow.vue').then(x => x.default), {
		user: post.value.user,
		initialComment: `Post: ${pageUrl}\n-----\n`,
	}, {
		closed: () => dispose(),
	});
}
function showMenu(ev: PointerEvent) {
	if (!post.value) return;
	const menuItems: MenuItem[] = [];
	if ($i && $i.id !== post.value.userId) {
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
					if (canceled || !post.value) return;
					os.apiWithDialog('gallery/posts/delete', { postId: post.value.id });
				}),
			});
		}
	}
	os.popupMenu(menuItems, ev.currentTarget ?? ev.target);
}
watch(() => props.postId, fetchPost, { immediate: true });
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: post.value ? post.value.title : i18n.ts.gallery,
	...post.value ? {
		avatar: post.value.user,
	} : {},
}));

return (_ctx: any,_cache: any) => {
  const _component_Mfm = _resolveComponent("Mfm")
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_MkAcct = _resolveComponent("MkAcct")
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
          style: "--MI_SPACER-w: 1000px; --MI_SPACER-min: 16px; --MI_SPACER-max: 32px;"
        }, [
          _createElementVNode("div", { class: "_root" }, [
            _createVNode(_Transition, {
              name: _unref(prefer).s.animation ? 'fade' : '',
              mode: "out-in"
            }, {
              default: _withCtx(() => [
                (post.value)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: "rkxwuolj"
                  }, [
                    _createElementVNode("div", { class: "files" }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(post.value.files, (file) => {
                        return (_openBlock(), _createElementBlock("div", {
                          key: file.id,
                          class: "file"
                        }, [
                          _createElementVNode("img", { src: file.url }, null, 8 /* PROPS */, ["src"])
                        ]))
                      }), 128 /* KEYED_FRAGMENT */))
                    ]),
                    _createElementVNode("div", { class: "body" }, [
                      _createElementVNode("div", _hoisted_1, _toDisplayString(post.value.title), 1 /* TEXT */),
                      _createElementVNode("div", { class: "description" }, [
                        (post.value.description != null)
                          ? (_openBlock(), _createBlock(_component_Mfm, {
                            key: 0,
                            text: post.value.description
                          }, null, 8 /* PROPS */, ["text"]))
                          : _createCommentVNode("v-if", true)
                      ]),
                      _createElementVNode("div", { class: "info" }, [
                        _hoisted_2,
                        _createTextVNode(),
                        _createVNode(_component_MkTime, {
                          time: post.value.createdAt,
                          mode: "detail"
                        }, null, 8 /* PROPS */, ["time"])
                      ]),
                      _createElementVNode("div", { class: "actions" }, [
                        _createElementVNode("div", { class: "like" }, [
                          (post.value.isLiked)
                            ? (_openBlock(), _createBlock(MkButton, {
                              key: 0,
                              class: "button",
                              primary: "",
                              onClick: _cache[0] || (_cache[0] = ($event: any) => (unlike()))
                            }, {
                              default: _withCtx(() => [
                                _hoisted_3,
                                (post.value.likedCount > 0)
                                  ? (_openBlock(), _createElementBlock("span", {
                                    key: 0,
                                    class: "count"
                                  }, _toDisplayString(post.value.likedCount), 1 /* TEXT */))
                                  : _createCommentVNode("v-if", true)
                              ]),
                              _: 1 /* STABLE */
                            }))
                            : (_openBlock(), _createBlock(MkButton, {
                              key: 1,
                              class: "button",
                              onClick: _cache[1] || (_cache[1] = ($event: any) => (like()))
                            }, {
                              default: _withCtx(() => [
                                _hoisted_4,
                                (post.value.likedCount > 0)
                                  ? (_openBlock(), _createElementBlock("span", {
                                    key: 0,
                                    class: "count"
                                  }, _toDisplayString(post.value.likedCount), 1 /* TEXT */))
                                  : _createCommentVNode("v-if", true)
                              ]),
                              _: 1 /* STABLE */
                            }))
                        ]),
                        _createElementVNode("div", { class: "other" }, [
                          (_unref($i) && _unref($i).id === post.value.user.id)
                            ? (_openBlock(), _createElementBlock("button", {
                              key: 0,
                              class: "_button",
                              onClick: edit
                            }, [
                              _hoisted_5
                            ]))
                            : _createCommentVNode("v-if", true),
                          _createElementVNode("button", {
                            class: "_button",
                            onClick: shareWithNote
                          }, [
                            _hoisted_6
                          ]),
                          _createElementVNode("button", {
                            class: "_button",
                            onClick: copyLink
                          }, [
                            _hoisted_7
                          ]),
                          (_unref(isSupportShare)())
                            ? (_openBlock(), _createElementBlock("button", {
                              key: 0,
                              class: "_button",
                              onClick: share
                            }, [
                              _hoisted_8
                            ]))
                            : _createCommentVNode("v-if", true),
                          (_unref($i) && _unref($i).id !== post.value.user.id)
                            ? (_openBlock(), _createElementBlock("button", {
                              key: 0,
                              class: "_button",
                              onClick: showMenu
                            }, [
                              _hoisted_9
                            ]))
                            : _createCommentVNode("v-if", true)
                        ])
                      ]),
                      _createElementVNode("div", { class: "user" }, [
                        _createVNode(_component_MkAvatar, {
                          user: post.value.user,
                          class: "avatar",
                          link: "",
                          preview: ""
                        }, null, 8 /* PROPS */, ["user"]),
                        _createElementVNode("div", { class: "name" }, [
                          _createVNode(_component_MkUserName, {
                            user: post.value.user,
                            style: "display: block;"
                          }, null, 8 /* PROPS */, ["user"]),
                          _createVNode(_component_MkAcct, { user: post.value.user }, null, 8 /* PROPS */, ["user"])
                        ])
                      ])
                    ]),
                    _createVNode(_component_MkAd, { preferForms: ['horizontal', 'horizontal-big'] }, null, 8 /* PROPS */, ["preferForms"]),
                    _createVNode(MkContainer, {
                      "max-height": 300,
                      foldable: true,
                      class: "other"
                    }, {
                      icon: _withCtx(() => [
                        _hoisted_10
                      ]),
                      header: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.recentPosts), 1 /* TEXT */)
                      ]),
                      default: _withCtx(() => [
                        _createVNode(MkPagination, { paginator: _unref(otherPostsPaginator) }, {
                          default: _withCtx(({items}) => [
                            _createElementVNode("div", { class: "sdrarzaf" }, [
                              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (post) => {
                                return (_openBlock(), _createBlock(MkGalleryPostPreview, {
                                  key: post.value.id,
                                  post: post.value,
                                  class: "post"
                                }, null, 8 /* PROPS */, ["post"]))
                              }), 128 /* KEYED_FRAGMENT */))
                            ])
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
                      onRetry: _cache[2] || (_cache[2] = ($event: any) => (fetchPost()))
                    }))
                  : (_openBlock(), _createBlock(_component_MkLoading, { key: 2 }))
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["name"])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
