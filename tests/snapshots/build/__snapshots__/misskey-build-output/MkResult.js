import { defineComponent as _defineComponent } from 'vue'
import { Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { style: "opacity: 0.7;" }
import { instance } from '@/instance.js'
import { i18n } from '@/i18n.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkResult',
  props: {
    type: { type: String, required: true },
    text: { type: String, required: false }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  const _component_MkSystemIcon = _resolveComponent("MkSystemIcon")

  return (_openBlock(), _createBlock(_Transition, {
      name: _unref(prefer).s.animation ? '_transition_zoom' : '',
      appear: ""
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(["_gaps", _ctx.$style.root])
        }, [
          (__props.type === 'empty' && _unref(instance).infoImageUrl)
            ? (_openBlock(), _createElementBlock("img", {
              key: 0,
              src: _unref(instance).infoImageUrl,
              draggable: "false",
              class: _normalizeClass(_ctx.$style.img)
            }))
            : (__props.type === 'empty')
              ? (_openBlock(), _createBlock(_component_MkSystemIcon, {
                key: 1,
                type: "info",
                class: _normalizeClass(_ctx.$style.icon)
              }))
            : _createCommentVNode("v-if", true),
          (__props.type === 'notFound' && _unref(instance).notFoundImageUrl)
            ? (_openBlock(), _createElementBlock("img", {
              key: 0,
              src: _unref(instance).notFoundImageUrl,
              draggable: "false",
              class: _normalizeClass(_ctx.$style.img)
            }))
            : (__props.type === 'notFound')
              ? (_openBlock(), _createBlock(_component_MkSystemIcon, {
                key: 1,
                type: "question",
                class: _normalizeClass(_ctx.$style.icon)
              }))
            : _createCommentVNode("v-if", true),
          (__props.type === 'error' && _unref(instance).serverErrorImageUrl)
            ? (_openBlock(), _createElementBlock("img", {
              key: 0,
              src: _unref(instance).serverErrorImageUrl,
              draggable: "false",
              class: _normalizeClass(_ctx.$style.img)
            }))
            : (__props.type === 'error')
              ? (_openBlock(), _createBlock(_component_MkSystemIcon, {
                key: 1,
                type: "error",
                class: _normalizeClass(_ctx.$style.icon)
              }))
            : _createCommentVNode("v-if", true),
          _createElementVNode("div", _hoisted_1, _toDisplayString(props.text ?? (__props.type === 'empty' ? _unref(i18n).ts.nothing : __props.type === 'notFound' ? _unref(i18n).ts.notFound : __props.type === 'error' ? _unref(i18n).ts.somethingHappened : null)), 1 /* TEXT */),
          _renderSlot(_ctx.$slots, "default")
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["name"]))
}
}

})
