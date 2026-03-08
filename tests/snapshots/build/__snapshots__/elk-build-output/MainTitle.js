import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveDynamicComponent as _resolveDynamicComponent, renderSlot as _renderSlot, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'MainTitle',
  props: {
    as: { type: String, required: false, default: 'div' },
    icon: { type: String, required: false },
    secondary: { type: Boolean, required: false }
  },
  setup(__props: any) {

function doScroll({ currentTarget, metaKey, ctrlKey }: MouseEvent | KeyboardEvent) {
  // Ctrl+click or Command+click to open the link in a new tab
  // should not scroll the current tab's timeline
  if ((metaKey || ctrlKey) && (currentTarget as HTMLElement)?.nodeName === 'A') {
    return
  }
  useNuxtApp().$scrollToTop()
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(_resolveDynamicComponent(__props.as), {
      class: _normalizeClass(["\n      flex items-center gap-2 min-h-10 px-3\n      text-start text-lg lh-tight font-bold cursor-pointer\n    ", { 'text-primary': !__props.secondary }]),
      onClick: _cache[0] || (_cache[0] = ($event: any) => (doScroll($event)))
    }, {
      default: _withCtx(() => [
        (__props.icon)
          ? (_openBlock(), _createElementBlock("span", {
            key: 0,
            class: _normalizeClass(__props.icon)
          }))
          : _createCommentVNode("v-if", true),
        _createElementVNode("span", {
          "min-w-8": "",
          "line-clamp-2": ""
        }, [
          _renderSlot(_ctx.$slots, "default")
        ])
      ]),
      _: 1 /* STABLE */
    }, 2 /* CLASS */))
}
}

})
