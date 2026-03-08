import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Teleport as _Teleport, Transition as _Transition, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { onClickOutside, useEventListener } from '@vueuse/core'

export default /*@__PURE__*/_defineComponent({
  __name: 'ManagerSelect',
  setup(__props) {

const selectedPM = useSelectedPackageManager()
const listRef = useTemplateRef('listRef')
const triggerRef = useTemplateRef('triggerRef')
const isOpen = shallowRef(false)
const highlightedIndex = shallowRef(-1)
const dropdownPosition = shallowRef<{ top: number; left: number } | null>(null)
function getDropdownStyle(): Record<string, string> {
  if (!dropdownPosition.value) return {}
  return {
    top: `${dropdownPosition.value.top}px`,
    left: `${dropdownPosition.value.left}px`,
  }
}
useEventListener('scroll', close, true)
// Generate unique ID for accessibility
const inputId = useId()
const listboxId = `${inputId}-listbox`
function toggle() {
  if (isOpen.value) {
    close()
  } else {
    if (triggerRef.value) {
      const rect = triggerRef.value.getBoundingClientRect()
      dropdownPosition.value = {
        top: rect.bottom + 4,
        left: rect.left,
      }
    }
    isOpen.value = true
    highlightedIndex.value = packageManagers.findIndex(pm => pm.id === selectedPM.value)
  }
}
function close() {
  isOpen.value = false
  highlightedIndex.value = -1
}
function select(id: PackageManagerId) {
  selectedPM.value = id
  close()
  triggerRef.value?.focus()
}
// Check for reduced motion preference
const prefersReducedMotion = useMediaQuery('(prefers-reduced-motion: reduce)')
onClickOutside(listRef, close, { ignore: [triggerRef] })
function handleKeydown(event: KeyboardEvent) {
  if (!isOpen.value) return
  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      highlightedIndex.value = (highlightedIndex.value + 1) % packageManagers.length
      break
    case 'ArrowUp':
      event.preventDefault()
      highlightedIndex.value =
        highlightedIndex.value <= 0 ? packageManagers.length - 1 : highlightedIndex.value - 1
      break
    case 'Enter': {
      event.preventDefault()
      const pm = packageManagers[highlightedIndex.value]
      if (pm) {
        select(pm.id)
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
  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createElementVNode("button", {
        ref_key: "triggerRef", ref: triggerRef,
        type: "button",
        class: "cursor-pointer flex items-center gap-1.5 px-2 py-2 font-mono text-xs text-fg-muted bg-bg-subtle border border-border-subtle border-solid rounded-md transition-colors duration-150 hover:(text-fg border-border-hover) active:scale-95 focus:border-border-hover focus-visible:outline-accent/70",
        "aria-expanded": isOpen.value,
        "aria-haspopup": "listbox",
        "aria-label": _ctx.$t('package.get_started.pm_label'),
        "aria-controls": _unref(listboxId),
        onClick: toggle,
        onKeydown: handleKeydown
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.packageManagers, (pmOption) => {
          return (_openBlock(), _createElementBlock(_Fragment, { key: pmOption.id }, [
            _createElementVNode("span", {
              class: _normalizeClass(["inline-block h-3 w-3 pm-select-content", pmOption.icon]),
              "data-pm-select": pmOption.id,
              "aria-hidden": "true"
            }, null, 10 /* CLASS, PROPS */, ["data-pm-select"]),
            _createElementVNode("span", {
              class: "pm-select-content",
              "data-pm-select": pmOption.id,
              "aria-hidden": pmOption.id !== _unref(selectedPM)
            }, _toDisplayString(pmOption.label), 9 /* TEXT, PROPS */, ["data-pm-select", "aria-hidden"])
          ], 64 /* STABLE_FRAGMENT */))
        }), 128 /* KEYED_FRAGMENT */)), _createElementVNode("span", {
          class: _normalizeClass(["i-lucide:chevron-down w-3 h-3", [
          { 'rotate-180': isOpen.value },
          _unref(prefersReducedMotion) ? '' : 'transition-transform duration-200',
        ]]),
          "aria-hidden": "true"
        }, null, 2 /* CLASS */) ], 40 /* PROPS, NEED_HYDRATION */, ["aria-expanded", "aria-label", "aria-controls"]), _createVNode(_Teleport, { to: "body" }, [ _createVNode(_Transition, {
          "enter-active-class": _unref(prefersReducedMotion) ? '' : 'transition-opacity duration-150',
          "enter-from-class": _unref(prefersReducedMotion) ? '' : 'opacity-0',
          "enter-to-class": "opacity-100",
          "leave-active-class": _unref(prefersReducedMotion) ? '' : 'transition-opacity duration-100',
          "leave-from-class": "opacity-100",
          "leave-to-class": _unref(prefersReducedMotion) ? '' : 'opacity-0'
        }, {
          default: _withCtx(() => [
            (isOpen.value)
              ? (_openBlock(), _createElementBlock("ul", {
                key: 0,
                id: _unref(listboxId),
                ref: "listRef",
                role: "listbox",
                "aria-activedescendant": 
            highlightedIndex.value >= 0
              ? `${_unref(listboxId)}-${_ctx.packageManagers[highlightedIndex.value]?.id}`
              : undefined
          ,
                "aria-label": _ctx.$t('package.get_started.pm_label'),
                style: getDropdownStyle(),
                class: "fixed bg-bg-subtle border border-border rounded-md shadow-lg z-50"
              }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.packageManagers, (pm, index) => {
                  return (_openBlock(), _createElementBlock("li", {
                    key: pm.id,
                    id: `${_unref(listboxId)}-${pm.id}`,
                    role: "option",
                    "aria-selected": _unref(selectedPM) === pm.id,
                    class: _normalizeClass(["cursor-pointer flex items-center gap-2 px-3 py-1.5 font-mono text-xs transition-colors duration-150", [
              _unref(selectedPM) === pm.id ? 'text-fg' : 'text-fg-subtle',
              highlightedIndex.value === index ? 'bg-bg-elevated' : 'hover:bg-bg-elevated',
            ]]),
                    onClick: _cache[0] || (_cache[0] = ($event: any) => (select(pm.id))),
                    onMouseenter: _cache[1] || (_cache[1] = ($event: any) => (highlightedIndex.value = index))
                  }, [
                    _createElementVNode("span", {
                      class: _normalizeClass(["inline-block h-3 w-3", pm.icon]),
                      "aria-hidden": "true"
                    }, null, 2 /* CLASS */),
                    _createElementVNode("span", null, _toDisplayString(pm.label), 1 /* TEXT */),
                    (_unref(selectedPM) === pm.id)
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: "i-lucide:check w-3 h-3 text-accent ms-auto",
                        "aria-hidden": "true"
                      }))
                      : _createCommentVNode("v-if", true)
                  ], 42 /* CLASS, PROPS, NEED_HYDRATION */, ["id", "aria-selected"]))
                }), 128 /* KEYED_FRAGMENT */))
              ]))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["enter-active-class", "enter-from-class", "leave-active-class", "leave-to-class"]) ]) ], 64 /* STABLE_FRAGMENT */))
}
}

})
