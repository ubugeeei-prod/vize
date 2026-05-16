import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, mergeProps as _mergeProps, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vModelText as _vModelText } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:info w-3 h-3 me-1 align-middle", "aria-hidden": "true" })
const _hoisted_2 = { class: "text-amber-400" }

export default /*@__PURE__*/_defineComponent({
  __name: 'UserCombobox',
  props: {
    suggestions: { type: Array, required: true },
    placeholder: { type: String, required: false },
    disabled: { type: Boolean, required: false },
    label: { type: String, required: false }
  },
  emits: ["select"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const inputValue = shallowRef('')
const isOpen = shallowRef(false)
const highlightedIndex = shallowRef(-1)
const listRef = useTemplateRef('listRef')
// Generate unique ID for accessibility
const inputId = useId()
const listboxId = `${inputId}-listbox`
// Filter suggestions based on input
const filteredSuggestions = computed(() => {
  if (!inputValue.value.trim()) {
    return props.suggestions.slice(0, 10) // Show first 10 when empty
  }
  const query = inputValue.value.toLowerCase().replace(/^@/, '')
  return props.suggestions.filter(s => s.toLowerCase().includes(query)).slice(0, 10)
})
// Check if current input matches a suggestion exactly
const isExactMatch = computed(() => {
  const normalized = inputValue.value.trim().replace(/^@/, '').toLowerCase()
  return props.suggestions.some(s => s.toLowerCase() === normalized)
})
// Show hint when typing a non-member username
const showNewUserHint = computed(() => {
  const value = inputValue.value.trim().replace(/^@/, '')
  return value.length > 0 && !isExactMatch.value && filteredSuggestions.value.length === 0
})
function handleInput() {
  isOpen.value = true
  highlightedIndex.value = -1
}
function handleFocus() {
  isOpen.value = true
}
function handleBlur(event: FocusEvent) {
  // Don't close if clicking within the dropdown
  const relatedTarget = event.relatedTarget as HTMLElement | null
  if (relatedTarget && listRef.value?.contains(relatedTarget)) {
    return
  }
  // Delay to allow click to register
  setTimeout(() => {
    isOpen.value = false
    highlightedIndex.value = -1
  }, 150)
}
function selectSuggestion(username: string) {
  inputValue.value = username
  isOpen.value = false
  highlightedIndex.value = -1
  emit('select', username, true)
  inputValue.value = ''
}
function handleSubmit() {
  const username = inputValue.value.trim().replace(/^@/, '')
  if (!username) return
  const inSuggestions = props.suggestions.some(s => s.toLowerCase() === username.toLowerCase())
  emit('select', username, inSuggestions)
  inputValue.value = ''
  isOpen.value = false
}
function handleKeydown(event: KeyboardEvent) {
  if (!isOpen.value) {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      isOpen.value = true
      event.preventDefault()
    }
    return
  }
  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      if (highlightedIndex.value < filteredSuggestions.value.length - 1) {
        highlightedIndex.value++
      }
      break
    case 'ArrowUp':
      event.preventDefault()
      if (highlightedIndex.value > 0) {
        highlightedIndex.value--
      }
      break
    case 'Enter': {
      event.preventDefault()
      const selectedSuggestion = filteredSuggestions.value[highlightedIndex.value]
      if (highlightedIndex.value >= 0 && selectedSuggestion) {
        selectSuggestion(selectedSuggestion)
      } else {
        handleSubmit()
      }
      break
    }
    case 'Escape':
      isOpen.value = false
      highlightedIndex.value = -1
      break
  }
}
// Scroll highlighted item into view
watch(highlightedIndex, index => {
  if (index >= 0 && listRef.value) {
    const item = listRef.value.children[index] as HTMLElement
    item?.scrollIntoView({ block: 'nearest' })
  }
})
// Check for reduced motion preference
const prefersReducedMotion = useMediaQuery('(prefers-reduced-motion: reduce)')

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "relative" }, [ (__props.label) ? (_openBlock(), _createElementBlock("label", {
          key: 0,
          for: _unref(inputId),
          class: "sr-only"
        }, _toDisplayString(__props.label), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _withDirectives(_createElementVNode("input", _mergeProps(_ctx.noCorrect, {
        id: _unref(inputId),
        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((inputValue).value = $event)),
        type: "text",
        placeholder: __props.placeholder ?? _ctx.$t('user.combobox.default_placeholder'),
        disabled: __props.disabled,
        role: "combobox",
        "aria-autocomplete": "list",
        "aria-expanded": isOpen.value && (filteredSuggestions.value.length > 0 || showNewUserHint.value),
        "aria-haspopup": "listbox",
        "aria-controls": _unref(listboxId),
        "aria-activedescendant": 
          highlightedIndex.value >= 0 ? `${_unref(listboxId)}-option-${highlightedIndex.value}` : undefined
        ,
        class: "w-full px-2 py-1 font-mono text-sm bg-bg-subtle border border-border rounded text-fg placeholder:text-fg-subtle transition-colors duration-200 focus:border-border-hover focus-visible:outline-accent/70 disabled:opacity-50 disabled:cursor-not-allowed",
        onInput: handleInput,
        onFocus: handleFocus,
        onBlur: handleBlur,
        onKeydown: handleKeydown
      }), null, 48 /* FULL_PROPS, NEED_HYDRATION */, ["id", "placeholder", "disabled", "aria-expanded", "aria-controls", "aria-activedescendant"]), [ [_vModelText, inputValue.value] ]), _createTextVNode("\n\n    " + "\n    "), _createVNode(_Transition, {
        "enter-active-class": _unref(prefersReducedMotion) ? '' : 'transition-opacity duration-150',
        "enter-from-class": _unref(prefersReducedMotion) ? '' : 'opacity-0',
        "enter-to-class": "opacity-100",
        "leave-active-class": _unref(prefersReducedMotion) ? '' : 'transition-opacity duration-100',
        "leave-from-class": "opacity-100",
        "leave-to-class": _unref(prefersReducedMotion) ? '' : 'opacity-0'
      }, {
        default: _withCtx(() => [
          (isOpen.value && (filteredSuggestions.value.length > 0 || showNewUserHint.value))
            ? (_openBlock(), _createElementBlock("ul", {
              key: 0,
              id: _unref(listboxId),
              ref: "listRef",
              role: "listbox",
              "aria-label": __props.label ?? _ctx.$t('user.combobox.suggestions_label'),
              class: "absolute z-50 w-full mt-1 py-1 bg-bg-elevated border border-border rounded shadow-lg max-h-48 overflow-y-auto"
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(filteredSuggestions.value, (username, index) => {
                return (_openBlock(), _createElementBlock("li", {
                  key: username,
                  id: `${_unref(listboxId)}-option-${index}`,
                  role: "option",
                  "aria-selected": highlightedIndex.value === index,
                  class: _normalizeClass(["px-2 py-1 font-mono text-sm transition-colors duration-100", 
              highlightedIndex.value === index
                ? 'bg-bg-muted text-fg'
                : 'text-fg-muted hover:bg-bg-subtle hover:text-fg'
            ]),
                  onMouseenter: _cache[1] || (_cache[1] = ($event: any) => (highlightedIndex.value = index)),
                  onClick: _cache[2] || (_cache[2] = ($event: any) => (selectSuggestion(username)))
                }, "\n          @" + _toDisplayString(username), 43 /* TEXT, CLASS, PROPS, NEED_HYDRATION */, ["id", "aria-selected"]))
              }), 128 /* KEYED_FRAGMENT */)),
              _createTextVNode("\n\n        "),
              _createTextVNode("\n        "),
              (showNewUserHint.value)
                ? (_openBlock(), _createElementBlock("li", {
                  key: 0,
                  class: "px-2 py-1 font-mono text-xs text-fg-subtle border-t border-border mt-1 pt-2",
                  role: "status",
                  "aria-live": "polite"
                }, [
                  _hoisted_1,
                  _createTextVNode("\n          "),
                  _toDisplayString(_ctx.$t('user.combobox.press_enter_to_add', {
                username: inputValue.value.trim().replace(/^@/, ''),
              })),
                  _createTextVNode("\n          "),
                  _createElementVNode("span", _hoisted_2, _toDisplayString(_ctx.$t('user.combobox.add_to_org_hint')), 1 /* TEXT */)
                ]))
                : _createCommentVNode("v-if", true)
            ]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["enter-active-class", "enter-from-class", "leave-active-class", "leave-to-class"]) ]))
}
}

})
