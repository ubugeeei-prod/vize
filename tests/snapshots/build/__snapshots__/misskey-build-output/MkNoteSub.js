import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-double-right" })
import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import MkNoteHeader from '@/components/MkNoteHeader.vue'
import MkSubNoteContent from '@/components/MkSubNoteContent.vue'
import MkCwButton from '@/components/MkCwButton.vue'
import { notePage } from '@/filters/note.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { $i } from '@/i.js'
import { userPage } from '@/filters/user.js'
import { checkWordMute } from '@/utility/check-word-mute.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkNoteSub',
  props: {
    note: { type: null, required: true },
    detail: { type: Boolean, required: false },
    depth: { type: Number, required: false, default: 1 }
  },
  setup(__props: any) {

const props = __props
const muted = ref(props.note && $i ? checkWordMute(props.note, $i, $i.mutedWords) : false);
const showContent = ref(false);
const replies = ref<Misskey.entities.Note[]>([]);
if (props.detail && props.note) {
	misskeyApi('notes/children', {
		noteId: props.note.id,
		limit: 5,
	}).then(res => {
		replies.value = res;
	});
}

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_Mfm = _resolveComponent("Mfm")
  const _component_MkNoteSub = _resolveComponent("MkNoteSub")
  const _component_MkA = _resolveComponent("MkA")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_I18n = _resolveComponent("I18n")
  const _directive_user_preview = _resolveDirective("user-preview")

  return (__props.note == null)
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        class: _normalizeClass(_ctx.$style.deleted)
      }, _toDisplayString(_unref(i18n).ts.deletedNote), 1 /* TEXT */))
      : (!muted.value)
        ? (_openBlock(), _createElementBlock("div", {
          key: 1,
          class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.children]: __props.depth > 1 }])
        }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.main)
          }, [ (__props.note.channel) ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: _normalizeClass(_ctx.$style.colorBar),
                style: _normalizeStyle({ background: __props.note.channel.color })
              })) : _createCommentVNode("v-if", true), _createVNode(_component_MkAvatar, {
              class: _normalizeClass(_ctx.$style.avatar),
              user: __props.note.user,
              link: "",
              preview: ""
            }, null, 8 /* PROPS */, ["user"]), _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.body)
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
                        nyaize: 'respect'
                      }, null, 8 /* PROPS */, ["text", "author", "nyaize"])) : _createCommentVNode("v-if", true), _createVNode(MkCwButton, {
                      text: __props.note.text,
                      files: __props.note.files,
                      poll: __props.note.poll,
                      modelValue: showContent.value,
                      "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((showContent).value = $event))
                    }, null, 8 /* PROPS */, ["text", "files", "poll", "modelValue"]) ])) : _createCommentVNode("v-if", true), _withDirectives(_createElementVNode("div", null, [ _createVNode(MkSubNoteContent, {
                    class: _normalizeClass(_ctx.$style.text),
                    note: __props.note
                  }, null, 8 /* PROPS */, ["note"]) ], 512 /* NEED_PATCH */), [ [_vShow, __props.note.cw == null || showContent.value] ]) ]) ]) ]), (__props.depth < 5) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(replies.value, (reply) => {
                return (_openBlock(), _createBlock(_component_MkNoteSub, {
                  key: reply.id,
                  note: reply,
                  class: _normalizeClass(_ctx.$style.reply),
                  detail: true,
                  depth: __props.depth + 1
                }, null, 8 /* PROPS */, ["note", "detail", "depth"]))
              }), 128 /* KEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock("div", {
              key: 1,
              class: _normalizeClass(_ctx.$style.more)
            }, [ _createVNode(_component_MkA, {
                class: "_link",
                to: _unref(notePage)(__props.note)
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.continueThread), 1 /* TEXT */),
                  _createTextVNode(" "),
                  _hoisted_1
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to"]) ])) ]))
      : (_openBlock(), _createElementBlock("div", {
        key: 2,
        class: _normalizeClass(_ctx.$style.muted),
        onClick: _cache[1] || (_cache[1] = ($event: any) => (muted.value = false))
      }, [ _createVNode(_component_I18n, {
          src: _unref(i18n).ts.userSaysSomething,
          tag: "small"
        }, {
          name: _withCtx(() => [
            _createVNode(_component_MkA, { to: _unref(userPage)(__props.note.user) }, {
              default: _withCtx(() => [
                _createVNode(_component_MkUserName, { user: __props.note.user }, null, 8 /* PROPS */, ["user"])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to"])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["src"]) ]))
}
}

})
