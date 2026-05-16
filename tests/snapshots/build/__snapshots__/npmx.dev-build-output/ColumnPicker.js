import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = { class: "px-3 py-2 text-xs font-mono text-fg-subtle uppercase tracking-wider border-b border-border", "aria-hidden": "true" }
const _hoisted_2 = { class: "text-sm text-fg-muted font-mono flex-1" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "size-4 flex justify-center items-center text-xs border rounded-full" }, "i")
import type { ColumnConfig, ColumnId } from '#shared/types/preferences'
import { onKeyDown } from '@vueuse/core'

export default /*@__PURE__*/_defineComponent({
  __name: 'ColumnPicker',
  props: {
    columns: { type: Array, required: true }
  },
  emits: ["toggle", "reset"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const isOpen = shallowRef(false)
const buttonRef = useTemplateRef('buttonRef')
const menuRef = useTemplateRef('menuRef')
const menuId = useId()
// Close on click outside (check both button and menu)
onClickOutside(
  menuRef,
  () => {
    isOpen.value = false
  },
  {
    ignore: [buttonRef],
  },
)
onKeyDown(
  'Escape',
  e => {
    if (!isOpen.value) return
    isOpen.value = false
    buttonRef.value?.focus()
  },
  { dedupe: true },
)
// Columns that can be toggled (name is always visible)
const toggleableColumns = computed(() => props.columns.filter(col => col.id !== 'name'))
// Map column IDs to i18n keys
const columnLabels = computed(() => ({
  name: $t('filters.columns.name'),
  version: $t('filters.columns.version'),
  description: $t('filters.columns.description'),
  downloads: $t('filters.columns.downloads'),
  updated: $t('filters.columns.published'),
  maintainers: $t('filters.columns.maintainers'),
  keywords: $t('filters.columns.keywords'),
  qualityScore: $t('filters.columns.quality_score'),
  popularityScore: $t('filters.columns.popularity_score'),
  maintenanceScore: $t('filters.columns.maintenance_score'),
  combinedScore: $t('filters.columns.combined_score'),
  security: $t('filters.columns.security'),
}))
function getColumnLabel(id: ColumnId): string {
  const key = columnLabels.value[id]
  return key ?? id
}
function handleReset() {
  emit('reset')
  isOpen.value = false
}

return (_ctx: any,_cache: any) => {
  const _component_ButtonBase = _resolveComponent("ButtonBase")
  const _component_TooltipApp = _resolveComponent("TooltipApp")

  return (_openBlock(), _createElementBlock("div", { class: "relative" }, [ _createVNode(_component_ButtonBase, {
        ref_key: "buttonRef", ref: buttonRef,
        "aria-expanded": isOpen.value,
        "aria-haspopup": "true",
        "aria-controls": _unref(menuId),
        onClick: _cache[0] || (_cache[0] = _withModifiers(($event: any) => (isOpen.value = !isOpen.value), ["stop"])),
        classicon: "i-lucide:columns-3-cog"
      }, {
        default: _withCtx(() => [
          _createTextVNode(_toDisplayString(_ctx.$t('filters.columns.title')), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["aria-expanded", "aria-controls"]), _createVNode(_Transition, { name: "dropdown" }, {
        default: _withCtx(() => [
          (isOpen.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              ref: "menuRef",
              id: _unref(menuId),
              class: "absolute top-full inset-is-auto sm:inset-ie-0 mt-2 w-60 bg-bg-subtle border border-border rounded-lg shadow-lg z-20",
              role: "group",
              "aria-label": _ctx.$t('filters.columns.show')
            }, [
              _createElementVNode("div", { class: "py-1" }, [
                _createElementVNode("div", _hoisted_1, _toDisplayString(_ctx.$t('filters.columns.show')), 1 /* TEXT */),
                _createElementVNode("div", { class: "py-1 max-h-64 overflow-y-auto" }, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(toggleableColumns.value, (column) => {
                    return (_openBlock(), _createElementBlock("label", {
                      key: column.id,
                      class: _normalizeClass(["flex gap-2 items-center px-3 py-2 transition-colors duration-200", column.disabled ? 'opacity-50 cursor-not-allowed' : 'hover:bg-bg-muted'])
                    }, [
                      _createElementVNode("input", {
                        type: "checkbox",
                        checked: column.visible,
                        disabled: column.disabled,
                        "aria-describedby": column.disabled ? `${column.id}-disabled-reason` : undefined,
                        class: "w-4 h-4 accent-fg bg-bg-muted border-border rounded disabled:opacity-50",
                        onChange: _cache[1] || (_cache[1] = ($event: any) => (!column.disabled && emit('toggle', column.id)))
                      }, null, 40 /* PROPS, NEED_HYDRATION */, ["checked", "disabled", "aria-describedby"]),
                      _createElementVNode("span", _hoisted_2, _toDisplayString(getColumnLabel(column.id)), 1 /* TEXT */),
                      (column.disabled)
                        ? (_openBlock(), _createBlock(_component_TooltipApp, {
                          key: 0,
                          id: `${column.id}-disabled-reason`,
                          class: "text-fg-subtle",
                          text: _ctx.$t('filters.columns.coming_soon'),
                          position: "left"
                        }, {
                          default: _withCtx(() => [
                            _hoisted_3
                          ]),
                          _: 2 /* DYNAMIC */
                        }, 8 /* PROPS */, ["id", "text"]))
                        : _createCommentVNode("v-if", true)
                    ], 2 /* CLASS */))
                  }), 128 /* KEYED_FRAGMENT */))
                ]),
                _createElementVNode("div", { class: "border-t border-border py-1" }, [
                  _createVNode(_component_ButtonBase, { onClick: handleReset }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_ctx.$t('filters.columns.reset')), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ])
              ])
            ]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }) ]))
}
}

})
