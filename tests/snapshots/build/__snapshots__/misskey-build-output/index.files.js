import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-photo" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
import { onMounted, ref } from 'vue'
import * as Misskey from 'misskey-js'
import { misskeyApi } from '@/utility/misskey-api.js'
import MkButton from '@/components/MkButton.vue'
import MkContainer from '@/components/MkContainer.vue'
import { i18n } from '@/i18n.js'
import MkNoteMediaGrid from '@/components/MkNoteMediaGrid.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'index.files',
  props: {
    user: { type: null, required: true }
  },
  emits: ["showMore"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const fetching = ref(true);
const notes = ref<Misskey.entities.Note[]>([]);
onMounted(() => {
	misskeyApi('users/notes', {
		userId: props.user.id,
		withFiles: true,
		limit: 10,
	}).then(_notes => {
		notes.value = _notes;
		fetching.value = false;
	});
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createBlock(MkContainer, null, {
      icon: _withCtx(() => [
        _hoisted_1
      ]),
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(i18n).ts.files), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.root)
        }, [
          (fetching.value)
            ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 }))
            : _createCommentVNode("v-if", true),
          (!fetching.value && notes.value.length > 0)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "_gaps_s"
            }, [
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.stream)
              }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(notes.value, (note) => {
                  return (_openBlock(), _createBlock(MkNoteMediaGrid, { note: note }, null, 8 /* PROPS */, ["note"]))
                }), 256 /* UNKEYED_FRAGMENT */))
              ]),
              _createVNode(MkButton, {
                rounded: "",
                full: "",
                onClick: _cache[0] || (_cache[0] = ($event: any) => (emit('showMore')))
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.showMore), 1 /* TEXT */),
                  _createTextVNode(" "),
                  _hoisted_2
                ]),
                _: 1 /* STABLE */
              })
            ]))
            : _createCommentVNode("v-if", true),
          (!fetching.value && notes.value.length == 0)
            ? (_openBlock(), _createElementBlock("p", { key: 0 }, _toDisplayString(_unref(i18n).ts.nothing), 1 /* TEXT */))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
