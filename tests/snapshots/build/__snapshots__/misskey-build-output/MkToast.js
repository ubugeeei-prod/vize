import { defineComponent as _defineComponent } from 'vue'
import { Transition as _Transition, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { style: "padding: 16px 24px;" }
import { onMounted, ref } from 'vue'
import * as os from '@/os.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkToast',
  props: {
    message: { type: String, required: true }
  },
  emits: ["closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const zIndex = os.claimZIndex('high');
const showing = ref(true);
onMounted(() => {
	window.setTimeout(() => {
		showing.value = false;
	}, 4000);
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", null, [ _createVNode(_Transition, {
        enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_toast_enterActive : '',
        leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_toast_leaveActive : '',
        enterFromClass: _unref(prefer).s.animation ? _ctx.$style.transition_toast_enterFrom : '',
        leaveToClass: _unref(prefer).s.animation ? _ctx.$style.transition_toast_leaveTo : '',
        appear: "",
        onAfterLeave: _cache[0] || (_cache[0] = ($event: any) => (emit('closed')))
      }, {
        default: _withCtx(() => [
          (showing.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(["_acrylic", _ctx.$style.root]),
              style: _normalizeStyle({ zIndex: _unref(zIndex) })
            }, [
              _createElementVNode("div", _hoisted_1, _toDisplayString(__props.message), 1 /* TEXT */)
            ]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass"]) ]))
}
}

})
