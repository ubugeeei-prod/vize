import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-back-up" })
import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import { shouldCollapsed } from '@@/js/collapsed.js'
import MkMediaList from '@/components/MkMediaList.vue'
import MkPoll from '@/components/MkPoll.vue'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSubNoteContent',
  props: {
    note: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const isLong = shouldCollapsed(props.note, []);
const collapsed = ref(isLong);

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")
  const _component_Mfm = _resolveComponent("Mfm")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.collapsed]: collapsed.value }])
    }, [ _createElementVNode("div", null, [ (__props.note.isHidden) ? (_openBlock(), _createElementBlock("span", {
            key: 0,
            style: "opacity: 0.5"
          }, "(" + _toDisplayString(_unref(i18n).ts.private) + ")", 1 /* TEXT */)) : _createCommentVNode("v-if", true), (__props.note.deletedAt) ? (_openBlock(), _createElementBlock("span", {
            key: 0,
            style: "opacity: 0.5"
          }, "(" + _toDisplayString(_unref(i18n).ts.deletedNote) + ")", 1 /* TEXT */)) : _createCommentVNode("v-if", true), (__props.note.replyId) ? (_openBlock(), _createBlock(_component_MkA, {
            key: 0,
            class: _normalizeClass(_ctx.$style.reply),
            to: `/notes/${__props.note.replyId}`
          }, {
            default: _withCtx(() => [
              _hoisted_1
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true), (__props.note.text) ? (_openBlock(), _createBlock(_component_Mfm, {
            key: 0,
            text: __props.note.text,
            author: __props.note.user,
            nyaize: 'respect',
            emojiUrls: __props.note.emojis
          }, null, 8 /* PROPS */, ["text", "author", "nyaize", "emojiUrls"])) : _createCommentVNode("v-if", true), (__props.note.renoteId) ? (_openBlock(), _createBlock(_component_MkA, {
            key: 0,
            class: _normalizeClass(_ctx.$style.rp),
            to: `/notes/${__props.note.renoteId}`
          }, {
            default: _withCtx(() => [
              _createTextVNode("RN: ...")
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true) ]), (__props.note.files && __props.note.files.length > 0) ? (_openBlock(), _createElementBlock("details", { key: 0 }, [ _createElementVNode("summary", null, "(" + _toDisplayString(_unref(i18n).tsx.withNFiles({ n: __props.note.files.length })) + ")", 1 /* TEXT */), _createVNode(MkMediaList, { mediaList: __props.note.files }, null, 8 /* PROPS */, ["mediaList"]) ])) : _createCommentVNode("v-if", true), (__props.note.poll) ? (_openBlock(), _createElementBlock("details", { key: 0 }, [ _createElementVNode("summary", null, _toDisplayString(_unref(i18n).ts.poll), 1 /* TEXT */), _createVNode(MkPoll, {
            noteId: __props.note.id,
            multiple: __props.note.poll.multiple,
            expiresAt: __props.note.poll.expiresAt,
            choices: __props.note.poll.choices,
            author: __props.note.user,
            emojiUrls: __props.note.emojis
          }, null, 8 /* PROPS */, ["noteId", "multiple", "expiresAt", "choices", "author", "emojiUrls"]) ])) : _createCommentVNode("v-if", true), (__props.note.hasPoll && __props.note.poll == null) ? (_openBlock(), _createBlock(_component_MkA, {
          key: 0,
          to: `/notes/${__props.note.id}`
        }, {
          default: _withCtx(() => [
            _createTextVNode("("),
            _createTextVNode(_toDisplayString(_unref(i18n).ts.poll), 1 /* TEXT */),
            _createTextVNode(")")
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true), (_unref(isLong) && collapsed.value) ? (_openBlock(), _createElementBlock("button", {
          key: 0,
          class: _normalizeClass(["_button", _ctx.$style.fade]),
          onClick: _cache[0] || (_cache[0] = ($event: any) => (collapsed.value = false))
        }, [ _createElementVNode("span", {
            class: _normalizeClass(_ctx.$style.fadeLabel)
          }, _toDisplayString(_unref(i18n).ts.showMore), 1 /* TEXT */) ])) : (_unref(isLong) && !collapsed.value) ? (_openBlock(), _createElementBlock("button", {
            key: 1,
            class: _normalizeClass(["_button", _ctx.$style.showLess]),
            onClick: _cache[1] || (_cache[1] = ($event: any) => (collapsed.value = true))
          }, [ _createElementVNode("span", {
              class: _normalizeClass(_ctx.$style.showLessLabel)
            }, _toDisplayString(_unref(i18n).ts.showLess), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true) ], 2 /* CLASS */))
}
}

})
