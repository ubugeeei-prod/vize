import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Teleport as _Teleport, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "truncate" }
const _hoisted_2 = { class: "truncate" }
const _hoisted_3 = { class: "truncate" }
import type { TocItem } from '#shared/types/readme'
import { onClickOutside, useEventListener } from '@vueuse/core'

interface TocNode extends TocItem {
  children: TocNode[]
}

export default /*@__PURE__*/_defineComponent({
  __name: 'ReadmeTocDropdown',
  props: {
    toc: { type: Array, required: true },
    activeId: { type: String, required: false }
  },
  setup(__props: any) {

const props = __props
function buildTocTree(items: TocItem[]): TocNode[] {
  const result: TocNode[] = []
  const stack: TocNode[] = []
  for (const item of items) {
    const node: TocNode = { ...item, children: [] }
    // Find parent: look for the last item with smaller depth
    while (stack.length > 0 && stack[stack.length - 1]!.depth >= item.depth) {
      stack.pop()
    }
    if (stack.length === 0) {
      result.push(node)
    } else {
      stack[stack.length - 1]!.children.push(node)
    }
    stack.push(node)
  }
  return result
}
const tocTree = computed(() => buildTocTree(props.toc))
// Create a map from id to index for efficient lookup
const idToIndex = computed(() => {
  const map = new Map<string, number>()
  props.toc.forEach((item, index) => map.set(item.id, index))
  return map
})
const listRef = useTemplateRef('listRef')
const triggerRef = useTemplateRef('triggerRef')
const isOpen = shallowRef(false)
const highlightedIndex = shallowRef(-1)
const dropdownPosition = shallowRef<{ top: number; right: number } | null>(null)
function getDropdownStyle(): Record<string, string> {
  if (!dropdownPosition.value) return {}
  return {
    top: `${dropdownPosition.value.top}px`,
    right: `${document.documentElement.clientWidth - dropdownPosition.value.right}px`,
  }
}
// Close on scroll (but not when scrolling inside the dropdown)
function handleScroll(event: Event) {
  if (!isOpen.value) return
  if (listRef.value && event.target instanceof Node && listRef.value.contains(event.target)) {
    return
  }
  close()
}
useEventListener('scroll', handleScroll, { passive: true })
// Generate unique ID for accessibility
const inputId = useId()
const listboxId = `${inputId}-toc-listbox`
function toggle() {
  if (isOpen.value) {
    close()
  } else {
    const rect = triggerRef.value?.getBoundingClientRect()
    if (rect) {
      dropdownPosition.value = {
        top: rect.bottom + 4,
        right: rect.right,
      }
    }
    isOpen.value = true
    // Highlight active item if any
    const activeIndex = idToIndex.value.get(props.activeId ?? '')
    highlightedIndex.value = activeIndex ?? 0
  }
}
function close() {
  isOpen.value = false
  highlightedIndex.value = -1
}
function select() {
  close()
  triggerRef.value?.focus()
}
function getIndex(id: string): number {
  return idToIndex.value.get(id) ?? -1
}
// Check for reduced motion preference
const prefersReducedMotion = useMediaQuery('(prefers-reduced-motion: reduce)')
onClickOutside(listRef, close, { ignore: [triggerRef] })
function handleKeydown(event: KeyboardEvent) {
  if (!isOpen.value) return
  const itemCount = props.toc.length
  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      highlightedIndex.value = (highlightedIndex.value + 1) % itemCount
      break
    case 'ArrowUp':
      event.preventDefault()
      highlightedIndex.value =
        highlightedIndex.value <= 0 ? itemCount - 1 : highlightedIndex.value - 1
      break
    case 'Enter': {
      event.preventDefault()
      const item = props.toc[highlightedIndex.value]
      if (item) {
        select()
      }
      break
    }
    case 'Escape':
      close()
      triggerRef.value?.focus()
      break
  }
}

return (_ctx: any,_cache: any) => {
  const _component_ButtonBase = _resolveComponent("ButtonBase")
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createVNode(_component_ButtonBase, {
        ref_key: "triggerRef", ref: triggerRef,
        type: "button",
        "aria-expanded": isOpen.value,
        "aria-haspopup": "listbox",
        "aria-label": _ctx.$t('package.readme.toc_title'),
        "aria-controls": _unref(listboxId),
        onClick: toggle,
        onKeydown: handleKeydown,
        classicon: "i-lucide:list",
        class: "px-2.5",
        block: ""
      }, {
        default: _withCtx(() => [
          _createElementVNode("span", {
            class: _normalizeClass(["i-lucide:chevron-down w-3 h-3", [
          { 'rotate-180': isOpen.value },
          _unref(prefersReducedMotion) ? '' : 'transition-transform duration-200',
        ]]),
            "aria-hidden": "true"
          }, null, 2 /* CLASS */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["aria-expanded", "aria-label", "aria-controls"]), _createVNode(_Teleport, { to: "body" }, [ _createVNode(_Transition, {
          "enter-active-class": _unref(prefersReducedMotion) ? '' : 'transition-opacity duration-150',
          "enter-from-class": _unref(prefersReducedMotion) ? '' : 'opacity-0',
          "enter-to-class": "opacity-100",
          "leave-active-class": _unref(prefersReducedMotion) ? '' : 'transition-opacity duration-100',
          "leave-from-class": "opacity-100",
          "leave-to-class": _unref(prefersReducedMotion) ? '' : 'opacity-0'
        }, {
          default: _withCtx(() => [
            (isOpen.value)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                id: _unref(listboxId),
                ref: "listRef",
                role: "listbox",
                "aria-activedescendant": 
            highlightedIndex.value >= 0 ? `${_unref(listboxId)}-${__props.toc[highlightedIndex.value]?.id}` : undefined
          ,
                "aria-label": _ctx.$t('package.readme.toc_title'),
                style: getDropdownStyle(),
                class: "fixed bg-bg-subtle border border-border rounded-md shadow-lg z-50 max-h-80 overflow-y-auto w-56 overscroll-contain"
              }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(tocTree.value, (node) => {
                  return (_openBlock(), _createElementBlock(_Fragment, { key: node.id }, [
                    _createVNode(_component_NuxtLink, {
                      id: `${_unref(listboxId)}-${node.id}`,
                      to: `#${node.id}`,
                      role: "option",
                      "aria-selected": __props.activeId === node.id,
                      class: _normalizeClass(["flex items-center gap-2 px-3 py-1.5 text-sm cursor-pointer transition-colors duration-150", [
                __props.activeId === node.id ? 'text-fg font-medium' : 'text-fg-muted',
                highlightedIndex.value === getIndex(node.id) ? 'bg-bg-elevated' : 'hover:bg-bg-elevated',
              ]]),
                      dir: "auto",
                      onClick: _cache[0] || (_cache[0] = ($event: any) => (select())),
                      onMouseenter: _cache[1] || (_cache[1] = ($event: any) => (highlightedIndex.value = getIndex(node.id)))
                    }, {
                      default: _withCtx(() => [
                        _createElementVNode("span", _hoisted_1, _toDisplayString(node.text), 1 /* TEXT */)
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 10 /* CLASS, PROPS */, ["id", "to", "aria-selected"]),
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(node.children, (child) => {
                      return (_openBlock(), _createElementBlock(_Fragment, { key: child.id }, [
                        _createVNode(_component_NuxtLink, {
                          id: `${_unref(listboxId)}-${child.id}`,
                          to: `#${child.id}`,
                          role: "option",
                          "aria-selected": __props.activeId === child.id,
                          class: _normalizeClass(["flex items-center gap-2 px-3 py-1.5 ps-6 text-sm cursor-pointer transition-colors duration-150", [
                  __props.activeId === child.id ? 'text-fg font-medium' : 'text-fg-subtle',
                  highlightedIndex.value === getIndex(child.id) ? 'bg-bg-elevated' : 'hover:bg-bg-elevated',
                ]]),
                          dir: "auto",
                          onClick: _cache[2] || (_cache[2] = ($event: any) => (select())),
                          onMouseenter: _cache[3] || (_cache[3] = ($event: any) => (highlightedIndex.value = getIndex(child.id)))
                        }, {
                          default: _withCtx(() => [
                            _createElementVNode("span", _hoisted_2, _toDisplayString(child.text), 1 /* TEXT */)
                          ]),
                          _: 2 /* DYNAMIC */
                        }, 10 /* CLASS, PROPS */, ["id", "to", "aria-selected"]),
                        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(child.children, (grandchild) => {
                          return (_openBlock(), _createBlock(_component_NuxtLink, {
                            key: grandchild.id,
                            id: `${_unref(listboxId)}-${grandchild.id}`,
                            to: `#${grandchild.id}`,
                            role: "option",
                            "aria-selected": __props.activeId === grandchild.id,
                            class: _normalizeClass(["flex items-center gap-2 px-3 py-1.5 ps-9 text-sm cursor-pointer transition-colors duration-150", [
                  __props.activeId === grandchild.id ? 'text-fg font-medium' : 'text-fg-subtle',
                  highlightedIndex.value === getIndex(grandchild.id)
                    ? 'bg-bg-elevated'
                    : 'hover:bg-bg-elevated',
                ]]),
                            dir: "auto",
                            onClick: _cache[4] || (_cache[4] = ($event: any) => (select())),
                            onMouseenter: _cache[5] || (_cache[5] = ($event: any) => (highlightedIndex.value = getIndex(grandchild.id)))
                          }, {
                            default: _withCtx(() => [
                              _createElementVNode("span", _hoisted_3, _toDisplayString(grandchild.text), 1 /* TEXT */)
                            ]),
                            _: 2 /* DYNAMIC */
                          }, 1034 /* CLASS, PROPS, DYNAMIC_SLOTS */, ["id", "to", "aria-selected"]))
                        }), 128 /* KEYED_FRAGMENT */))
                      ], 64 /* STABLE_FRAGMENT */))
                    }), 128 /* KEYED_FRAGMENT */))
                  ], 64 /* STABLE_FRAGMENT */))
                }), 128 /* KEYED_FRAGMENT */))
              ]))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["enter-active-class", "enter-from-class", "leave-active-class", "leave-to-class"]) ]) ], 64 /* STABLE_FRAGMENT */))
}
}

})
