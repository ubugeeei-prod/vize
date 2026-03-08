import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, resolveComponent as _resolveComponent, unref as _unref } from "vue"

import { computed } from 'vue'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { pleaseLogin } from '@/utility/please-login.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'not-found',
  props: {
    showLoginPopup: { type: Boolean, required: false }
  },
  setup(__props: any) {

const props = __props
if (props.showLoginPopup) {
	pleaseLogin({ path: '/' });
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.notFound,
	icon: 'ti ti-alert-triangle',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkResult = _resolveComponent("MkResult")

  return (_openBlock(), _createElementBlock("div", { style: "align-content: center; height: 100cqh;" }, [ _createVNode(_component_MkResult, {
        type: "notFound",
        text: _unref(i18n).ts.notFoundDescription
      }, null, 8 /* PROPS */, ["text"]) ]))
}
}

})
