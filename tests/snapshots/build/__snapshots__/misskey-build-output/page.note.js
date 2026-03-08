import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, normalizeClass as _normalizeClass } from "vue"

import { onMounted, ref } from 'vue'
import * as Misskey from 'misskey-js'
import MkNote from '@/components/MkNote.vue'
import MkNoteDetailed from '@/components/MkNoteDetailed.vue'
import { misskeyApi } from '@/utility/misskey-api.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'page.note',
  props: {
    block: { type: null, required: true },
    page: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const note = ref<Misskey.entities.Note | null>(null);
onMounted(() => {
	if (props.block.note == null) return;
	misskeyApi('notes/show', { noteId: props.block.note })
		.then(result => {
			note.value = result;
		});
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ (note.value && !__props.block.detailed) ? (_openBlock(), _createBlock(MkNote, {
          key: note.value.id + ':normal',
          note: note.value
        }, null, 8 /* PROPS */, ["note"])) : _createCommentVNode("v-if", true), (note.value && __props.block.detailed) ? (_openBlock(), _createBlock(MkNoteDetailed, {
          key: note.value.id + ':detail',
          note: note.value
        }, null, 8 /* PROPS */, ["note"])) : _createCommentVNode("v-if", true) ]))
}
}

})
