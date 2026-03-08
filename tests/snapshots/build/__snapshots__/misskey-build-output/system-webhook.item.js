import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, createSlots as _createSlots, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-settings" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
import { entities } from 'misskey-js'
import { toRefs } from 'vue'
import MkFolder from '@/components/MkFolder.vue'
import { i18n } from '@/i18n.js'
import MkButton from '@/components/MkButton.vue'
import MkKeyValue from '@/components/MkKeyValue.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'system-webhook.item',
  props: {
    entity: { type: null, required: true }
  },
  emits: ["edit", "delete"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const { entity } = toRefs(props);
function onEditClick() {
	emit('edit', entity.value);
}
function onDeleteClick() {
	emit('delete', entity.value);
}

return (_ctx: any,_cache: any) => {
  const _component_MkTime = _resolveComponent("MkTime")

  return (_openBlock(), _createBlock(MkFolder, null, _createSlots({ _: 2 /* DYNAMIC */ }, [ {
        name: "label",
        fn: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(entity).name || _unref(entity).url), 1 /* TEXT */)
        ])
      }, (_unref(entity).name != null && _unref(entity).name != '') ? {
          name: "caption",
          fn: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(entity).url), 1 /* TEXT */)
          ]),
          key: "0"
        } : undefined, {
        name: "icon",
        fn: _withCtx(() => [
          (!_unref(entity).isActive)
            ? (_openBlock(), _createElementBlock("i", {
              key: 0,
              class: "ti ti-player-pause"
            }))
            : (_unref(entity).latestStatus === null)
              ? (_openBlock(), _createElementBlock("i", {
                key: 1,
                class: "ti ti-circle"
              }))
            : ([200, 201, 204].includes(_unref(entity).latestStatus))
              ? (_openBlock(), _createElementBlock("i", {
                key: 2,
                class: "ti ti-check",
                style: { color: 'var(--MI_THEME-success)' }
              }))
            : (_openBlock(), _createElementBlock("i", {
              key: 3,
              class: "ti ti-alert-triangle",
              style: { color: 'var(--MI_THEME-error)' }
            }))
        ])
      }, {
        name: "suffix",
        fn: _withCtx(() => [
          (_unref(entity).latestSentAt)
            ? (_openBlock(), _createBlock(_component_MkTime, {
              key: 0,
              time: _unref(entity).latestSentAt,
              style: "margin-right: 8px"
            }, null, 8 /* PROPS */, ["time"]))
            : (_openBlock(), _createElementBlock("span", { key: 1 }, "-"))
        ])
      }, {
        name: "footer",
        fn: _withCtx(() => [
          _createElementVNode("div", { class: "_buttons" }, [
            _createVNode(MkButton, { onClick: onEditClick }, {
              default: _withCtx(() => [
                _hoisted_1,
                _createTextVNode(" "),
                _createTextVNode(_toDisplayString(_unref(i18n).ts.edit), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }),
            _createVNode(MkButton, {
              danger: "",
              onClick: onDeleteClick
            }, {
              default: _withCtx(() => [
                _hoisted_2,
                _createTextVNode(" "),
                _createTextVNode(_toDisplayString(_unref(i18n).ts.delete), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            })
          ])
        ])
      } ]), 1024 /* DYNAMIC_SLOTS */))
}
}

})
