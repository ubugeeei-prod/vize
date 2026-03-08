import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye-exclamation" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-photo" })
import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import { notePage } from '@/filters/note.js'
import { i18n } from '@/i18n.js'
import { prefer } from '@/preferences.js'
import { shouldHideFileByDefault, canRevealFile } from '@/utility/sensitive-file.js'
import bytes from '@/filters/bytes.js'
import MkDriveFileThumbnail from '@/components/MkDriveFileThumbnail.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkNoteMediaGrid',
  props: {
    note: { type: null, required: true },
    square: { type: Boolean, required: false }
  },
  setup(__props: any) {

const showingFiles = ref<Set<string>>(new Set());
function isHiding(file: Misskey.entities.DriveFile) {
	if (shouldHideFileByDefault(file) && !showingFiles.value.has(file.id)) {
		if (!file.isSensitive && !file.type.startsWith('image/')) {
			return false;
		}
		return true;
	}
	return false;
}
async function reveal(file: Misskey.entities.DriveFile) {
	if (!(await canRevealFile(file))) {
		return;
	}
	showingFiles.value.add(file.id);
}

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.note.files, (file) => {
      return (_openBlock(), _createElementBlock(_Fragment, null, [
        (isHiding(file))
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass([_ctx.$style.filePreview, { [_ctx.$style.square]: __props.square }]),
            onClick: _cache[0] || (_cache[0] = ($event: any) => (reveal(file)))
          }, [
            _createVNode(MkDriveFileThumbnail, {
              file: file,
              fit: "cover",
              highlightWhenSensitive: _unref(prefer).s.highlightSensitiveMedia,
              forceBlurhash: true,
              large: true,
              class: _normalizeClass(_ctx.$style.file)
            }, null, 8 /* PROPS */, ["file", "highlightWhenSensitive", "forceBlurhash", "large"]),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.sensitive)
            }, [
              _createElementVNode("div", null, [
                (file.isSensitive)
                  ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                    _hoisted_1,
                    _createTextVNode(),
                    _toDisplayString(_unref(i18n).ts.sensitive),
                    _toDisplayString(_unref(prefer).s.dataSaver.media && file.size ? ` (${bytes(file.size)})` : '')
                  ]))
                  : (_openBlock(), _createElementBlock("div", { key: 1 }, [
                    _hoisted_2,
                    _createTextVNode(),
                    _toDisplayString(_unref(prefer).s.dataSaver.media && file.size ? bytes(file.size) : _unref(i18n).ts.image)
                  ])),
                _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts.clickToShow), 1 /* TEXT */)
              ])
            ])
          ]))
          : (_openBlock(), _createBlock(_component_MkA, {
            key: 1,
            class: _normalizeClass([_ctx.$style.filePreview, { [_ctx.$style.square]: __props.square }]),
            to: _unref(notePage)(__props.note)
          }, {
            default: _withCtx(() => [
              _createVNode(MkDriveFileThumbnail, {
                file: file,
                fit: "cover",
                highlightWhenSensitive: _unref(prefer).s.highlightSensitiveMedia,
                large: true,
                class: _normalizeClass(_ctx.$style.file)
              }, null, 8 /* PROPS */, ["file", "highlightWhenSensitive", "large"])
            ]),
            _: 2 /* DYNAMIC */
          }, 10 /* CLASS, PROPS */, ["to"]))
      ], 64 /* STABLE_FRAGMENT */))
    }), 256 /* UNKEYED_FRAGMENT */))
}
}

})
