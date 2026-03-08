import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { ref, computed, markRaw } from 'vue'
import * as Misskey from 'misskey-js'
import MkNotesTimeline from '@/components/MkNotesTimeline.vue'
import MkTab from '@/components/MkTab.vue'
import { i18n } from '@/i18n.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'notes',
  props: {
    user: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const tab = ref<'featured' | 'notes' | 'all' | 'files'>('all');
const featuredPaginator = markRaw(new Paginator('users/featured-notes', {
	limit: 10,
	params: {
		userId: props.user.id,
	},
}));
const notesPaginator = markRaw(new Paginator('users/notes', {
	limit: 10,
	computedParams: computed(() => ({
		userId: props.user.id,
		withRenotes: tab.value === 'all',
		withReplies: tab.value === 'all',
		withChannelNotes: tab.value === 'all',
		withFiles: tab.value === 'files',
	})),
}));

return (_ctx: any,_cache: any) => {
  const _component_MkStickyContainer = _resolveComponent("MkStickyContainer")

  return (_openBlock(), _createElementBlock("div", {
      class: "_spacer",
      style: "--MI_SPACER-w: 800px;"
    }, [ _createElementVNode("div", null, [ _createVNode(_component_MkStickyContainer, null, {
          header: _withCtx(() => [
            _createVNode(MkTab, {
              tabs: [
  						{ key: 'featured', label: _unref(i18n).ts.featured },
  						{ key: 'notes', label: _unref(i18n).ts.notes },
  						{ key: 'all', label: _unref(i18n).ts.all },
  						{ key: 'files', label: _unref(i18n).ts.withFiles },
  					],
              class: _normalizeClass(_ctx.$style.tab),
              modelValue: tab.value,
              "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
            }, null, 8 /* PROPS */, ["tabs", "modelValue"])
          ]),
          default: _withCtx(() => [
            (tab.value === 'featured')
              ? (_openBlock(), _createBlock(MkNotesTimeline, {
                key: 0,
                noGap: true,
                paginator: _unref(featuredPaginator),
                class: _normalizeClass(_ctx.$style.tl)
              }, null, 8 /* PROPS */, ["noGap", "paginator"]))
              : (_openBlock(), _createBlock(MkNotesTimeline, {
                key: 1,
                noGap: true,
                paginator: _unref(notesPaginator),
                class: _normalizeClass(_ctx.$style.tl)
              }, null, 8 /* PROPS */, ["noGap", "paginator"]))
          ]),
          _: 1 /* STABLE */
        }) ]) ]))
}
}

})
