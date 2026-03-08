import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
import MkButton from './MkButton.vue'
import type { useForm } from '@/composables/use-form.js'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkFormFooter',
  props: {
    form: { type: null, required: true },
    canSaving: { type: Boolean, required: false, default: true }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  return (__props.form.modified.value)
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        class: _normalizeClass(_ctx.$style.root)
      }, [ _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.text)
        }, _toDisplayString(_unref(i18n).tsx.thereAreNChanges({ n: __props.form.modifiedCount.value })), 1 /* TEXT */), _createElementVNode("div", {
          style: "margin-left: auto;",
          class: "_buttons"
        }, [ _createVNode(MkButton, {
            danger: "",
            rounded: "",
            onClick: _cache[0] || (_cache[0] = ($event: any) => (__props.form.discard))
          }, {
            default: _withCtx(() => [
              _hoisted_1,
              _createTextVNode(" "),
              _createTextVNode(_toDisplayString(_unref(i18n).ts.discard), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }), _createVNode(MkButton, {
            primary: "",
            rounded: "",
            disabled: !__props.canSaving,
            onClick: _cache[1] || (_cache[1] = ($event: any) => (__props.form.save))
          }, {
            default: _withCtx(() => [
              _hoisted_2,
              _createTextVNode(" "),
              _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["disabled"]) ]) ]))
      : _createCommentVNode("v-if", true)
}
}

})
