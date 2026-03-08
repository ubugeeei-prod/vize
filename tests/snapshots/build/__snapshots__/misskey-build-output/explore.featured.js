import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, unref as _unref } from "vue"

import { markRaw, ref } from 'vue'
import MkNotesTimeline from '@/components/MkNotesTimeline.vue'
import MkTab from '@/components/MkTab.vue'
import { i18n } from '@/i18n.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'explore.featured',
  setup(__props) {

const paginatorForNotes = markRaw(new Paginator('notes/featured', {
	limit: 10,
}));
const paginatorForPolls = markRaw(new Paginator('notes/polls/recommendation', {
	limit: 10,
	offsetMode: true,
	params: {
		excludeChannels: true,
	},
}));
const tab = ref<'notes' | 'polls'>('notes');

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: "_spacer",
      style: "--MI_SPACER-w: 800px;"
    }, [ _createVNode(MkTab, {
        tabs: [
  			{ key: 'notes', label: _unref(i18n).ts.notes },
  			{ key: 'polls', label: _unref(i18n).ts.poll },
  		],
        style: "margin-bottom: var(--MI-margin);",
        modelValue: tab.value,
        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
      }, null, 8 /* PROPS */, ["tabs", "modelValue"]), (tab.value === 'notes') ? (_openBlock(), _createBlock(MkNotesTimeline, {
          key: 0,
          paginator: _unref(paginatorForNotes)
        }, null, 8 /* PROPS */, ["paginator"])) : (tab.value === 'polls') ? (_openBlock(), _createBlock(MkNotesTimeline, {
            key: 1,
            paginator: _unref(paginatorForPolls)
          }, null, 8 /* PROPS */, ["paginator"])) : _createCommentVNode("v-if", true) ]))
}
}

})
