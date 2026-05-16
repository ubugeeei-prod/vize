import { useSlots as _useSlots } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"


const _hoisted_1 = { block: "true", "sm:hidden": "true", "xl:block": "true", "select-none": "true" }

export default /*@__PURE__*/_defineComponent({
  __name: 'NavSideItem',
  props: {
    text: { type: String, required: false },
    icon: { type: String, required: true },
    to: { type: [String, null], required: true },
    userOnly: { type: Boolean, required: false, default: false },
    command: { type: Boolean, required: false }
  },
  setup(__props: any) {

const router = useRouter()
useCommand({
  scope: 'Navigation',
  name: () => __props.text ?? (typeof __props.to === 'string' ? __props.to as string : __props.to.name),
  icon: () => __props.icon,
  visible: () => __props.command,
  onActivate() {
    router.push(__props.to)
  },
})
const activeClass = ref('text-primary')
onHydrated(async () => {
  // TODO: force NuxtLink to reevaluate, we now we are in this route though, so we should force it to active
  // we don't have currentServer defined until later
  activeClass.value = ''
  await nextTick()
  activeClass.value = 'text-primary'
})
// Optimize rendering for the common case of being logged in, only show visual feedback for disabled user-only items
// when we know there is no user.
const noUserDisable = computed(() => !isHydrated.value || (__props.userOnly && !currentUser.value))
const noUserVisual = computed(() => isHydrated.value && __props.userOnly && !currentUser.value)

return (_ctx: any,_cache: any) => {
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (_openBlock(), _createBlock(_component_NuxtLink, {
      to: __props.to,
      disabled: noUserDisable.value,
      class: _normalizeClass(noUserVisual.value ? 'op25 pointer-events-none ' : ''),
      "active-class": activeClass.value,
      group: "",
      "focus:outline-none": "",
      "disabled:pointer-events-none": "",
      tabindex: noUserDisable.value ? -1 : undefined,
      onClick: _cache[0] || (_cache[0] = (...args) => (_ctx.$scrollToTop && _ctx.$scrollToTop(...args)))
    }, {
      default: _withCtx(() => [
        _createVNode(_component_CommonTooltip, {
          disabled: !_ctx.isMediumOrLargeScreen,
          content: __props.text,
          placement: "right"
        }, {
          default: _withCtx(() => [
            _createElementVNode("div", {
              flex: "",
              "items-center": "",
              gap4: "",
              xl: "ml0 mr5 px5 w-auto",
              class: _normalizeClass(["item", _ctx.isSmallScreen
            ? `
              w-full
              px5 sm:mxa
              transition-colors duration-200 transform
              hover-bg-gray-100 hover-dark:(bg-gray-700 text-white)
            ` : `
              w-fit rounded-3
              px2 mx3 sm:mxa
              transition-100
              elk-group-hover-bg-active
              group-focus-visible:ring-2
              group-focus-visible:ring-current
            `])
            }, [
              _renderSlot(_ctx.$slots, "icon", {}, () => [
                _createElementVNode("div", {
                  class: _normalizeClass(__props.icon),
                  "text-xl": ""
                }, null, 2 /* CLASS */)
              ]),
              _renderSlot(_ctx.$slots, "default", {}, () => [
                _createElementVNode("span", _hoisted_1, _toDisplayString(_ctx.isHydrated ? __props.text : '&nbsp;'), 1 /* TEXT */)
              ])
            ])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["disabled", "content"])
      ]),
      _: 1 /* STABLE */
    }, 10 /* CLASS, PROPS */, ["to", "disabled", "active-class", "tabindex"]))
}
}

})
