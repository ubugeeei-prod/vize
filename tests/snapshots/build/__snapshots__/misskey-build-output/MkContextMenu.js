import { defineComponent as _defineComponent } from 'vue'
import { Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"

import { onMounted, onBeforeUnmount, useTemplateRef, ref } from 'vue'
import MkMenu from './MkMenu.vue'
import type { MenuItem } from '@/types/menu.js'
import { elementContains } from '@/utility/element-contains.js'
import { prefer } from '@/preferences.js'
import * as os from '@/os.js'
const SCROLLBAR_THICKNESS = 16;

export default /*@__PURE__*/_defineComponent({
  __name: 'MkContextMenu',
  props: {
    items: { type: Array, required: true },
    ev: { type: null, required: true }
  },
  emits: ["closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const rootEl = useTemplateRef('rootEl');
const zIndex = ref<number>(os.claimZIndex('high'));
onMounted(() => {
	let left = props.ev.pageX + 1; // 間違って右ダブルクリックした場合に意図せずアイテムがクリックされるのを防ぐため + 1
	let top = props.ev.pageY + 1; // 間違って右ダブルクリックした場合に意図せずアイテムがクリックされるのを防ぐため + 1
	const width = rootEl.value!.offsetWidth;
	const height = rootEl.value!.offsetHeight;
	if (left + width - window.scrollX >= (window.innerWidth - SCROLLBAR_THICKNESS)) {
		left = (window.innerWidth - SCROLLBAR_THICKNESS) - width + window.scrollX;
	}
	if (top + height - window.scrollY >= (window.innerHeight - SCROLLBAR_THICKNESS)) {
		top = (window.innerHeight - SCROLLBAR_THICKNESS) - height + window.scrollY;
	}
	if (top < 0) {
		top = 0;
	}
	if (left < 0) {
		left = 0;
	}
	if (rootEl.value) {
		rootEl.value.style.top = `${top}px`;
		rootEl.value.style.left = `${left}px`;
	}
	window.document.body.addEventListener('mousedown', onMousedown);
});
onBeforeUnmount(() => {
	window.document.body.removeEventListener('mousedown', onMousedown);
});
function onMousedown(evt: MouseEvent) {
	if (!elementContains(rootEl.value, evt.target as Element) && (rootEl.value !== evt.target)) emit('closed');
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(_Transition, {
      appear: "",
      enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_fade_enterActive : '',
      leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_fade_leaveActive : '',
      enterFromClass: _unref(prefer).s.animation ? _ctx.$style.transition_fade_enterFrom : '',
      leaveToClass: _unref(prefer).s.animation ? _ctx.$style.transition_fade_leaveTo : ''
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          ref_key: "rootEl", ref: rootEl,
          class: _normalizeClass(_ctx.$style.root),
          style: _normalizeStyle({ zIndex: zIndex.value }),
          onContextmenu: _cache[0] || (_cache[0] = _withModifiers(() => {}, ["prevent","stop"]))
        }, [
          _createVNode(MkMenu, {
            items: __props.items,
            align: 'left',
            onClose: _cache[1] || (_cache[1] = ($event: any) => (emit('closed')))
          }, null, 8 /* PROPS */, ["items", "align"])
        ], 36 /* STYLE, NEED_HYDRATION */)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass"]))
}
}

})
