import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, normalizeClass as _normalizeClass } from "vue"

import { nextTick, onMounted, onActivated, onBeforeUnmount, ref, useTemplateRef } from 'vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkLazy',
  setup(__props) {

const rootEl = useTemplateRef('rootEl');
const showing = ref(false);
const observer = new IntersectionObserver(
	(entries) => {
		if (entries.some((entry) => entry.isIntersecting)) {
			showing.value = true;
		}
	},
);
onMounted(() => {
	nextTick(() => {
		observer.observe(rootEl.value!);
	});
});
onActivated(() => {
	nextTick(() => {
		observer.observe(rootEl.value!);
	});
});
onBeforeUnmount(() => {
	observer.disconnect();
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      ref_key: "rootEl", ref: rootEl,
      class: _normalizeClass(_ctx.$style.root)
    }, [ (!showing.value) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: _normalizeClass(_ctx.$style.placeholder)
        })) : (_openBlock(), _createElementBlock("slot", { key: 1 })) ], 512 /* NEED_PATCH */))
}
}

})
