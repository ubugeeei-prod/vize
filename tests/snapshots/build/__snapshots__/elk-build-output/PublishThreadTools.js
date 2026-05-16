import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { "text-secondary-light": "true" }, "/")
const _hoisted_2 = { "text-secondary-light": "true" }

export default /*@__PURE__*/_defineComponent({
  __name: 'PublishThreadTools',
  props: {
    draftKey: { type: null, required: true },
    draftItemIndex: { type: Number, required: true }
  },
  setup(__props: any) {

const { threadIsActive, addThreadItem, threadItems, removeThreadItem } = useThreadComposer(__props.draftKey)
const isRemovableItem = computed(() => threadIsActive.value && __props.draftItemIndex < threadItems.value.length - 1)
function addOrRemoveItem() {
  if (isRemovableItem.value)
    removeThreadItem(__props.draftItemIndex)
  else
    addThreadItem()
}
const { t } = useI18n()
const label = computed(() => {
  if (!isRemovableItem.value && __props.draftItemIndex === 0)
    return t('tooltip.start_thread')

  return isRemovableItem.value ? t('tooltip.remove_thread_item') : t('tooltip.add_thread_item')
})

return (_ctx: any,_cache: any) => {
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")

  return (_openBlock(), _createElementBlock("div", {
      flex: "",
      "flex-row": "",
      "rounded-3": "",
      class: _normalizeClass({ 'bg-border': _unref(threadIsActive) })
    }, [ (_unref(threadIsActive)) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          dir: "ltr",
          "pointer-events-none": "",
          "pe-1": "",
          "pt-2": "",
          "pl-2": "",
          "text-sm": "",
          "tabular-nums": "",
          "text-secondary": "",
          flex: "",
          gap: "0.5"
        }, [ _toDisplayString(__props.draftItemIndex + 1), _hoisted_1, _createElementVNode("span", _hoisted_2, _toDisplayString(_unref(threadItems).length), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true), _createVNode(_component_CommonTooltip, {
        placement: "top",
        content: label.value
      }, {
        default: _withCtx(() => [
          _createElementVNode("button", {
            "btn-action-icon": "",
            "aria-label": label.value,
            onClick: addOrRemoveItem
          }, [
            (isRemovableItem.value)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                "i-ri:chat-delete-line": ""
              }))
              : (_openBlock(), _createElementBlock("div", {
                key: 1,
                "i-ri:chat-new-line": ""
              }))
          ], 8 /* PROPS */, ["aria-label"])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["content"]) ], 2 /* CLASS */))
}
}

})
