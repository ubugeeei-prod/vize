import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { computed } from 'vue'
import { ensureSignin } from '@/i.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'avatar-decoration.decoration',
  props: {
    active: { type: Boolean, required: false },
    decoration: { type: Object, required: true },
    angle: { type: Number, required: false },
    flipH: { type: Boolean, required: false },
    offsetX: { type: Number, required: false },
    offsetY: { type: Number, required: false }
  },
  emits: ["click"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const $i = ensureSignin();
const locked = computed(() => props.decoration.roleIdsThatCanBeUsedThisDecoration.length > 0 && !$i.roles.some(r => props.decoration.roleIdsThatCanBeUsedThisDecoration.includes(r.id)));

return (_ctx: any,_cache: any) => {
  const _component_MkCondensedLine = _resolveComponent("MkCondensedLine")
  const _component_MkAvatar = _resolveComponent("MkAvatar")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.active]: __props.active }]),
      onClick: _cache[0] || (_cache[0] = ($event: any) => (emit('click')))
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.name)
      }, [ _createVNode(_component_MkCondensedLine, { minScale: 0.5 }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(__props.decoration.name), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["minScale"]) ]), _createVNode(_component_MkAvatar, {
        style: "width: 60px; height: 60px;",
        user: _unref($i),
        decorations: [{ url: __props.decoration.url, angle: __props.angle, flipH: __props.flipH, offsetX: __props.offsetX, offsetY: __props.offsetY }],
        forceShowDecoration: ""
      }, null, 8 /* PROPS */, ["user", "decorations"]), (locked.value) ? (_openBlock(), _createElementBlock("i", {
          key: 0,
          class: _normalizeClass(["ti ti-lock", _ctx.$style.lock])
        })) : _createCommentVNode("v-if", true) ], 2 /* CLASS */))
}
}

})
