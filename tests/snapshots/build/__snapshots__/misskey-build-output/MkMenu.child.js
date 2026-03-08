import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, normalizeClass as _normalizeClass } from "vue"

import { nextTick, onMounted, onUnmounted, provide, useTemplateRef, watch } from 'vue'
import MkMenu from './MkMenu.vue'
import type { MenuItem } from '@/types/menu.js'
const align = 'left';
const SCROLLBAR_THICKNESS = 16;

export default /*@__PURE__*/_defineComponent({
  __name: 'MkMenu.child',
  props: {
    items: { type: Array, required: true },
    anchorElement: { type: null, required: true },
    rootElement: { type: null, required: true },
    width: { type: Number, required: false }
  },
  emits: ["closed", "actioned"],
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
provide('isNestingMenu', true);
const el = useTemplateRef('el');
function setPosition() {
	if (el.value == null) return;
	const rootRect = props.rootElement.getBoundingClientRect();
	const parentRect = props.anchorElement.getBoundingClientRect();
	const myRect = el.value.getBoundingClientRect();
	let left = props.anchorElement.offsetWidth;
	let top = (parentRect.top - rootRect.top) - 8;
	if (rootRect.left + left + myRect.width >= (window.innerWidth - SCROLLBAR_THICKNESS)) {
		left = -myRect.width;
	}
	if (rootRect.top + top + myRect.height >= (window.innerHeight - SCROLLBAR_THICKNESS)) {
		top = top - ((rootRect.top + top + myRect.height) - (window.innerHeight - SCROLLBAR_THICKNESS));
	}
	el.value.style.left = left + 'px';
	el.value.style.top = top + 'px';
}
function onChildClosed(actioned?: boolean) {
	if (actioned) {
		emit('actioned');
	} else {
		emit('closed');
	}
}
watch(() => props.anchorElement, () => {
	setPosition();
});
const ro = new ResizeObserver((entries, observer) => {
	setPosition();
});
onMounted(() => {
	if (el.value) ro.observe(el.value);
	setPosition();
	nextTick(() => {
		setPosition();
	});
});
onUnmounted(() => {
	ro.disconnect();
});
__expose({
	checkHit: (ev: MouseEvent) => {
		return (ev.target === el.value || el.value?.contains(ev.target as Node));
	},
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      ref_key: "el", ref: el,
      class: _normalizeClass(_ctx.$style.root)
    }, [ _createVNode(MkMenu, {
        items: __props.items,
        align: align,
        width: __props.width,
        asDrawer: false,
        onClose: onChildClosed
      }, null, 8 /* PROPS */, ["items", "align", "width", "asDrawer"]) ], 512 /* NEED_PATCH */))
}
}

})
