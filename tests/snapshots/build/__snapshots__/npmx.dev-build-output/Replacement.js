import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:lightbulb w-4 h-4", "aria-hidden": "true" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:external-link w-3 h-3", "aria-hidden": "true" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:external-link w-3 h-3", "aria-hidden": "true" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:external-link w-3 h-3", "aria-hidden": "true" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:external-link w-3 h-3", "aria-hidden": "true" })
import type { ModuleReplacement } from 'module-replacements'

export default /*@__PURE__*/_defineComponent({
  __name: 'Replacement',
  props: {
    replacement: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const mdnUrl = computed(() => {
  if (props.replacement.type !== 'native' || !props.replacement.mdnPath) return null
  return `https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/${props.replacement.mdnPath}`
})
const docPath = computed(() => {
  if (props.replacement.type !== 'documented' || !props.replacement.docPath) return null
  return `https://e18e.dev/docs/replacements/${props.replacement.docPath}.html`
})

return (_ctx: any,_cache: any) => {
  const _component_i18n_t = _resolveComponent("i18n-t")

  return (_openBlock(), _createElementBlock("div", { class: "border border-amber-600/40 bg-amber-500/10 rounded-lg px-3 py-2 text-base text-amber-800 dark:text-amber-400" }, [ _createElementVNode("h2", { class: "font-medium mb-1 flex items-center gap-2" }, [ _hoisted_1, _createTextVNode("\n      " + _toDisplayString(_ctx.$t('package.replacement.title')), 1 /* TEXT */) ]), _createElementVNode("p", { class: "text-sm m-0" }, [ (__props.replacement.type === 'native') ? (_openBlock(), _createBlock(_component_i18n_t, {
            key: 0,
            keypath: "package.replacement.native",
            scope: "global"
          }, {
            replacement: _withCtx(() => [
              _createTextVNode(_toDisplayString(__props.replacement.replacement), 1 /* TEXT */)
            ]),
            nodeVersion: _withCtx(() => [
              _createTextVNode(_toDisplayString(__props.replacement.nodeVersion), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          })) : (__props.replacement.type === 'simple') ? (_openBlock(), _createBlock(_component_i18n_t, {
              key: 1,
              keypath: "package.replacement.simple",
              scope: "global"
            }, {
              community: _withCtx(() => [
                _createElementVNode("a", {
                  href: "https://e18e.dev/docs/replacements/",
                  target: "_blank",
                  rel: "noopener noreferrer",
                  class: "inline-flex items-center gap-1 ms-1 underline underline-offset-4 decoration-amber-600/60 dark:decoration-amber-400/50 hover:decoration-fg transition-colors"
                }, [
                  _createTextVNode(_toDisplayString(_ctx.$t('package.replacement.community')) + "\n            ", 1 /* TEXT */),
                  _hoisted_2
                ])
              ]),
              replacement: _withCtx(() => [
                _createTextVNode(_toDisplayString(__props.replacement.replacement), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            })) : (__props.replacement.type === 'documented') ? (_openBlock(), _createBlock(_component_i18n_t, {
              key: 2,
              keypath: "package.replacement.documented",
              scope: "global"
            }, {
              community: _withCtx(() => [
                _createElementVNode("a", {
                  href: "https://e18e.dev/docs/replacements/",
                  target: "_blank",
                  rel: "noopener noreferrer",
                  class: "inline-flex items-center gap-1 ms-1 underline underline-offset-4 decoration-amber-600/60 dark:decoration-amber-400/50 hover:decoration-fg transition-colors"
                }, [
                  _createTextVNode(_toDisplayString(_ctx.$t('package.replacement.community')) + "\n            ", 1 /* TEXT */),
                  _hoisted_3
                ])
              ]),
              _: 1 /* STABLE */
            })) : (_openBlock(), _createElementBlock(_Fragment, { key: 3 }, [ _toDisplayString(_ctx.$t('package.replacement.none')) ], 64 /* STABLE_FRAGMENT */)), (mdnUrl.value) ? (_openBlock(), _createElementBlock("a", {
            key: 0,
            href: mdnUrl.value,
            target: "_blank",
            rel: "noopener noreferrer",
            class: "inline-flex items-center gap-1 ms-1 underline underline-offset-4 decoration-amber-600/60 dark:decoration-amber-400/50 hover:decoration-fg transition-colors"
          }, [ _toDisplayString(_ctx.$t('package.replacement.mdn')), _createTextVNode("\n        "), _hoisted_4 ])) : _createCommentVNode("v-if", true), (docPath.value) ? (_openBlock(), _createElementBlock("a", {
            key: 0,
            href: docPath.value,
            target: "_blank",
            rel: "noopener noreferrer",
            class: "inline-flex items-center gap-1 ms-1 underline underline-offset-4 decoration-amber-600/60 dark:decoration-amber-400/50 hover:decoration-fg transition-colors"
          }, [ _toDisplayString(_ctx.$t('package.replacement.learn_more')), _createTextVNode("\n        "), _hoisted_5 ])) : _createCommentVNode("v-if", true) ]) ]))
}
}

})
