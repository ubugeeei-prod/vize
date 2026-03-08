import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue"

import { onActivated, onDeactivated, onMounted, onBeforeUnmount, watch, computed, ref, useTemplateRef, inject } from 'vue'
import { DI } from '@/di.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'SearchMarker',
  props: {
    markerId: { type: String, required: false },
    label: { type: String, required: false },
    icon: { type: String, required: false },
    keywords: { type: Array, required: false },
    children: { type: Array, required: false },
    inlining: { type: Array, required: false }
  },
  setup(__props: any) {

const props = __props
const rootEl = useTemplateRef('root');
const rootElMutationObserver = new MutationObserver(() => {
	checkChildren();
});
const injectedSearchMarkerId = inject(DI.inAppSearchMarkerId, null);
const searchMarkerId = computed(() => injectedSearchMarkerId?.value ?? window.location.hash.slice(1));
const highlighted = ref(props.markerId === searchMarkerId.value);
const isParentOfTarget = computed(() => props.children?.includes(searchMarkerId.value));
function checkChildren() {
	if (isParentOfTarget.value) {
		const el = window.document.querySelector(`[data-in-app-search-marker-id="${searchMarkerId.value}"]`);
		highlighted.value = el == null;
	}
}
watch([
	searchMarkerId,
	() => props.children,
], () => {
	if (props.children != null && props.children.length > 0) {
		checkChildren();
	}
}, { flush: 'post' });
function init() {
	checkChildren();
	if (highlighted.value) {
		rootEl.value?.scrollIntoView({
			behavior: 'smooth',
			block: 'center',
		});
	}
	if (rootEl.value != null) {
		rootElMutationObserver.observe(rootEl.value, {
			childList: true,
			subtree: true,
		});
	}
}
function dispose() {
	rootElMutationObserver.disconnect();
}
onMounted(init);
onActivated(init);
onDeactivated(dispose);
onBeforeUnmount(dispose);

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      ref: "root",
      class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.highlighted]: highlighted.value }])
    }, [ _renderSlot(_ctx.$slots, "default", { isParentOfTarget: isParentOfTarget.value }) ], 2 /* CLASS */))
}
}

})
