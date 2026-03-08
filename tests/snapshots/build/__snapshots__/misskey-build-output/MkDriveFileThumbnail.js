import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, resolveDirective as _resolveDirective, withDirectives as _withDirectives, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, unref as _unref } from "vue"

import { computed } from 'vue'
import * as Misskey from 'misskey-js'
import MkImgWithBlurhash from '@/components/MkImgWithBlurhash.vue'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkDriveFileThumbnail',
  props: {
    file: { type: null, required: true },
    fit: { type: String, required: true },
    highlightWhenSensitive: { type: Boolean, required: false },
    forceBlurhash: { type: Boolean, required: false },
    large: { type: Boolean, required: false }
  },
  setup(__props: any) {

const props = __props
const is = computed(() => {
	if (props.file.type.startsWith('image/')) return 'image';
	if (props.file.type.startsWith('video/')) return 'video';
	if (props.file.type === 'audio/midi') return 'midi';
	if (props.file.type.startsWith('audio/')) return 'audio';
	if (props.file.type.endsWith('/csv')) return 'csv';
	if (props.file.type.endsWith('/pdf')) return 'pdf';
	if (props.file.type.startsWith('text/')) return 'textfile';
	if ([
		'application/zip',
		'application/x-cpio',
		'application/x-bzip',
		'application/x-bzip2',
		'application/java-archive',
		'application/x-rar-compressed',
		'application/x-tar',
		'application/gzip',
		'application/x-7z-compressed',
	].some(archiveType => archiveType === props.file.type)) return 'archive';
	return 'unknown';
});
const isThumbnailAvailable = computed(() => {
	return props.file.thumbnailUrl
		? (is.value === 'image' || is.value === 'video')
		: false;
});

return (_ctx: any,_cache: any) => {
  const _directive_panel = _resolveDirective("panel")

  return _withDirectives((_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.root, {
  		[_ctx.$style.sensitiveHighlight]: __props.highlightWhenSensitive && __props.file.isSensitive,
  		[_ctx.$style.large]: __props.large,
  	}])
    }, [ (isThumbnailAvailable.value && _unref(prefer).s.enableHighQualityImagePlaceholders) ? (_openBlock(), _createBlock(MkImgWithBlurhash, {
          key: 0,
          hash: __props.file.blurhash,
          src: __props.file.thumbnailUrl,
          alt: __props.file.name,
          title: __props.file.name,
          class: _normalizeClass(_ctx.$style.thumbnail),
          cover: __props.fit !== 'contain',
          forceBlurhash: __props.forceBlurhash
        }, null, 8 /* PROPS */, ["hash", "src", "alt", "title", "cover", "forceBlurhash"])) : (isThumbnailAvailable.value && __props.file.thumbnailUrl != null) ? (_openBlock(), _createElementBlock("img", {
            key: 1,
            src: __props.file.thumbnailUrl,
            alt: __props.file.name,
            title: __props.file.name,
            class: _normalizeClass(_ctx.$style.thumbnail),
            style: _normalizeStyle({ objectFit: __props.fit })
          })) : (is.value === 'image') ? (_openBlock(), _createElementBlock("i", {
            key: 2,
            class: _normalizeClass(["ti ti-photo", _ctx.$style.icon])
          })) : (is.value === 'video') ? (_openBlock(), _createElementBlock("i", {
            key: 3,
            class: _normalizeClass(["ti ti-video", _ctx.$style.icon])
          })) : (is.value === 'audio' || is.value === 'midi') ? (_openBlock(), _createElementBlock("i", {
            key: 4,
            class: _normalizeClass(["ti ti-file-music", _ctx.$style.icon])
          })) : (is.value === 'csv') ? (_openBlock(), _createElementBlock("i", {
            key: 5,
            class: _normalizeClass(["ti ti-file-text", _ctx.$style.icon])
          })) : (is.value === 'pdf') ? (_openBlock(), _createElementBlock("i", {
            key: 6,
            class: _normalizeClass(["ti ti-file-text", _ctx.$style.icon])
          })) : (is.value === 'textfile') ? (_openBlock(), _createElementBlock("i", {
            key: 7,
            class: _normalizeClass(["ti ti-file-text", _ctx.$style.icon])
          })) : (is.value === 'archive') ? (_openBlock(), _createElementBlock("i", {
            key: 8,
            class: _normalizeClass(["ti ti-file-zip", _ctx.$style.icon])
          })) : (_openBlock(), _createElementBlock("i", {
          key: 9,
          class: _normalizeClass(["ti ti-file", _ctx.$style.icon])
        })), (isThumbnailAvailable.value && is.value === 'video') ? (_openBlock(), _createElementBlock("i", {
          key: 0,
          class: _normalizeClass(["ti ti-video", _ctx.$style.iconSub])
        })) : _createCommentVNode("v-if", true) ], 2 /* CLASS */)), [ [_directive_panel] ])
}
}

})
