import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"


const _hoisted_1 = { style: "font-weight: bold;" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("hr")
import * as Misskey from 'misskey-js'

export default /*@__PURE__*/_defineComponent({
  __name: 'XRoom',
  props: {
    room: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(), _createBlock(_component_MkA, {
      to: `/chat/room/${__props.room.id}`,
      class: _normalizeClass(["_panel _gaps_s", _ctx.$style.root])
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.header)
        }, [
          _createElementVNode("div", _hoisted_1, _toDisplayString(__props.room.name), 1 /* TEXT */),
          _createVNode(_component_MkAvatar, {
            user: __props.room.owner,
            link: false,
            class: _normalizeClass(_ctx.$style.headerAvatar)
          }, null, 8 /* PROPS */, ["user", "link"])
        ]),
        _hoisted_2,
        _createElementVNode("div", null, _toDisplayString(__props.room.description), 1 /* TEXT */)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["to"]))
}
}

})
