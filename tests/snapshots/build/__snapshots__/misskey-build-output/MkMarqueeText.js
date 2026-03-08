import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderList as _renderList, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue"

import { onMounted, useTemplateRef, watch } from 'vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkMarqueeText',
  props: {
    duration: { type: Number, required: false, default: 15 },
    repeat: { type: Number, required: false, default: 2 },
    paused: { type: Boolean, required: false, default: false },
    reverse: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {

const props = __props
const contentEl = useTemplateRef('contentEl');
function calcDuration() {
	if (contentEl.value == null) return;
	const eachLength = contentEl.value.offsetWidth / props.repeat;
	const factor = 3000;
	const duration = props.duration / ((1 / eachLength) * factor);
	contentEl.value.style.animationDuration = `${duration}s`;
}
watch(() => props.duration, calcDuration);
onMounted(calcDuration);

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.wrap)
    }, [ _createElementVNode("span", {
        ref_key: "contentEl", ref: contentEl,
        class: _normalizeClass([_ctx.$style.content, {
  			[_ctx.$style.paused]: __props.paused,
  			[_ctx.$style.reverse]: __props.reverse,
  		}])
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.repeat, (key) => {
          return (_openBlock(), _createElementBlock("span", {
            key: key,
            class: _normalizeClass(_ctx.$style.text)
          }, [
            _renderSlot(_ctx.$slots, "default")
          ]))
        }), 128 /* KEYED_FRAGMENT */)) ], 2 /* CLASS */) ]))
}
}

})
