import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-section" })
import { onMounted, onBeforeUnmount, ref } from 'vue'
import * as Misskey from 'misskey-js'
import XPie from './pie.vue'
import bytes from '@/filters/bytes.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'mem',
  props: {
    connection: { type: null, required: true },
    meta: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const usage = ref<number>(0);
const total = ref<number>(0);
const used = ref<number>(0);
const free = ref<number>(0);
function onStats(stats: Misskey.entities.ServerStats) {
	usage.value = stats.mem.active / props.meta.mem.total;
	total.value = props.meta.mem.total;
	used.value = stats.mem.active;
	free.value = total.value - used.value;
}
onMounted(() => {
	props.connection.on('stats', onStats);
});
onBeforeUnmount(() => {
	props.connection.off('stats', onStats);
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "zlxnikvl" }, [ _createVNode(XPie, {
        class: "pie",
        value: usage.value
      }, null, 8 /* PROPS */, ["value"]), _createElementVNode("div", null, [ _createElementVNode("p", null, [ _hoisted_1, _createTextVNode("RAM") ]), _createElementVNode("p", null, "Total: " + _toDisplayString(bytes(total.value, 1)), 1 /* TEXT */), _createElementVNode("p", null, "Used: " + _toDisplayString(bytes(used.value, 1)), 1 /* TEXT */), _createElementVNode("p", null, "Free: " + _toDisplayString(bytes(free.value, 1)), 1 /* TEXT */) ]) ]))
}
}

})
