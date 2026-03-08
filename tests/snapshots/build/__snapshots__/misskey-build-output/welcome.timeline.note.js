import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-back-up" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-back-up" })
import { ref, useTemplateRef, onUpdated, onMounted } from 'vue'
import * as Misskey from 'misskey-js'
import MkReactionsViewer from '@/components/MkReactionsViewer.vue'
import MkMediaList from '@/components/MkMediaList.vue'
import MkPoll from '@/components/MkPoll.vue'
import MkCwButton from '@/components/MkCwButton.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'welcome.timeline.note',
  props: {
    note: { type: null, required: true }
  },
  setup(__props: any) {

const noteTextEl = useTemplateRef('noteTextEl');
const shouldCollapse = ref(false);
const showContent = ref(false);
function calcCollapse() {
	if (noteTextEl.value) {
		const height = noteTextEl.value.scrollHeight;
		if (height > 200) {
			shouldCollapse.value = true;
		}
	}
}
onMounted(() => {
	calcCollapse();
});
onUpdated(() => {
	calcCollapse();
});

return (_ctx: any,_cache: any) => {
  const _component_Mfm = _resolveComponent("Mfm")
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(), _createElementBlock("div", {
      key: __props.note.id,
      class: _normalizeClass(_ctx.$style.note)
    }, [ _createElementVNode("div", {
        class: _normalizeClass(["_panel _gaps_s", _ctx.$style.content])
      }, [ (__props.note.cw != null) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.richcontent)
          }, [ _createElementVNode("div", null, [ _createVNode(_component_Mfm, {
                text: __props.note.cw,
                author: __props.note.user
              }, null, 8 /* PROPS */, ["text", "author"]) ]), _createVNode(MkCwButton, {
              text: __props.note.text,
              renote: __props.note.renote,
              files: __props.note.files,
              poll: __props.note.poll,
              style: "margin: 4px 0;",
              modelValue: showContent.value,
              "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((showContent).value = $event))
            }, null, 8 /* PROPS */, ["text", "renote", "files", "poll", "modelValue"]), (showContent.value) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ (__props.note.replyId) ? (_openBlock(), _createBlock(_component_MkA, {
                    key: 0,
                    class: "reply",
                    to: `/notes/${__props.note.replyId}`
                  }, {
                    default: _withCtx(() => [
                      _hoisted_1
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true), (__props.note.text) ? (_openBlock(), _createBlock(_component_Mfm, {
                    key: 0,
                    text: __props.note.text,
                    author: __props.note.user
                  }, null, 8 /* PROPS */, ["text", "author"])) : _createCommentVNode("v-if", true), (__props.note.renoteId) ? (_openBlock(), _createBlock(_component_MkA, {
                    key: 0,
                    class: "rp",
                    to: `/notes/${__props.note.renoteId}`
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode("RN: ...")
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true) ])) : (_openBlock(), _createElementBlock("div", {
            key: 1,
            ref: "noteTextEl",
            class: _normalizeClass([_ctx.$style.text, { [_ctx.$style.collapsed]: shouldCollapse.value }])
          }, [ (__props.note.replyId) ? (_openBlock(), _createBlock(_component_MkA, {
                key: 0,
                class: "reply",
                to: `/notes/${__props.note.replyId}`
              }, {
                default: _withCtx(() => [
                  _hoisted_2
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true), (__props.note.text) ? (_openBlock(), _createBlock(_component_Mfm, {
                key: 0,
                text: __props.note.text,
                author: __props.note.user
              }, null, 8 /* PROPS */, ["text", "author"])) : _createCommentVNode("v-if", true), (__props.note.renoteId) ? (_openBlock(), _createBlock(_component_MkA, {
                key: 0,
                class: "rp",
                to: `/notes/${__props.note.renoteId}`
              }, {
                default: _withCtx(() => [
                  _createTextVNode("RN: ...")
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true) ])), (__props.note.files && __props.note.files.length > 0 && (__props.note.cw == null || showContent.value)) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.richcontent)
          }, [ _createVNode(MkMediaList, { mediaList: __props.note.files.slice(0, 4) }, null, 8 /* PROPS */, ["mediaList"]) ])) : _createCommentVNode("v-if", true), (__props.note.reactionCount > 0) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.reactions)
          }, [ _createVNode(MkReactionsViewer, {
              noteId: __props.note.id,
              reactions: __props.note.reactions,
              reactionEmojis: __props.note.reactionEmojis,
              myReaction: __props.note.myReaction,
              maxNumber: 16
            }, null, 8 /* PROPS */, ["noteId", "reactions", "reactionEmojis", "myReaction", "maxNumber"]) ])) : _createCommentVNode("v-if", true) ]) ]))
}
}

})
