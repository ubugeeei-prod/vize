import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderSlot as _renderSlot, normalizeStyle as _normalizeStyle } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'Sidebar',
  setup(__props) {

const viewport = useWindowSize()
const scroll = useWindowScroll()
const container = useTemplateRef<HTMLDivElement>('container')
const content = useTemplateRef<HTMLDivElement>('content')
const bounds = useElementBounding(content)
const active = computed(() => {
  return bounds.height.value > viewport.height.value
})
const direction = computed((previous = 'up'): string => {
  if (!active.value) return 'up'
  return scroll.directions.bottom ? 'down' : scroll.directions.top ? 'up' : previous
})
const offset = computed(() => {
  if (!active.value) return 0
  if (!container.value) return 0
  if (!content.value) return 0

  return direction.value === 'down'
    ? content.value.offsetTop
    : container.value.offsetHeight - content.value.offsetTop - content.value.offsetHeight
})
const style = computed(() => {
  return direction.value === 'down'
    ? { paddingBlockStart: `${offset.value}px` }
    : { paddingBlockEnd: `${offset.value}px` }
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      ref_key: "container", ref: container,
      class: "group relative data-[active=true]:flex",
      "data-direction": direction.value,
      "data-active": active.value,
      style: _normalizeStyle(style.value)
    }, [ _createElementVNode("div", {
        ref_key: "content", ref: content,
        class: "sticky w-full group-data-[direction=up]:(self-start top-30 xl:top-14) group-data-[direction=down]:(self-end bottom-8)"
      }, [ _renderSlot(_ctx.$slots, "default") ], 512 /* NEED_PATCH */) ], 12 /* STYLE, PROPS */, ["data-direction", "data-active"]))
}
}

})
