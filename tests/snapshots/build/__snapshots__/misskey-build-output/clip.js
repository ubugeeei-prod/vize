import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-heart" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-heart" })
import { computed, watch, provide, ref, markRaw } from 'vue'
import * as Misskey from 'misskey-js'
import { url } from '@@/js/config.js'
import type { MenuItem } from '@/types/menu.js'
import type { PageHeaderItem } from '@/types/page-header.js'
import MkNotesTimeline from '@/components/MkNotesTimeline.vue'
import { $i } from '@/i.js'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { definePage } from '@/page.js'
import MkButton from '@/components/MkButton.vue'
import { clipsCache } from '@/cache.js'
import { isSupportShare } from '@/utility/navigator.js'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'
import { genEmbedCode } from '@/utility/get-embed-code.js'
import { assertServerContext, serverContext } from '@/server-context.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'clip',
  props: {
    clipId: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
// contextは非ログイン状態の情報しかないためログイン時は利用できない
const CTX_CLIP = !$i && assertServerContext(serverContext, 'clip') ? serverContext.clip : null;
const clip = ref<Misskey.entities.Clip | null>(CTX_CLIP);
const favorited = ref(false);
const paginator = markRaw(new Paginator('clips/notes', {
	limit: 10,
	canSearch: true,
	computedParams: computed(() => ({
		clipId: props.clipId,
	})),
}));
const isOwned = computed<boolean | null>(() => $i && clip.value && ($i.id === clip.value.userId));
watch(() => props.clipId, async () => {
	if (CTX_CLIP && CTX_CLIP.id === props.clipId) {
		clip.value = CTX_CLIP;
		return;
	}
	clip.value = await misskeyApi('clips/show', {
		clipId: props.clipId,
	});
	favorited.value = clip.value!.isFavorited ?? false;
}, {
	immediate: true,
});
provide('currentClip', clip);
function favorite() {
	os.apiWithDialog('clips/favorite', {
		clipId: props.clipId,
	}).then(() => {
		favorited.value = true;
	});
}
async function unfavorite() {
	const confirm = await os.confirm({
		type: 'warning',
		text: i18n.ts.unfavoriteConfirm,
	});
	if (confirm.canceled) return;
	os.apiWithDialog('clips/unfavorite', {
		clipId: props.clipId,
	}).then(() => {
		favorited.value = false;
	});
}
const headerActions = computed<PageHeaderItem[] | null>(() => clip.value && isOwned.value ? [{
	icon: 'ti ti-pencil',
	text: i18n.ts.edit,
	handler: async (): Promise<void> => {
		if (clip.value == null) return;

		const { canceled, result } = await os.form(clip.value.name, {
			name: {
				type: 'string',
				label: i18n.ts.name,
				default: clip.value.name,
			},
			description: {
				type: 'string',
				required: false,
				multiline: true,
				treatAsMfm: true,
				label: i18n.ts.description,
				default: clip.value.description,
			},
			isPublic: {
				type: 'boolean',
				label: i18n.ts.public,
				default: clip.value.isPublic,
			},
		});

		if (canceled) return;

		os.apiWithDialog('clips/update', {
			clipId: clip.value.id,
			...result,
		});

		clipsCache.delete();
	},
}, ...(clip.value.isPublic ? [{
	icon: 'ti ti-share',
	text: i18n.ts.share,
	handler: (ev): void => {
		const menuItems: MenuItem[] = [];

		menuItems.push({
			icon: 'ti ti-link',
			text: i18n.ts.copyUrl,
			action: () => {
				copyToClipboard(`${url}/clips/${clip.value!.id}`);
			},
		}, {
			icon: 'ti ti-code',
			text: i18n.ts.embed,
			action: () => {
				genEmbedCode('clips', clip.value!.id);
			},
		});

		if (isSupportShare()) {
			menuItems.push({
				icon: 'ti ti-share',
				text: i18n.ts.share,
				action: async () => {
					navigator.share({
						title: clip.value!.name,
						text: clip.value!.description ?? '',
						url: `${url}/clips/${clip.value!.id}`,
					});
				},
			});
		}

		os.popupMenu(menuItems, ev.currentTarget ?? ev.target);
	},
}] satisfies PageHeaderItem[] : []), {
	icon: 'ti ti-trash',
	text: i18n.ts.delete,
	danger: true,
	handler: async (): Promise<void> => {
		if (clip.value == null) return;

		const { canceled } = await os.confirm({
			type: 'warning',
			text: i18n.tsx.deleteAreYouSure({ x: clip.value.name }),
		});
		if (canceled) return;

		await os.apiWithDialog('clips/delete', {
			clipId: clip.value.id,
		});

		clipsCache.delete();
	},
}] satisfies PageHeaderItem[] : null);
definePage(() => ({
	title: clip.value ? clip.value.name : i18n.ts.clip,
	icon: 'ti ti-paperclip',
}));

return (_ctx: any,_cache: any) => {
  const _component_Mfm = _resolveComponent("Mfm")
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createBlock(_component_PageWithHeader, { actions: headerActions.value }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 800px;"
        }, [
          (clip.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "_gaps"
            }, [
              _createElementVNode("div", { class: "_panel" }, [
                _createElementVNode("div", {
                  class: _normalizeClass(["_gaps_s", _ctx.$style.description])
                }, [
                  (clip.value.description)
                    ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                      _createVNode(_component_Mfm, {
                        text: clip.value.description,
                        isNote: false
                      }, null, 8 /* PROPS */, ["text", "isNote"])
                    ]))
                    : (_openBlock(), _createElementBlock("div", { key: 1 }, "(" + _toDisplayString(_unref(i18n).ts.noDescription) + ")", 1 /* TEXT */)),
                  _createElementVNode("div", null, [
                    (favorited.value)
                      ? (_openBlock(), _createBlock(MkButton, {
                        key: 0,
                        asLike: "",
                        rounded: "",
                        primary: "",
                        onClick: _cache[0] || (_cache[0] = ($event: any) => (unfavorite()))
                      }, {
                        default: _withCtx(() => [
                          _hoisted_1,
                          (clip.value.favoritedCount > 0)
                            ? (_openBlock(), _createElementBlock("span", {
                              key: 0,
                              style: "margin-left: 6px;"
                            }, _toDisplayString(clip.value.favoritedCount), 1 /* TEXT */))
                            : _createCommentVNode("v-if", true)
                        ]),
                        _: 1 /* STABLE */
                      }))
                      : (_openBlock(), _createBlock(MkButton, {
                        key: 1,
                        asLike: "",
                        rounded: "",
                        onClick: _cache[1] || (_cache[1] = ($event: any) => (favorite()))
                      }, {
                        default: _withCtx(() => [
                          _hoisted_2,
                          (clip.value.favoritedCount > 0)
                            ? (_openBlock(), _createElementBlock("span", {
                              key: 0,
                              style: "margin-left: 6px;"
                            }, _toDisplayString(clip.value.favoritedCount), 1 /* TEXT */))
                            : _createCommentVNode("v-if", true)
                        ]),
                        _: 1 /* STABLE */
                      }))
                  ])
                ]),
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.user)
                }, [
                  _createVNode(_component_MkAvatar, {
                    user: clip.value.user,
                    class: _normalizeClass(_ctx.$style.avatar),
                    indicator: "",
                    link: "",
                    preview: ""
                  }, null, 8 /* PROPS */, ["user"]),
                  _createTextVNode(),
                  _createVNode(_component_MkUserName, {
                    user: clip.value.user,
                    nowrap: false
                  }, null, 8 /* PROPS */, ["user", "nowrap"])
                ])
              ]),
              _createVNode(MkNotesTimeline, {
                paginator: _unref(paginator),
                detail: true
              }, null, 8 /* PROPS */, ["paginator", "detail"])
            ]))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions"]))
}
}

})
