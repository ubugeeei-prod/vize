import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, withCtx as _withCtx, unref as _unref } from "vue"

import { computed, markRaw } from 'vue'
import MkUserList from '@/components/MkUserList.vue'
import { definePage } from '@/page.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'user-tag',
  props: {
    tag: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const paginator = markRaw(new Paginator('hashtags/users', {
	limit: 30,
	offsetMode: true,
	computedParams: computed(() => ({
		tag: props.tag,
		origin: 'combined',
		sort: '+follower',
	})),
}));
definePage(() => ({
	title: props.tag,
	icon: 'ti ti-user-search',
}));

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, null, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 1200px;"
        }, [
          _createElementVNode("div", { class: "_gaps_s" }, [
            _createVNode(MkUserList, { paginator: _unref(paginator) }, null, 8 /* PROPS */, ["paginator"])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
