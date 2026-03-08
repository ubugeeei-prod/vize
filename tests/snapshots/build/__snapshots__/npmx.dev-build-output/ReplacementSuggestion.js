import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"


const _hoisted_1 = { class: "font-medium" }
import type { ModuleReplacement } from 'module-replacements'

export default /*@__PURE__*/_defineComponent({
  __name: 'ReplacementSuggestion',
  props: {
    packageName: { type: String, required: true },
    replacement: { type: null, required: true },
    variant: { type: String, required: true },
    showAction: { type: Boolean, required: false }
  },
  emits: ["addNoDep"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const docUrl = computed(() => {
  if (props.replacement.type !== 'documented' || !props.replacement.docPath) return null
  return `https://e18e.dev/docs/replacements/${props.replacement.docPath}.html`
})

return (_ctx: any,_cache: any) => {
  const _component_ButtonBase = _resolveComponent("ButtonBase")
  const _component_LinkBase = _resolveComponent("LinkBase")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["flex items-start gap-2 px-3 py-2 rounded-lg text-sm", 
        __props.variant === 'nodep'
          ? 'bg-amber-500/10 border border-amber-600/30 text-amber-800 dark:text-amber-400'
          : 'bg-blue-500/10 border border-blue-600/30 text-blue-700 dark:text-blue-400'
      ])
    }, [ _createElementVNode("span", {
        class: _normalizeClass(["w-4 h-4 flex-shrink-0 mt-0.5", __props.variant === 'nodep' ? 'i-lucide:lightbulb' : 'i-lucide:info'])
      }, null, 2 /* CLASS */), _createElementVNode("div", { class: "min-w-0 flex-1" }, [ _createElementVNode("p", _hoisted_1, _toDisplayString(__props.packageName) + ": " + _toDisplayString(_ctx.$t('package.replacement.title')), 1 /* TEXT */), _createElementVNode("p", { class: "text-xs mt-0.5 opacity-80" }, [ (__props.replacement.type === 'native') ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _toDisplayString(_ctx.$t('package.replacement.native', {
                replacement: __props.replacement.replacement,
                nodeVersion: __props.replacement.nodeVersion,
              })) ], 64 /* STABLE_FRAGMENT */)) : (__props.replacement.type === 'simple') ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ _toDisplayString(_ctx.$t('package.replacement.simple', {
                replacement: __props.replacement.replacement,
                community: _ctx.$t('package.replacement.community'),
              })) ], 64 /* STABLE_FRAGMENT */)) : (__props.replacement.type === 'documented') ? (_openBlock(), _createElementBlock(_Fragment, { key: 2 }, [ _toDisplayString(_ctx.$t('package.replacement.documented', {
                community: _ctx.$t('package.replacement.community'),
              })) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ]) ]), _createTextVNode("\n\n    " + "\n    "), (__props.variant === 'nodep' && __props.showAction !== false) ? (_openBlock(), _createBlock(_component_ButtonBase, {
          key: 0,
          size: "small",
          "aria-label": _ctx.$t('compare.no_dependency.add_column'),
          onClick: _cache[0] || (_cache[0] = ($event: any) => (emit('addNoDep')))
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_ctx.$t('package.replacement.consider_no_dep')), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["aria-label"])) : (docUrl.value) ? (_openBlock(), _createBlock(_component_LinkBase, {
            key: 1,
            to: docUrl.value,
            variant: "button-secondary",
            size: "small"
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_ctx.$t('package.replacement.learn_more')), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    " + "\n    ") ], 2 /* CLASS */))
}
}

})
