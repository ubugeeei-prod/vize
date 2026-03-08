import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, resolveComponent as _resolveComponent, renderList as _renderList, renderSlot as _renderSlot, withCtx as _withCtx } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'PublishVisibilityPicker',
  props: {
    editing: { type: Boolean, required: false },
    "modelValue": {
  required: true,
}
  },
  emits: ["update:modelValue"],
  setup(__props: any) {

const modelValue = _useModel(__props, "modelValue")
const currentVisibility = computed(() =>
  statusVisibilities.find(v => v.value === modelValue.value) || statusVisibilities[0],
)
function chooseVisibility(visibility: mastodon.v1.StatusVisibility) {
  modelValue.value = visibility
}

return (_ctx: any,_cache: any) => {
  const _component_CommonDropdownItem = _resolveComponent("CommonDropdownItem")
  const _component_CommonDropdown = _resolveComponent("CommonDropdown")
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")

  return (_openBlock(), _createBlock(_component_CommonTooltip, {
      placement: "top",
      content: __props.editing ? _ctx.$t(`visibility.${currentVisibility.value}`) : _ctx.$t('tooltip.change_content_visibility')
    }, {
      default: _withCtx(() => [
        _createVNode(_component_CommonDropdown, { placement: "bottom" }, {
          popper: _withCtx(() => [
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.statusVisibilities, (visibility) => {
              return (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                key: visibility.value,
                icon: visibility.icon,
                text: _ctx.$t(`visibility.${visibility.value}`),
                description: _ctx.$t(`visibility.${visibility.value}_desc`),
                checked: visibility.value === modelValue.value,
                onClick: _cache[0] || (_cache[0] = ($event: any) => (chooseVisibility(visibility.value)))
              }, null, 8 /* PROPS */, ["icon", "text", "description", "checked"]))
            }), 128 /* KEYED_FRAGMENT */))
          ]),
          default: _withCtx(() => [
            _renderSlot(_ctx.$slots, "default", { visibility: currentVisibility.value })
          ]),
          _: 1 /* STABLE */
        })
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["content"]))
}
}

})
