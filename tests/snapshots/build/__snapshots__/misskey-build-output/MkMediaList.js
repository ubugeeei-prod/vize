import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, renderList as _renderList, normalizeClass as _normalizeClass, unref as _unref } from "vue"

import { computed, onMounted, onUnmounted, useTemplateRef } from 'vue'
import * as Misskey from 'misskey-js'
import PhotoSwipeLightbox from 'photoswipe/lightbox'
import PhotoSwipe from 'photoswipe'
import 'photoswipe/style.css'
import { FILE_TYPE_BROWSERSAFE } from '@@/js/const.js'
import XBanner from '@/components/MkMediaBanner.vue'
import XImage from '@/components/MkMediaImage.vue'
import XVideo from '@/components/MkMediaVideo.vue'
import * as os from '@/os.js'
import { focusParent } from '@/utility/focus.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkMediaList',
  props: {
    mediaList: { type: Array, required: true },
    raw: { type: Boolean, required: false }
  },
  setup(__props: any, { expose: __expose }) {

const props = __props
const gallery = useTemplateRef('gallery');
const pswpZIndex = os.claimZIndex('middle');
window.document.documentElement.style.setProperty('--mk-pswp-root-z-index', pswpZIndex.toString());
const count = computed(() => props.mediaList.filter(media => previewable(media)).length);
let lightbox: PhotoSwipeLightbox | null = null;
let activeEl: HTMLElement | null = null;
const popstateHandler = (): void => {
	if (lightbox?.pswp && lightbox.pswp.isOpen === true) {
		lightbox.pswp.close();
	}
};
async function calcAspectRatio() {
	if (!gallery.value) return;
	const img = props.mediaList[0];
	if (props.mediaList.length !== 1 || !(img.properties.width && img.properties.height)) {
		gallery.value.style.aspectRatio = '';
		return;
	}
	const ratioMax = (ratio: number) => {
		if (img.properties.width == null || img.properties.height == null) return '';
		return `${Math.max(ratio, img.properties.width / img.properties.height).toString()} / 1`;
	};
	switch (prefer.s.mediaListWithOneImageAppearance) {
		case '16_9':
			gallery.value.style.aspectRatio = ratioMax(16 / 9);
			break;
		case '1_1':
			gallery.value.style.aspectRatio = ratioMax(1 / 1);
			break;
		case '2_3':
			gallery.value.style.aspectRatio = ratioMax(2 / 3);
			break;
		default:
			gallery.value.style.aspectRatio = '';
			break;
	}
}
onMounted(() => {
	calcAspectRatio();
	if (gallery.value == null) return; // TSを黙らすため
	lightbox = new PhotoSwipeLightbox({
		dataSource: props.mediaList
			.filter(media => {
				if (media.type === 'image/svg+xml') return true; // svgのwebpublicはpngなのでtrue
				return media.type.startsWith('image') && FILE_TYPE_BROWSERSAFE.includes(media.type);
			})
			.map(media => {
				const item = {
					src: media.url,
					w: media.properties.width,
					h: media.properties.height,
					// eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
					alt: media.comment || media.name,
					// eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
					comment: media.comment || media.name,
				};
				if (media.properties.orientation != null && media.properties.orientation >= 5) {
					[item.w, item.h] = [item.h, item.w];
				}
				return item;
			}),
		gallery: gallery.value,
		mainClass: 'pswp',
		children: '.image',
		thumbSelector: '.image',
		loop: false,
		padding: window.innerWidth > 500 ? {
			top: 32,
			bottom: 90,
			left: 32,
			right: 32,
		} : {
			top: 0,
			bottom: 78,
			left: 0,
			right: 0,
		},
		imageClickAction: 'close',
		tapAction: 'close',
		bgOpacity: 1,
		showAnimationDuration: 100,
		hideAnimationDuration: 100,
		returnFocus: false,
		pswpModule: PhotoSwipe,
	});
	lightbox.addFilter('itemData', (itemData) => {
		// element is children
		const { element } = itemData;
		const id = element?.dataset.id;
		const file = props.mediaList.find(media => media.id === id);
		if (!file) return itemData;
		itemData.src = file.url;
		itemData.w = Number(file.properties.width);
		itemData.h = Number(file.properties.height);
		if (file.properties.orientation != null && file.properties.orientation >= 5) {
			[itemData.w, itemData.h] = [itemData.h, itemData.w];
		}
		itemData.msrc = file.thumbnailUrl ?? undefined;
		// eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
		itemData.alt = file.comment || file.name;
		// eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
		itemData.comment = file.comment || file.name;
		itemData.thumbCropped = true;
		return itemData;
	});
	lightbox.on('uiRegister', () => {
		lightbox?.pswp?.ui?.registerElement({
			name: 'altText',
			className: 'pswp__alt-text-container',
			appendTo: 'wrapper',
			onInit: (el, pswp) => {
				const textBox = window.document.createElement('p');
				textBox.className = 'pswp__alt-text _acrylic';
				el.appendChild(textBox);
				pswp.on('change', () => {
					textBox.textContent = pswp.currSlide?.data.comment;
				});
			},
		});
	});
	lightbox.on('afterInit', () => {
		activeEl = window.document.activeElement instanceof HTMLElement ? window.document.activeElement : null;
		focusParent(activeEl, true, true);
		lightbox?.pswp?.element?.focus({
			preventScroll: true,
		});
		window.history.pushState(null, '', '#pswp');
	});
	lightbox.on('destroy', () => {
		focusParent(activeEl, true, false);
		activeEl = null;
		if (window.location.hash === '#pswp') {
			window.history.back();
		}
	});
	window.addEventListener('popstate', popstateHandler);
	lightbox.init();
});
onUnmounted(() => {
	window.removeEventListener('popstate', popstateHandler);
	lightbox?.destroy();
	lightbox = null;
	activeEl = null;
});
const previewable = (file: Misskey.entities.DriveFile): boolean => {
	if (file.type === 'image/svg+xml') return true; // svgのwebpublic/thumbnailはpngなのでtrue
	// FILE_TYPE_BROWSERSAFEに適合しないものはブラウザで表示するのに不適切
	return (file.type.startsWith('video') || file.type.startsWith('image')) && FILE_TYPE_BROWSERSAFE.includes(file.type);
};
const openGallery = () => {
	if (props.mediaList.filter(media => previewable(media)).length > 0) {
		lightbox?.loadAndOpen(0);
	}
};
__expose({
	openGallery,
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.mediaList.filter(media => !previewable(media)), (media) => {
        return (_openBlock(), _createBlock(XBanner, {
          key: media.id,
          media: media
        }, null, 8 /* PROPS */, ["media"]))
      }), 128 /* KEYED_FRAGMENT */)), (__props.mediaList.filter(media => previewable(media)).length > 0) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: _normalizeClass(_ctx.$style.container)
        }, [ _createElementVNode("div", {
            ref_key: "gallery", ref: gallery,
            class: _normalizeClass([
  				_ctx.$style.medias,
  				...(_unref(prefer).s.showMediaListByGridInWideArea ? [_ctx.$style.gridInWideArea] : []),
  				count.value === 1 ? [_ctx.$style.n1, {
  					[_ctx.$style.n116_9]: _unref(prefer).s.mediaListWithOneImageAppearance === '16_9',
  					[_ctx.$style.n11_1]: _unref(prefer).s.mediaListWithOneImageAppearance === '1_1',
  					[_ctx.$style.n12_3]: _unref(prefer).s.mediaListWithOneImageAppearance === '2_3',
  				}] : count.value === 2 ? _ctx.$style.n2 : count.value === 3 ? _ctx.$style.n3 : count.value === 4 ? _ctx.$style.n4 : _ctx.$style.nMany,
  			])
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.mediaList.filter(media => previewable(media)), (media) => {
              return (_openBlock(), _createElementBlock(_Fragment, null, [
                (media.type.startsWith('video'))
                  ? (_openBlock(), _createBlock(XVideo, {
                    key: `video:${media.id}`,
                    class: _normalizeClass(_ctx.$style.media),
                    video: media
                  }, null, 8 /* PROPS */, ["video"]))
                  : (media.type.startsWith('image'))
                    ? (_openBlock(), _createBlock(XImage, {
                      key: `image:${media.id}`,
                      class: _normalizeClass(["image", _ctx.$style.media]),
                      "data-id": media.id,
                      image: media,
                      raw: __props.raw
                    }, null, 8 /* PROPS */, ["data-id", "image", "raw"]))
                  : _createCommentVNode("v-if", true)
              ], 64 /* STABLE_FRAGMENT */))
            }), 256 /* UNKEYED_FRAGMENT */)) ], 2 /* CLASS */) ])) : _createCommentVNode("v-if", true) ]))
}
}

})
