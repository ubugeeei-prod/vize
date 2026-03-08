import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-reload" })
import { ref, watch } from 'vue'
import MkButton from '@/components/MkButton.vue'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSuspense',
  props: {
    p: { type: Function, required: true }
  },
  emits: ["resolved"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const pending = ref(true);
const resolved = ref(false);
const rejected = ref(false);
const result = ref<T | null>(null);
const error = ref<any | null>(null);
const process = () => {
	const promise = props.p();
	pending.value = true;
	resolved.value = false;
	rejected.value = false;
	promise.then((_result) => {
		pending.value = false;
		resolved.value = true;
		result.value = _result;
		emit('resolved', _result);
	});
	promise.catch((_error) => {
		pending.value = false;
		rejected.value = true;
		error.value = _error;
	});
};
watch(() => props.p, () => {
	process();
}, {
	immediate: true,
});
const retry = () => {
	process();
};

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (pending.value)
      ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createVNode(_component_MkLoading) ]))
      : (resolved.value)
        ? (_openBlock(), _createElementBlock("div", { key: 1 }, [ _renderSlot(_ctx.$slots, "default", { result: result.value }) ]))
      : (_openBlock(), _createElementBlock("div", { key: 2 }, [ _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.error)
        }, [ _renderSlot(_ctx.$slots, "error", {
            name: "error",
            error: error.value
          }, () => [ _createElementVNode("div", null, [ _hoisted_1, _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.somethingHappened), 1 /* TEXT */) ]), (error.value) ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(JSON.stringify(error.value)), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createVNode(MkButton, {
              inline: "",
              style: "margin-top: 16px;",
              onClick: retry
            }, {
              default: _withCtx(() => [
                _hoisted_2,
                _createTextVNode(" "),
                _createTextVNode(_toDisplayString(_unref(i18n).ts.retry), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }) ]) ]) ]))
}
}

})
