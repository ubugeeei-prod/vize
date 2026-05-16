import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"

import { LinkBase, TagStatic } from '#components'

export default /*@__PURE__*/_defineComponent({
  __name: 'MetricsBadges',
  props: {
    packageName: { type: String, required: true },
    isBinary: { type: Boolean, required: false },
    version: { type: String, required: false }
  },
  setup(__props: any) {

const props = __props
const { data: analysis, status } = usePackageAnalysis(
  () => props.packageName,
  () => props.version,
)
const isLoading = computed(() => status.value !== 'error' && !analysis.value)
// ESM support
const hasEsm = computed(() => {
  if (!analysis.value) return false
  return analysis.value.moduleFormat === 'esm' || analysis.value.moduleFormat === 'dual'
})
// CJS support (only show badge if present, omit if missing)
const hasCjs = computed(() => {
  if (!analysis.value) return false
  return analysis.value.moduleFormat === 'cjs' || analysis.value.moduleFormat === 'dual'
})
// Types support
const hasTypes = computed(() => {
  if (!analysis.value) return false
  return analysis.value.types?.kind === 'included' || analysis.value.types?.kind === '@types'
})
const typesTooltip = computed(() => {
  if (!analysis.value) return ''
  switch (analysis.value.types?.kind) {
    case 'included':
      return $t('package.metrics.types_included')
    case '@types':
      return $t('package.metrics.types_available', { package: analysis.value.types.packageName })
    default:
      return $t('package.metrics.no_types')
  }
})
const typesHref = computed(() => {
  if (!analysis.value) return null
  if (analysis.value.types?.kind === '@types') {
    return `/package/${analysis.value.types.packageName}`
  }
  return null
})

return (_ctx: any,_cache: any) => {
  const _component_TooltipApp = _resolveComponent("TooltipApp")

  return (_openBlock(), _createElementBlock("ul", { class: "flex items-center gap-1.5 list-none m-0 p-0" }, [ (!props.isBinary) ? (_openBlock(), _createElementBlock("li", {
          key: 0,
          class: "contents"
        }, [ _createVNode(_component_TooltipApp, {
            text: typesTooltip.value,
            strategy: "fixed"
          }, {
            default: _withCtx(() => [
              (typesHref.value)
                ? (_openBlock(), _createBlock(LinkBase, {
                  key: 0,
                  variant: "button-secondary",
                  size: "small",
                  to: typesHref.value,
                  classicon: "i-lucide:check"
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_ctx.$t('package.metrics.types_label')), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["to"]))
                : (_openBlock(), _createBlock(TagStatic, {
                  key: 1,
                  variant: hasTypes.value && !isLoading.value ? 'default' : 'ghost',
                  tabindex: 0,
                  classicon: 
              isLoading.value ? 'i-svg-spinners:ring-resize ' : hasTypes.value ? 'i-lucide:check' : 'i-lucide:x'
          
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_ctx.$t('package.metrics.types_label')), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["variant", "tabindex", "classicon"]))
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["text"]) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    " + "\n    "), _createElementVNode("li", { class: "contents" }, [ _createVNode(_component_TooltipApp, {
          text: isLoading.value ? '' : hasEsm.value ? _ctx.$t('package.metrics.esm') : _ctx.$t('package.metrics.no_esm'),
          strategy: "fixed"
        }, {
          default: _withCtx(() => [
            _createVNode(TagStatic, {
              tabindex: "0",
              variant: hasEsm.value && !isLoading.value ? 'default' : 'ghost',
              classicon: 
              isLoading.value ? 'i-svg-spinners:ring-resize ' : hasEsm.value ? 'i-lucide:check' : 'i-lucide:x'
          
            }, {
              default: _withCtx(() => [
                _createTextVNode("\n          ESM\n        ")
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["variant", "classicon"])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["text"]) ]), _createTextVNode("\n\n    " + "\n    "), (isLoading.value || hasCjs.value) ? (_openBlock(), _createElementBlock("li", {
          key: 0,
          class: "contents"
        }, [ _createVNode(_component_TooltipApp, {
            text: isLoading.value ? '' : _ctx.$t('package.metrics.cjs'),
            strategy: "fixed"
          }, {
            default: _withCtx(() => [
              _createVNode(TagStatic, {
                tabindex: "0",
                variant: isLoading.value ? 'ghost' : 'default',
                classicon: isLoading.value ? 'i-svg-spinners:ring-resize ' : 'i-lucide:check'
              }, {
                default: _withCtx(() => [
                  _createTextVNode("\n          CJS\n        ")
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["variant", "classicon"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["text"]) ])) : _createCommentVNode("v-if", true) ]))
}
}

})
