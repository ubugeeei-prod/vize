import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode } from "vue"

import { computed } from 'vue'
import MkSample from '@/components/MkPreview.vue'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'preview',
  setup(__props) {

const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(computed(() => ({
	title: i18n.ts.preview,
	icon: 'ti ti-eye',
})));

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", null, [ _createVNode(MkSample) ]))
}
}

})
