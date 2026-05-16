import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveDynamicComponent as _resolveDynamicComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"


const _hoisted_1 = { class: "tabular-nums text-center opacity-50 px-2 text-xs select-none w-12 shrink-0" }
import type { DiffLine as DiffLineType } from '#shared/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'Line',
  props: {
    line: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const diffContext = inject<{
  fileStatus: ComputedRef<'add' | 'delete' | 'modify'>
  wordWrap?: ComputedRef<boolean>
}>('diffContext')
const lineNumberNew = computed(() => {
  if (props.line.type === 'normal') {
    return props.line.newLineNumber
  }
  return props.line.lineNumber ?? props.line.newLineNumber
})
const lineNumberOld = computed(() => {
  if (props.line.type === 'normal') {
    return props.line.oldLineNumber
  }
  return props.line.type === 'delete'
    ? (props.line.lineNumber ?? props.line.oldLineNumber)
    : undefined
})
const rowClasses = computed(() => {
  const shouldWrap = diffContext?.wordWrap?.value ?? false
  const classes = ['whitespace-pre-wrap', 'box-border', 'border-none']
  if (shouldWrap) classes.push('min-h-6')
  else classes.push('h-6', 'min-h-6')
  const fileStatus = diffContext?.fileStatus.value

  if (props.line.type === 'insert' && fileStatus !== 'add') {
    classes.push('bg-[var(--code-added)]/10')
  }
  if (props.line.type === 'delete' && fileStatus !== 'delete') {
    classes.push('bg-[var(--code-removed)]/10')
  }

  return classes
})
const borderClasses = computed(() => {
  const classes = ['border-transparent', 'w-1', 'border-is-3']

  if (props.line.type === 'insert') {
    classes.push('border-[color:var(--code-added)]/60')
  }
  if (props.line.type === 'delete') {
    classes.push('border-[color:var(--code-removed)]/80')
  }

  return classes
})
const contentClasses = computed(() => {
  const shouldWrap = diffContext?.wordWrap?.value ?? false
  return ['pe-6', shouldWrap ? 'whitespace-pre-wrap break-words' : 'text-nowrap']
})
function escapeHtml(str: string): string {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}
// Segments carry pre-highlighted HTML from the server API. Fall back to
// escaped plain text for unsupported languages.
const renderedSegments = computed(() =>
  props.line.content.map(seg => ({
    html: seg.html ?? escapeHtml(seg.value),
    type: seg.type,
  })),
)

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("tr", {
      "data-line-new": lineNumberNew.value,
      "data-line-old": lineNumberOld.value,
      "data-line-kind": __props.line.type,
      class: _normalizeClass(rowClasses.value)
    }, [ _createElementVNode("td", {
        class: _normalizeClass(borderClasses.value)
      }, null, 2 /* CLASS */), _createTextVNode("\n\n    " + "\n    "), _createElementVNode("td", _hoisted_1, _toDisplayString(__props.line.type === 'delete' ? '–' : lineNumberNew.value), 1 /* TEXT */), _createTextVNode("\n\n    " + "\n    "), _createElementVNode("td", {
        class: _normalizeClass(contentClasses.value)
      }, [ _createVNode(_resolveDynamicComponent(__props.line.type === 'insert' ? 'ins' : __props.line.type === 'delete' ? 'del' : 'span'), null, {
          default: _withCtx(() => [
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(renderedSegments.value, (seg, i) => {
              return (_openBlock(), _createElementBlock("span", {
                key: i,
                class: _normalizeClass({
              'bg-[var(--code-added)]/20': seg.type === 'insert',
              'bg-[var(--code-removed)]/20': seg.type === 'delete',
            }),
                innerHTML: seg.html
              }, 10 /* CLASS, PROPS */, ["innerHTML"]))
            }), 128 /* KEYED_FRAGMENT */))
          ]),
          _: 1 /* STABLE */
        }) ], 2 /* CLASS */) ], 10 /* CLASS, PROPS */, ["data-line-new", "data-line-old", "data-line-kind"]))
}
}

})
