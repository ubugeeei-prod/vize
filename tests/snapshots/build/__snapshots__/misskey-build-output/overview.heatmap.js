import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, normalizeClass as _normalizeClass, unref as _unref } from "vue"

import MkHeatmap from '@/components/MkHeatmap.vue'
import MkSelect from '@/components/MkSelect.vue'
import { useMkSelect } from '@/composables/use-mkselect.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'overview.heatmap',
  setup(__props) {

const {
	model: src,
	def: srcDef,
} = useMkSelect({
	items: [
		{ label: 'Active users', value: 'active-users' },
		{ label: 'Notes', value: 'notes' },
		{ label: 'AP Requests: inboxReceived', value: 'ap-requests-inbox-received' },
		{ label: 'AP Requests: deliverSucceeded', value: 'ap-requests-deliver-succeeded' },
		{ label: 'AP Requests: deliverFailed', value: 'ap-requests-deliver-failed' },
	],
	initialValue: 'active-users',
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["_panel", _ctx.$style.root])
    }, [ _createVNode(MkSelect, {
        items: _unref(srcDef),
        style: "margin: 0 0 12px 0;",
        small: "",
        modelValue: _unref(src),
        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((src).value = $event))
      }, null, 8 /* PROPS */, ["items", "modelValue"]), _createVNode(MkHeatmap, { src: _unref(src) }, null, 8 /* PROPS */, ["src"]) ]))
}
}

})
