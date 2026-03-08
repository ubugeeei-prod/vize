import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:network w-3 h-3", "aria-hidden": "true" })
import { onKeyDown } from '@vueuse/core'
import { UseFocusTrap } from '@vueuse/integrations/useFocusTrap/component'

export default /*@__PURE__*/_defineComponent({
  __name: 'DependencyPathPopup',
  props: {
    path: { type: Array, required: true }
  },
  setup(__props: any) {

const isOpen = shallowRef(false)
const popupEl = useTemplateRef('popupEl')
const popupPosition = shallowRef<{ top: number; left: number } | null>(null)
function closePopup() {
  isOpen.value = false
}
// Close popup on click outside
onClickOutside(popupEl, () => {
  if (isOpen.value) closePopup()
})
onKeyDown(
  'Escape',
  e => {
    e.preventDefault()
    closePopup()
  },
  { dedupe: true, target: popupEl },
)
useEventListener('scroll', closePopup, { passive: true })
function togglePopup(event: MouseEvent) {
  if (isOpen.value) {
    closePopup()
  } else {
    const button = event.currentTarget as HTMLElement
    const rect = button.getBoundingClientRect()
    popupPosition.value = {
      top: rect.bottom + 4,
      left: rect.left,
    }
    isOpen.value = true
  }
}
function getPopupStyle(): Record<string, string> {
  if (!popupPosition.value) return {}
  return {
    top: `${popupPosition.value.top}px`,
    left: `${popupPosition.value.left}px`,
  }
}
// Parse package string "name@version" into { name, version }
function parsePackageString(pkg: string): { name: string; version: string } {
  const atIndex = pkg.lastIndexOf('@')
  if (atIndex > 0) {
    return { name: pkg.slice(0, atIndex), version: pkg.slice(atIndex + 1) }
  }
  return { name: pkg, version: '' }
}

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (_openBlock(), _createElementBlock("div", { class: "relative" }, [ _createElementVNode("button", {
        type: "button",
        class: "path-badge font-mono text-3xs px-1.5 py-0.5 rounded bg-amber-500/10 border border-amber-500/30 text-amber-800 dark:text-amber-400 transition-all duration-200 ease-out whitespace-nowrap flex items-center gap-1 hover:bg-amber-500/20 hover:border-amber-500/50",
        "aria-expanded": isOpen.value,
        onClick: _withModifiers(togglePopup, ["stop"])
      }, [ _hoisted_1, _createElementVNode("span", null, _toDisplayString(_ctx.$t('package.vulnerabilities.path')), 1 /* TEXT */) ], 8 /* PROPS */, ["aria-expanded"]), _createTextVNode("\n\n    " + "\n    "), (isOpen.value) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          ref: "popupEl",
          class: "fixed z-[100] bg-bg-elevated border border-border rounded-lg shadow-xl p-3 min-w-64 max-w-sm",
          style: getPopupStyle()
        }, [ _createVNode(UseFocusTrap, { options: { immediate: true } }, {
            default: _withCtx(() => [
              _createElementVNode("ul", { class: "list-none m-0 p-0 space-y-0.5" }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.path, (pathItem, idx) => {
                  return (_openBlock(), _createElementBlock("li", {
                    key: idx,
                    class: "font-mono text-xs",
                    style: _normalizeStyle({ paddingLeft: `${idx * 12}px` })
                  }, [
                    (idx > 0)
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: "text-fg-subtle me-1"
                      }, "└─"))
                      : _createCommentVNode("v-if", true),
                    _createVNode(_component_NuxtLink, {
                      to: 
                  _ctx.packageRoute(
                    parsePackageString(pathItem).name,
                    parsePackageString(pathItem).version,
                  )
                ,
                      class: _normalizeClass(["hover:underline", idx === __props.path.length - 1 ? 'text-fg font-medium' : 'text-fg-muted']),
                      onClick: closePopup
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(pathItem), 1 /* TEXT */)
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 10 /* CLASS, PROPS */, ["to"]),
                    (idx === __props.path.length - 1)
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: "ms-1 text-amber-500"
                      }, "⚠"))
                      : _createCommentVNode("v-if", true)
                  ], 4 /* STYLE */))
                }), 128 /* KEYED_FRAGMENT */))
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["options"]) ])) : _createCommentVNode("v-if", true) ]))
}
}

})
