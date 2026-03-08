import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, createSlots as _createSlots, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil" })
import { computed, markRaw, ref } from 'vue'
import type { PageHeaderItem } from '@/types/page-header.js'
import MkNotesTimeline from '@/components/MkNotesTimeline.vue'
import MkButton from '@/components/MkButton.vue'
import { definePage } from '@/page.js'
import { i18n } from '@/i18n.js'
import { $i } from '@/i.js'
import { store } from '@/store.js'
import * as os from '@/os.js'
import { genEmbedCode } from '@/utility/get-embed-code.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'tag',
  props: {
    tag: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const paginator = markRaw(new Paginator('notes/search-by-tag', {
	limit: 10,
	computedParams: computed(() => ({
		tag: props.tag,
	})),
}));
async function post() {
	store.set('postFormHashtags', props.tag);
	store.set('postFormWithHashtags', true);
	await os.post();
	store.set('postFormHashtags', '');
	store.set('postFormWithHashtags', false);
	paginator.reload();
}
const headerActions = computed<PageHeaderItem[]>(() => [{
	icon: 'ti ti-dots',
	text: i18n.ts.more,
	handler: (ev) => {
		os.popupMenu([{
			text: i18n.ts.embed,
			icon: 'ti ti-code',
			action: () => {
				genEmbedCode('tags', props.tag);
			},
		}], ev.currentTarget ?? ev.target);
	},
}]);
const headerTabs = computed(() => []);
definePage(() => ({
	title: props.tag,
	icon: 'ti ti-hash',
}));

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, _createSlots({ _: 2 /* DYNAMIC */ }, [ (_unref($i)) ? {
          name: "footer",
          fn: _withCtx(() => [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.footer)
            }, [
              _createElementVNode("div", {
                class: "_spacer",
                style: "--MI_SPACER-w: 800px; --MI_SPACER-min: 16px; --MI_SPACER-max: 16px;"
              }, [
                _createVNode(MkButton, {
                  rounded: "",
                  primary: "",
                  class: _normalizeClass(_ctx.$style.button),
                  onClick: _cache[0] || (_cache[0] = ($event: any) => (post()))
                }, {
                  default: _withCtx(() => [
                    _hoisted_1,
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.postToHashtag), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                })
              ])
            ])
          ]),
          key: "0"
        } : undefined ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["actions", "tabs"]))
}
}

})
