import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withModifiers as _withModifiers } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'Viewer',
  props: {
    html: { type: String, required: true },
    lines: { type: Number, required: true },
    selectedLines: { type: Object, required: true }
  },
  emits: ["lineClick"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const codeRef = useTemplateRef('codeRef')
// Generate line numbers array
const lineNumbers = computed(() => {
  return Array.from({ length: props.lines }, (_, i) => i + 1)
})
// Used for CSS calculation of line number column width
const lineDigits = computed(() => {
  return String(props.lines).length
})
// Check if a line is selected
function isLineSelected(lineNum: number): boolean {
  if (!props.selectedLines) return false
  return lineNum >= props.selectedLines.start && lineNum <= props.selectedLines.end
}
// Handle line number click
function onLineClick(lineNum: number, event: MouseEvent) {
  emit('lineClick', lineNum, event)
}
// Apply highlighting to code lines when selection changes
function updateLineHighlighting() {
  if (!codeRef.value) return
  // Lines are inside pre > code > .line
  const lines = codeRef.value.querySelectorAll('code > .line')
  lines.forEach((line, index) => {
    const lineNum = index + 1
    if (isLineSelected(lineNum)) {
      line.classList.add('highlighted')
    } else {
      line.classList.remove('highlighted')
    }
  })
}
// Watch for changes to selection and HTML content
// Use deep watch and nextTick to ensure DOM is updated
watch(
  () => [props.selectedLines, props.html] as const,
  () => {
    nextTick(updateLineHighlighting)
  },
  { immediate: true },
)
// Use Nuxt's `navigateTo` for the rendered import links
function handleImportLinkNavigate() {
  if (!codeRef.value) return
  const anchors = codeRef.value.querySelectorAll<HTMLAnchorElement>('a.import-link')
  anchors.forEach(anchor => {
    // NOTE: We do not need to remove previous listeners because we re-create the entire HTML content on each html update
    anchor.addEventListener('click', event => {
      if (event.metaKey || event.altKey || event.ctrlKey || event.shiftKey) return
      const href = anchor.getAttribute('href')
      if (href) {
        event.preventDefault()
        navigateTo(href)
      }
    })
  })
}
watch(
  () => props.html,
  () => {
    nextTick(handleImportLinkNavigate)
  },
  { immediate: true },
)

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "code-viewer flex min-h-full max-w-full" }, [ _createElementVNode("div", {
        class: "line-numbers shrink-0 bg-bg-subtle border-ie border-solid border-border text-end select-none relative",
        style: _normalizeStyle({ '--line-digits': lineDigits.value }),
        "aria-hidden": "true"
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(lineNumbers.value, (lineNum) => {
          return (_openBlock(), _createElementBlock("a", {
            key: lineNum,
            id: `L${lineNum}`,
            href: `#L${lineNum}`,
            tabindex: "-1",
            class: _normalizeClass(["line-number block px-3 py-0 font-mono text-sm leading-6 cursor-pointer transition-colors no-underline", [
            isLineSelected(lineNum)
              ? 'bg-yellow-500/20 text-fg'
              : 'text-fg-subtle hover:text-fg-muted',
          ]]),
            onClick: _cache[0] || (_cache[0] = _withModifiers(($event: any) => (onLineClick(lineNum, $event)), ["prevent"]))
          }, _toDisplayString(lineNum), 11 /* TEXT, CLASS, PROPS */, ["id", "href"]))
        }), 128 /* KEYED_FRAGMENT */)) ], 4 /* STYLE */), _createTextVNode("\n\n    " + "\n    "), _createElementVNode("div", { class: "code-content flex-1 overflow-x-auto min-w-0" }, [ _createElementVNode("div", {
          ref_key: "codeRef", ref: codeRef,
          class: "code-lines w-fit",
          innerHTML: __props.html
        }, null, 8 /* PROPS */, ["innerHTML"]) ]) ]))
}
}

})
