import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue"

import { onMounted, onUnmounted, provide, inject, ref, watch, useTemplateRef } from 'vue'
import { DI } from '@/di.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkStickyContainer',
  setup(__props, { expose: __expose }) {

const rootEl = useTemplateRef('rootEl');
const headerEl = useTemplateRef('headerEl');
const footerEl = useTemplateRef('footerEl');
const headerHeight = ref<string | undefined>();
const childStickyTop = ref(0);
const parentStickyTop = inject(DI.currentStickyTop, ref(0));
provide(DI.currentStickyTop, childStickyTop);
const footerHeight = ref<string | undefined>();
const childStickyBottom = ref(0);
const parentStickyBottom = inject(DI.currentStickyBottom, ref(0));
provide(DI.currentStickyBottom, childStickyBottom);
const calc = () => {
	// コンポーネントが表示されてないけどKeepAliveで残ってる場合などは null になる
	if (headerEl.value != null) {
		childStickyTop.value = parentStickyTop.value + headerEl.value.offsetHeight;
		headerHeight.value = headerEl.value.offsetHeight.toString();
	}

	// コンポーネントが表示されてないけどKeepAliveで残ってる場合などは null になる
	if (footerEl.value != null) {
		childStickyBottom.value = parentStickyBottom.value + footerEl.value.offsetHeight;
		footerHeight.value = footerEl.value.offsetHeight.toString();
	}
};
const observer = new ResizeObserver(() => {
	window.setTimeout(() => {
		calc();
	}, 100);
});
onMounted(() => {
	calc();
	watch([parentStickyTop, parentStickyBottom], calc);
	if (headerEl.value != null) {
		observer.observe(headerEl.value);
	}
	if (footerEl.value != null) {
		observer.observe(footerEl.value);
	}
});
onUnmounted(() => {
	observer.disconnect();
});
__expose({
	rootEl,
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { ref_key: "rootEl", ref: rootEl }, [ _createElementVNode("div", {
        ref_key: "headerEl", ref: headerEl,
        class: _normalizeClass(_ctx.$style.header)
      }, [ _renderSlot(_ctx.$slots, "header") ], 512 /* NEED_PATCH */), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.body),
        "data-sticky-container-header-height": headerHeight.value,
        "data-sticky-container-footer-height": footerHeight.value
      }, [ _renderSlot(_ctx.$slots, "default") ], 8 /* PROPS */, ["data-sticky-container-header-height", "data-sticky-container-footer-height"]), _createElementVNode("div", {
        ref_key: "footerEl", ref: footerEl,
        class: _normalizeClass(_ctx.$style.footer)
      }, [ _renderSlot(_ctx.$slots, "footer") ], 512 /* NEED_PATCH */) ], 512 /* NEED_PATCH */))
}
}

})
