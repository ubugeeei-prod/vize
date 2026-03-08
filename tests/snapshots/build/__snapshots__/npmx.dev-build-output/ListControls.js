import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString } from "vue"


const _hoisted_1 = { for: "package-filter", class: "sr-only" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { class: "i-lucide:search w-4 h-4" })

export type SortOption = 'downloads' | 'updated' | 'name-asc' | 'name-desc'

export default /*@__PURE__*/_defineComponent({
  __name: 'ListControls',
  props: {
    filter: { type: String, required: true },
    sort: { type: String, required: true },
    placeholder: { type: String, required: false },
    totalCount: { type: Number, required: false },
    filteredCount: { type: Number, required: false }
  },
  emits: ["update", "update"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const filterValue = computed({
  get: () => props.filter,
  set: value => emit('update:filter', value),
})
const sortValue = computed({
  get: () => props.sort,
  set: value => emit('update:sort', value),
})
const sortOptions = computed(
  () =>
    [
      { value: 'downloads', label: $t('package.sort.downloads') },
      { value: 'updated', label: $t('package.sort.published') },
      { value: 'name-asc', label: $t('package.sort.name_asc') },
      { value: 'name-desc', label: $t('package.sort.name_desc') },
    ] as const,
)
// Show filter count when filtering is active
const showFilteredCount = computed(() => {
  return (
    props.filter &&
    props.filteredCount !== undefined &&
    props.totalCount !== undefined &&
    props.filteredCount !== props.totalCount
  )
})

return (_ctx: any,_cache: any) => {
  const _component_InputBase = _resolveComponent("InputBase")
  const _component_SelectField = _resolveComponent("SelectField")

  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createElementVNode("div", { class: "flex flex-col sm:flex-row gap-3 mb-6" }, [ _createElementVNode("div", { class: "flex-1 relative" }, [ _createElementVNode("label", _hoisted_1, _toDisplayString(_ctx.$t('package.list.filter_label')), 1 /* TEXT */), _createElementVNode("div", {
            class: "absolute h-full w-10 flex items-center justify-center text-fg-subtle pointer-events-none",
            "aria-hidden": "true"
          }, [ _hoisted_2 ]), _createVNode(_component_InputBase, {
            id: "package-filter",
            type: "search",
            placeholder: __props.placeholder ?? _ctx.$t('package.list.filter_placeholder'),
            "no-correct": "",
            class: "w-full min-w-25 ps-10",
            size: "medium",
            modelValue: filterValue.value,
            "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((filterValue).value = $event))
          }, null, 8 /* PROPS */, ["placeholder", "modelValue"]) ]), _createTextVNode("\n\n    " + "\n    "), _createVNode(_component_SelectField, {
          label: _ctx.$t('package.list.sort_label'),
          "hidden-label": "",
          id: "package-sort",
          class: "relative shrink-0",
          items: sortOptions.value.map((option) => ({
  	label: option.label,
  	value: option.value
  })),
          modelValue: sortValue.value,
          "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((sortValue).value = $event))
        }, null, 8 /* PROPS */, ["label", "items", "modelValue"]) ]), (showFilteredCount.value) ? (_openBlock(), _createElementBlock("p", {
          key: 0,
          class: "text-fg-subtle text-xs font-mono mb-4"
        }, _toDisplayString(_ctx.$t('package.list.showing_count', { filtered: __props.filteredCount, total: __props.totalCount })), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */))
}
}

})
