import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveDynamicComponent as _resolveDynamicComponent, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { shallowRef, computed } from 'vue'
import { LinkBase } from '#components'

interface Props {
  title: string
  subtitle?: string
  isLoading?: boolean
  headingLevel?: `h${number}`
  id: string
  icon?: string
}

export default /*@__PURE__*/_defineComponent({
  __name: 'CollapsibleSection',
  props: {
    title: { type: String, required: true },
    subtitle: { type: String, required: false },
    isLoading: { type: Boolean, required: false, default: false },
    headingLevel: { type: null, required: false, default: 'h2' },
    id: { type: String, required: true },
    icon: { type: String, required: false }
  },
  setup(__props: any) {

const props = __props
const appSettings = useSettings()
const buttonId = `${props.id}-collapsible-button`
const contentId = `${props.id}-collapsible-content`
const isOpen = shallowRef(true)
onPrehydrate(() => {
  const settings = JSON.parse(localStorage.getItem('npmx-settings') || '{}')
  const collapsed: string[] = settings?.sidebar?.collapsed || []
  for (const id of collapsed) {
    if (!document.documentElement.dataset.collapsed?.split(' ').includes(id)) {
      document.documentElement.dataset.collapsed = (
        document.documentElement.dataset.collapsed +
        ' ' +
        id
      ).trim()
    }
  }
})
onMounted(() => {
  if (document?.documentElement) {
    isOpen.value = !(
      document.documentElement.dataset.collapsed?.split(' ').includes(props.id) ?? false
    )
  }
})
function toggle() {
  isOpen.value = !isOpen.value
  const removed = appSettings.settings.value.sidebar.collapsed.filter(c => c !== props.id)
  if (isOpen.value) {
    appSettings.settings.value.sidebar.collapsed = removed
  } else {
    removed.push(props.id)
    appSettings.settings.value.sidebar.collapsed = removed
  }
  document.documentElement.dataset.collapsed =
    appSettings.settings.value.sidebar.collapsed.join(' ')
}
const ariaLabel = computed(() => {
  const action = isOpen.value ? 'Collapse' : 'Expand'
  return props.title ? `${action} ${props.title}` : action
})
useHead({
  style: [
    {
      innerHTML: `
:root[data-collapsed~='${props.id}'] section[data-anchor-id='${props.id}'] .collapsible-content {
  grid-template-rows: 0fr;
}`,
    },
  ],
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("section", {
      id: __props.id,
      "data-anchor-id": __props.id,
      class: "scroll-mt-20 xl:scroll-mt-0"
    }, [ _createElementVNode("div", { class: "flex items-center justify-between mb-3 px-1" }, [ _createVNode(_resolveDynamicComponent(__props.headingLevel), {
          class: _normalizeClass(["group text-xs text-fg-subtle uppercase tracking-wider flex gap-2", __props.subtitle ? 'items-start' : 'items-center'])
        }, {
          default: _withCtx(() => [
            _createElementVNode("button", {
              id: _unref(buttonId),
              type: "button",
              class: "size-5 -me-1 flex items-center justify-center text-fg-subtle hover:text-fg-muted transition-colors duration-200 shrink-0 focus-visible:outline-accent/70 rounded",
              "aria-expanded": isOpen.value,
              "aria-controls": _unref(contentId),
              "aria-label": ariaLabel.value,
              onClick: toggle
            }, [
              (__props.isLoading)
                ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  class: "i-svg-spinners:ring-resize w-3 h-3",
                  "aria-hidden": "true"
                }))
                : (_openBlock(), _createElementBlock("span", {
                  key: 1,
                  class: _normalizeClass(["w-3 h-3 transition-transform duration-200", isOpen.value ? 'i-lucide:chevron-down' : 'i-lucide:chevron-right']),
                  "aria-hidden": "true"
                }))
            ], 8 /* PROPS */, ["id", "aria-expanded", "aria-controls", "aria-label"]),
            _createElementVNode("span", null, [
              _createVNode(LinkBase, { to: `#${__props.id}` }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(__props.title), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to"]),
              (__props.subtitle)
                ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  class: "block text-2xs normal-case tracking-normal"
                }, _toDisplayString(__props.subtitle), 1 /* TEXT */))
                : _createCommentVNode("v-if", true)
            ])
          ]),
          _: 1 /* STABLE */
        }, 2 /* CLASS */), _createTextVNode("\n\n      " + "\n      "), _createElementVNode("div", { class: "pe-1" }, [ _renderSlot(_ctx.$slots, "actions") ]) ]), _createElementVNode("div", {
        id: _unref(contentId),
        class: "grid ms-6 grid-rows-[1fr] transition-[grid-template-rows] duration-200 ease-in-out collapsible-content overflow-hidden",
        inert: !isOpen.value
      }, [ _createElementVNode("div", { class: "min-h-0 min-w-0" }, [ _renderSlot(_ctx.$slots, "default") ]) ], 8 /* PROPS */, ["id", "inert"]) ], 8 /* PROPS */, ["id", "data-anchor-id"]))
}
}

})
