import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, renderList as _renderList, normalizeClass as _normalizeClass } from "vue"

import * as Misskey from 'misskey-js'
import { onUpdated, ref, useTemplateRef } from 'vue'
import { getScrollContainer } from '@@/js/scroll.js'
import XNote from '@/pages/welcome.timeline.note.vue'
import { misskeyApiGet } from '@/utility/misskey-api.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'welcome.timeline',
  setup(__props) {

const notes = ref<Misskey.entities.Note[]>([]);
const isScrolling = ref(false);
const scrollState = ref<null | 'intro' | 'loop'>(null);
const notesMainContainerEl = useTemplateRef('notesMainContainerEl');
misskeyApiGet('notes/featured').then(_notes => {
	notes.value = _notes;
});
function changeScrollState() {
	if (scrollState.value !== 'loop') {
		scrollState.value = 'loop';
	}
}
onUpdated(() => {
	if (!notesMainContainerEl.value) return;
	const container = getScrollContainer(notesMainContainerEl.value);
	const containerHeight = container ? container.clientHeight : window.innerHeight;
	if (notesMainContainerEl.value.offsetHeight > containerHeight) {
		if (scrollState.value === null) {
			scrollState.value = 'intro';
		}
		isScrolling.value = true;
	}
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["_gaps", _ctx.$style.root])
    }, [ _createElementVNode("div", {
        ref_key: "notesMainContainerEl", ref: notesMainContainerEl,
        class: _normalizeClass(["_gaps", [_ctx.$style.scrollBoxMain, { [_ctx.$style.scrollIntro]: (scrollState.value === 'intro'), [_ctx.$style.scrollLoop]: (scrollState.value === 'loop') }]]),
        onAnimationend: changeScrollState
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(notes.value, (note) => {
          return (_openBlock(), _createBlock(XNote, {
            key: `${note.id}_1`,
            class: _normalizeClass(_ctx.$style.note),
            note: note
          }, null, 8 /* PROPS */, ["note"]))
        }), 128 /* KEYED_FRAGMENT */)) ], 34 /* CLASS, NEED_HYDRATION */), (isScrolling.value) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: _normalizeClass(["_gaps", [_ctx.$style.scrollBoxSub, { [_ctx.$style.scrollIntro]: (scrollState.value === 'intro'), [_ctx.$style.scrollLoop]: (scrollState.value === 'loop') }]])
        }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(notes.value, (note) => {
            return (_openBlock(), _createBlock(XNote, {
              key: `${note.id}_2`,
              class: _normalizeClass(_ctx.$style.note),
              note: note
            }, null, 8 /* PROPS */, ["note"]))
          }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true) ]))
}
}

})
