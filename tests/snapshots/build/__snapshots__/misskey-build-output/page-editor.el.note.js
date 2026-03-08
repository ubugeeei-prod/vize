import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-note" })
import { watch, ref } from 'vue'
import * as Misskey from 'misskey-js'
import XContainer from '../page-editor.container.vue'
import MkInput from '@/components/MkInput.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkNote from '@/components/MkNote.vue'
import MkNoteDetailed from '@/components/MkNoteDetailed.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'page-editor.el.note',
  props: {
    modelValue: { type: null, required: true }
  },
  emits: ["update:modelValue", "note", "remove"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
/* eslint-disable vue/no-mutating-props */
const id = ref(props.modelValue.note);
const note = ref<Misskey.entities.Note | null>(null);
watch(id, async () => {
	if (id.value && (id.value.startsWith('http://') || id.value.startsWith('https://'))) {
		id.value = (id.value.endsWith('/') ? id.value.slice(0, -1) : id.value).split('/').pop() ?? null;
	}
	if (!id.value) {
		note.value = null;
		return;
	}
	emit('update:modelValue', {
		...props.modelValue,
		note: id.value,
	});
	note.value = await misskeyApi('notes/show', { noteId: id.value });
}, {
	immediate: true,
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(XContainer, {
      draggable: true,
      onRemove: _cache[0] || (_cache[0] = () => emit('remove'))
    }, {
      header: _withCtx(() => [
        _hoisted_1,
        _createTextVNode(" "),
        _createTextVNode(_toDisplayString(_unref(i18n).ts._pages.blocks.note), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createElementVNode("section", {
          style: "padding: 16px;",
          class: "_gaps_s"
        }, [
          _createVNode(MkInput, {
            modelValue: id.value,
            "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((id).value = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._pages.blocks._note.id), 1 /* TEXT */)
            ]),
            caption: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._pages.blocks._note.idDescription), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]),
          _createVNode(MkSwitch, {
            modelValue: props.modelValue.detailed,
            "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((props.modelValue.detailed) = $event))
          }, {
            default: _withCtx(() => [
              _createElementVNode("span", null, _toDisplayString(_unref(i18n).ts._pages.blocks._note.detailed), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]),
          (note.value && !props.modelValue.detailed)
            ? (_openBlock(), _createBlock(MkNote, {
              key: note.value.id + ':normal',
              style: "margin-bottom: 16px;",
              note: note.value,
              "onUpdate:note": _cache[3] || (_cache[3] = ($event: any) => ((note).value = $event))
            }, null, 8 /* PROPS */, ["note"]))
            : _createCommentVNode("v-if", true),
          (note.value && props.modelValue.detailed)
            ? (_openBlock(), _createBlock(MkNoteDetailed, {
              key: note.value.id + ':detail',
              style: "margin-bottom: 16px;",
              note: note.value,
              "onUpdate:note": _cache[4] || (_cache[4] = ($event: any) => ((note).value = $event))
            }, null, 8 /* PROPS */, ["note"]))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["draggable"]))
}
}

})
