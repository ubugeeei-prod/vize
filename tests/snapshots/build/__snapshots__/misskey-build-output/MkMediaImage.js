import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveDynamicComponent as _resolveDynamicComponent, toDisplayString as _toDisplayString, mergeProps as _mergeProps, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye-exclamation" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-photo" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye-exclamation" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots", style: "vertical-align: middle;" })
import { watch, ref, computed } from 'vue'
import * as Misskey from 'misskey-js'
import type { MenuItem } from '@/types/menu.js'
import { copyToClipboard } from '@/utility/copy-to-clipboard'
import { getStaticImageUrl } from '@/utility/media-proxy.js'
import bytes from '@/filters/bytes.js'
import MkImgWithBlurhash from '@/components/MkImgWithBlurhash.vue'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'
import { $i, iAmModerator } from '@/i.js'
import { prefer } from '@/preferences.js'
import { shouldHideFileByDefault, canRevealFile } from '@/utility/sensitive-file.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkMediaImage',
  props: {
    image: { type: null, required: true },
    raw: { type: Boolean, required: false },
    cover: { type: Boolean, required: false, default: false },
    disableImageLink: { type: Boolean, required: false, default: false },
    controls: { type: Boolean, required: false, default: true }
  },
  setup(__props: any) {

const props = __props
const hide = ref(true);
const url = computed(() => (props.raw || prefer.s.loadRawImages)
	? props.image.url
	: prefer.s.disableShowingAnimatedImages
		? getStaticImageUrl(props.image.url)
		: props.image.thumbnailUrl!,
);
async function reveal(ev: PointerEvent) {
	if (!props.controls) {
		return;
	}
	if (hide.value) {
		ev.stopPropagation();
		if (!(await canRevealFile(props.image))) {
			return;
		}
		hide.value = false;
	}
}
// Plugin:register_note_view_interruptor を使って書き換えられる可能性があるためwatchする
watch(() => props.image, (newImage) => {
	hide.value = shouldHideFileByDefault(newImage);
}, {
	deep: true,
	immediate: true,
});
function getMenu() {
	const menuItems: MenuItem[] = [];
	menuItems.push({
		text: i18n.ts.hide,
		icon: 'ti ti-eye-off',
		action: () => {
			hide.value = true;
		},
	});
	if (iAmModerator) {
		menuItems.push({
			text: props.image.isSensitive ? i18n.ts.unmarkAsSensitive : i18n.ts.markAsSensitive,
			icon: 'ti ti-eye-exclamation',
			danger: true,
			action: async () => {
				const { canceled } = await os.confirm({
					type: 'warning',
					text: props.image.isSensitive ? i18n.ts.unmarkAsSensitiveConfirm : i18n.ts.markAsSensitiveConfirm,
				});
				if (canceled) return;
				os.apiWithDialog('drive/files/update', {
					fileId: props.image.id,
					isSensitive: !props.image.isSensitive,
				});
			},
		});
	}
	const details: MenuItem[] = [];
	if ($i?.id === props.image.userId) {
		details.push({
			type: 'link',
			text: i18n.ts._fileViewer.title,
			icon: 'ti ti-info-circle',
			to: `/my/drive/file/${props.image.id}`,
		});
	}
	if (iAmModerator) {
		details.push({
			type: 'link',
			text: i18n.ts.moderation,
			icon: 'ti ti-photo-exclamation',
			to: `/admin/file/${props.image.id}`,
		});
	}
	if (details.length > 0) {
		menuItems.push({ type: 'divider' }, ...details);
	}
	if (prefer.s.devMode) {
		menuItems.push({ type: 'divider' }, {
			icon: 'ti ti-hash',
			text: i18n.ts.copyFileId,
			action: () => {
				copyToClipboard(props.image.id);
			},
		});
	}
	return menuItems;
}
function showMenu(ev: PointerEvent) {
	os.popupMenu(getMenu(), (ev.currentTarget ?? ev.target ?? undefined) as HTMLElement | undefined);
}
function onContextmenu(ev: PointerEvent) {
	os.contextMenu(getMenu(), ev);
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([hide.value ? _ctx.$style.hidden : _ctx.$style.visible, (__props.image.isSensitive && _unref(prefer).s.highlightSensitiveMedia) && _ctx.$style.sensitive]),
      onClick: reveal,
      onContextmenu: _withModifiers(onContextmenu, ["stop"])
    }, [ _createVNode(_resolveDynamicComponent(__props.disableImageLink ? 'div' : 'a'), _mergeProps(__props.disableImageLink ? {
  			title: __props.image.name,
  			class: _ctx.$style.imageContainer,
  		} : {
  			title: __props.image.name,
  			class: _ctx.$style.imageContainer,
  			href: __props.image.url,
  			style: 'cursor: zoom-in;'
  		}, {  }), {
        default: _withCtx(() => [
          (_unref(prefer).s.enableHighQualityImagePlaceholders)
            ? (_openBlock(), _createBlock(MkImgWithBlurhash, {
              key: 0,
              hash: __props.image.blurhash,
              src: (_unref(prefer).s.dataSaver.media && hide.value) ? null : url.value,
              forceBlurhash: hide.value,
              cover: hide.value || __props.cover,
              alt: __props.image.comment || __props.image.name,
              title: __props.image.comment || __props.image.name,
              width: __props.image.properties.width,
              height: __props.image.properties.height,
              style: _normalizeStyle(hide.value ? 'filter: brightness(0.7);' : null),
              class: _normalizeClass(_ctx.$style.image)
            }, null, 12 /* STYLE, PROPS */, ["hash", "src", "forceBlurhash", "cover", "alt", "title", "width", "height"]))
            : (_unref(prefer).s.dataSaver.media || hide.value)
              ? (_openBlock(), _createElementBlock("div", {
                key: 1,
                title: __props.image.comment || __props.image.name,
                style: _normalizeStyle(hide.value ? 'background: #888;' : null),
                class: _normalizeClass(_ctx.$style.image)
              }))
            : (_openBlock(), _createElementBlock("img", {
              key: 2,
              src: url.value,
              alt: __props.image.comment || __props.image.name,
              title: __props.image.comment || __props.image.name,
              class: _normalizeClass(_ctx.$style.image)
            }))
        ]),
        _: 1 /* STABLE */
      }, 16 /* FULL_PROPS */), (hide.value) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: _normalizeClass(_ctx.$style.hiddenText)
        }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.hiddenTextWrapper)
          }, [ (__props.image.isSensitive) ? (_openBlock(), _createElementBlock("b", {
                key: 0,
                style: "display: block;"
              }, [ _hoisted_1, _createTextVNode(), _toDisplayString(_unref(i18n).ts.sensitive), _toDisplayString(_unref(prefer).s.dataSaver.media ? ` (${_unref(i18n).ts.image}${__props.image.size ? ' ' + bytes(__props.image.size) : ''})` : '') ])) : (_openBlock(), _createElementBlock("b", {
                key: 1,
                style: "display: block;"
              }, [ _hoisted_2, _createTextVNode(), _toDisplayString(_unref(prefer).s.dataSaver.media && __props.image.size ? bytes(__props.image.size) : _unref(i18n).ts.image) ])), (__props.controls) ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                style: "display: block;"
              }, _toDisplayString(_unref(i18n).ts.clickToShow), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]) ])) : (__props.controls) ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.indicators)
            }, [ (['image/gif', 'image/apng'].includes(__props.image.type)) ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.indicator)
                }, "GIF")) : _createCommentVNode("v-if", true), (__props.image.comment) ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.indicator)
                }, "ALT")) : _createCommentVNode("v-if", true), (__props.image.isSensitive) ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.indicator),
                  style: "color: var(--MI_THEME-warn);",
                  title: _unref(i18n).ts.sensitive
                }, [ _hoisted_3 ])) : _createCommentVNode("v-if", true) ]), _createElementVNode("button", {
              class: _normalizeClass(["_button", _ctx.$style.menu]),
              onClick: _withModifiers(showMenu, ["stop"])
            }, [ _hoisted_4 ]), _createElementVNode("i", {
              class: _normalizeClass(["ti ti-eye-off", _ctx.$style.hide]),
              onClick: _cache[0] || (_cache[0] = _withModifiers(($event: any) => (hide.value = true), ["stop"]))
            }) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ], 34 /* CLASS, NEED_HYDRATION */))
}
}

})
