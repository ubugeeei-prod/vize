import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref, vModelText as _vModelText } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-search" })
import { ref } from 'vue'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkGoogle',
  props: {
    q: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const query = ref(props.q);
const search = () => {
	const sp = new URLSearchParams();
	sp.append('q', query.value);
	window.open(`https://www.google.com/search?${sp.toString()}`, '_blank', 'noopener');
};

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ _withDirectives(_createElementVNode("input", {
        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((query).value = $event)),
        class: _normalizeClass(_ctx.$style.input),
        type: "search",
        placeholder: __props.q
      }, null, 8 /* PROPS */, ["placeholder"]), [ [_vModelText, query.value] ]), _createElementVNode("button", {
        class: _normalizeClass(_ctx.$style.button),
        onClick: search
      }, [ _hoisted_1, _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.searchByGoogle), 1 /* TEXT */) ]) ]))
}
}

})
