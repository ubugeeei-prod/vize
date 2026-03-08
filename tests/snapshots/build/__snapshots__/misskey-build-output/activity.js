import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-activity" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-users" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye" })
import * as Misskey from 'misskey-js'
import XPv from './activity.pv.vue'
import XNotes from './activity.notes.vue'
import XFollowing from './activity.following.vue'
import MkFoldableSection from '@/components/MkFoldableSection.vue'
import MkHeatmap from '@/components/MkHeatmap.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'activity',
  props: {
    user: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: "_spacer",
      style: "--MI_SPACER-w: 700px;"
    }, [ _createElementVNode("div", { class: "_gaps" }, [ _createVNode(MkFoldableSection, { class: "item" }, {
          header: _withCtx(() => [
            _hoisted_1,
            _createTextVNode(" Heatmap")
          ]),
          default: _withCtx(() => [
            _createVNode(MkHeatmap, {
              user: __props.user,
              src: 'notes'
            }, null, 8 /* PROPS */, ["user", "src"])
          ]),
          _: 1 /* STABLE */
        }), _createVNode(MkFoldableSection, { class: "item" }, {
          header: _withCtx(() => [
            _hoisted_2,
            _createTextVNode(" Notes")
          ]),
          default: _withCtx(() => [
            _createVNode(XNotes, { user: __props.user }, null, 8 /* PROPS */, ["user"])
          ]),
          _: 1 /* STABLE */
        }), _createVNode(MkFoldableSection, { class: "item" }, {
          header: _withCtx(() => [
            _hoisted_3,
            _createTextVNode(" Following")
          ]),
          default: _withCtx(() => [
            _createVNode(XFollowing, { user: __props.user }, null, 8 /* PROPS */, ["user"])
          ]),
          _: 1 /* STABLE */
        }), _createVNode(MkFoldableSection, { class: "item" }, {
          header: _withCtx(() => [
            _hoisted_4,
            _createTextVNode(" PV")
          ]),
          default: _withCtx(() => [
            _createVNode(XPv, { user: __props.user }, null, 8 /* PROPS */, ["user"])
          ]),
          _: 1 /* STABLE */
        }) ]) ]))
}
}

})
