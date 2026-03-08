import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, withDirectives as _withDirectives, renderSlot as _renderSlot, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-up" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-down" })
import { onBeforeUnmount, onMounted, ref, useTemplateRef, watch } from 'vue'
import { miLocalStorage } from '@/local-storage.js'
import { prefer } from '@/preferences.js'
import { globalEvents } from '@/events.js'
import { getBgColor } from '@/utility/get-bg-color.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkFoldableSection',
  props: {
    expanded: { type: Boolean, required: false, default: true },
    persistKey: { type: String, required: false, default: null }
  },
  setup(__props: any) {

const props = __props
const miLocalStoragePrefix = 'ui:folder:' as const;
const rootEl = useTemplateRef('rootEl');
const parentBg = ref<string | null>(null);
// eslint-disable-next-line vue/no-setup-props-reactivity-loss
const showBody = ref((props.persistKey && miLocalStorage.getItem(`${miLocalStoragePrefix}${props.persistKey}`)) ? (miLocalStorage.getItem(`${miLocalStoragePrefix}${props.persistKey}`) === 't') : props.expanded);
watch(showBody, () => {
	if (props.persistKey) {
		miLocalStorage.setItem(`${miLocalStoragePrefix}${props.persistKey}`, showBody.value ? 't' : 'f');
	}
});
function enter(el: Element) {
	if (!(el instanceof HTMLElement)) return;
	const elementHeight = el.getBoundingClientRect().height;
	el.style.height = '0';
	el.offsetHeight; // reflow
	el.style.height = `${elementHeight}px`;
}
function afterEnter(el: Element) {
	if (!(el instanceof HTMLElement)) return;
	el.style.height = '';
}
function leave(el: Element) {
	if (!(el instanceof HTMLElement)) return;
	const elementHeight = el.getBoundingClientRect().height;
	el.style.height = `${elementHeight}px`;
	el.offsetHeight; // reflow
	el.style.height = '0';
}
function afterLeave(el: Element) {
	if (!(el instanceof HTMLElement)) return;
	el.style.height = '';
}
function updateBgColor() {
	if (rootEl.value) {
		parentBg.value = getBgColor(rootEl.value.parentElement);
	}
}
onMounted(() => {
	updateBgColor();
	globalEvents.on('themeChanging', updateBgColor);
});
onBeforeUnmount(() => {
	globalEvents.off('themeChanging', updateBgColor);
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      ref_key: "rootEl", ref: rootEl,
      class: _normalizeClass(_ctx.$style.root)
    }, [ _createElementVNode("header", {
        class: _normalizeClass(["_button", _ctx.$style.header]),
        onClick: _cache[0] || (_cache[0] = ($event: any) => (showBody.value = !showBody.value))
      }, [ _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.title)
        }, [ _createElementVNode("div", null, [ _renderSlot(_ctx.$slots, "header") ]) ]), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.divider)
        }), _createElementVNode("button", {
          class: _normalizeClass(["_button", _ctx.$style.button])
        }, [ (showBody.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _hoisted_1 ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ _hoisted_2 ], 64 /* STABLE_FRAGMENT */)) ]) ]), _createVNode(_Transition, {
        enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.folderToggleEnterActive : '',
        leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.folderToggleLeaveActive : '',
        enterFromClass: _unref(prefer).s.animation ? _ctx.$style.folderToggleEnterFrom : '',
        leaveToClass: _unref(prefer).s.animation ? _ctx.$style.folderToggleLeaveTo : '',
        onEnter: enter,
        onAfterEnter: afterEnter,
        onLeave: leave,
        onAfterLeave: afterLeave
      }, {
        default: _withCtx(() => [
          _withDirectives(_createElementVNode("div", null, [
            _renderSlot(_ctx.$slots, "default")
          ], 512 /* NEED_PATCH */), [
            [_vShow, showBody.value]
          ])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass"]) ], 512 /* NEED_PATCH */))
}
}

})
