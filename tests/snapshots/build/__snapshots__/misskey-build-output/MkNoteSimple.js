import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref, vShow as _vShow } from "vue"

import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import MkNoteHeader from '@/components/MkNoteHeader.vue'
import MkSubNoteContent from '@/components/MkSubNoteContent.vue'
import MkCwButton from '@/components/MkCwButton.vue'
import { i18n } from '@/i18n.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkNoteSimple',
  props: {
    note: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const showContent = ref(false);

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_Mfm = _resolveComponent("Mfm")

  return (__props.note)
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        class: _normalizeClass(_ctx.$style.root)
      }, [ _createVNode(_component_MkAvatar, {
          class: _normalizeClass([_ctx.$style.avatar, _unref(prefer).s.useStickyIcons ? _ctx.$style.useSticky : null]),
          user: __props.note.user,
          link: "",
          preview: ""
        }, null, 10 /* CLASS, PROPS */, ["user"]), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.main)
        }, [ _createVNode(MkNoteHeader, {
            class: _normalizeClass(_ctx.$style.header),
            note: __props.note,
            mini: true
          }, null, 8 /* PROPS */, ["note", "mini"]), _createElementVNode("div", null, [ (__props.note.cw != null) ? (_openBlock(), _createElementBlock("p", {
                key: 0,
                class: _normalizeClass(_ctx.$style.cw)
              }, [ (__props.note.cw != '') ? (_openBlock(), _createBlock(_component_Mfm, {
                    key: 0,
                    style: "margin-right: 8px;",
                    text: __props.note.cw,
                    author: __props.note.user,
                    nyaize: 'respect',
                    emojiUrls: __props.note.emojis
                  }, null, 8 /* PROPS */, ["text", "author", "nyaize", "emojiUrls"])) : _createCommentVNode("v-if", true), _createVNode(MkCwButton, {
                  text: __props.note.text,
                  files: __props.note.files,
                  poll: __props.note.poll,
                  modelValue: showContent.value,
                  "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((showContent).value = $event))
                }, null, 8 /* PROPS */, ["text", "files", "poll", "modelValue"]) ])) : _createCommentVNode("v-if", true), _withDirectives(_createElementVNode("div", null, [ _createVNode(MkSubNoteContent, {
                class: _normalizeClass(_ctx.$style.text),
                note: __props.note
              }, null, 8 /* PROPS */, ["note"]) ], 512 /* NEED_PATCH */), [ [_vShow, __props.note.cw == null || showContent.value] ]) ]) ]) ]))
      : (_openBlock(), _createElementBlock("div", {
        key: 1,
        class: _normalizeClass(_ctx.$style.deleted)
      }, _toDisplayString(_unref(i18n).ts.deletedNote), 1 /* TEXT */))
}
}

})
