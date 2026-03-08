import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, withDirectives as _withDirectives, renderSlot as _renderSlot, vShow as _vShow } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-menu-2" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-up" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-down" })
import { ref } from 'vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'page-editor.container',
  props: {
    expanded: { type: Boolean, required: false, default: true },
    removable: { type: Boolean, required: false, default: true },
    draggable: { type: Boolean, required: false }
  },
  emits: ["toggle", "remove"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const showBody = ref(props.expanded);
function toggleContent(show: boolean) {
	showBody.value = show;
	emit('toggle', show);
}
function remove() {
	emit('remove');
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "cpjygsrt" }, [ _createElementVNode("header", null, [ _createElementVNode("div", { class: "title" }, [ _renderSlot(_ctx.$slots, "header") ]), _createElementVNode("div", { class: "buttons" }, [ _renderSlot(_ctx.$slots, "func"), (__props.removable) ? (_openBlock(), _createElementBlock("button", {
              key: 0,
              class: "_button",
              onClick: _cache[0] || (_cache[0] = ($event: any) => (remove()))
            }, [ _hoisted_1 ])) : _createCommentVNode("v-if", true), (__props.draggable) ? (_openBlock(), _createElementBlock("button", {
              key: 0,
              class: "drag-handle _button"
            }, [ _hoisted_2 ])) : _createCommentVNode("v-if", true), _createElementVNode("button", {
            class: "_button",
            onClick: _cache[1] || (_cache[1] = ($event: any) => (toggleContent(!showBody.value)))
          }, [ (showBody.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _hoisted_3 ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ _hoisted_4 ], 64 /* STABLE_FRAGMENT */)) ]) ]) ]), _withDirectives(_createElementVNode("div", { class: "body" }, [ _renderSlot(_ctx.$slots, "default") ], 512 /* NEED_PATCH */), [ [_vShow, showBody.value] ]) ]))
}
}

})
