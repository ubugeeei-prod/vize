import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-database" })
import { computed } from 'vue'
import * as Misskey from 'misskey-js'
import XPie from './pie.vue'
import bytes from '@/filters/bytes.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'disk',
  props: {
    meta: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const usage = computed(() => props.meta.fs.used / props.meta.fs.total);
const total = computed(() => props.meta.fs.total);
const used = computed(() => props.meta.fs.used);
const available = computed(() => props.meta.fs.total - props.meta.fs.used);

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "zbwaqsat" }, [ _createVNode(XPie, {
        class: "pie",
        value: usage.value
      }, null, 8 /* PROPS */, ["value"]), _createElementVNode("div", null, [ _createElementVNode("p", null, [ _hoisted_1, _createTextVNode("Disk") ]), _createElementVNode("p", null, "Total: " + _toDisplayString(bytes(total.value, 1)), 1 /* TEXT */), _createElementVNode("p", null, "Free: " + _toDisplayString(bytes(available.value, 1)), 1 /* TEXT */), _createElementVNode("p", null, "Used: " + _toDisplayString(bytes(used.value, 1)), 1 /* TEXT */) ]) ]))
}
}

})
