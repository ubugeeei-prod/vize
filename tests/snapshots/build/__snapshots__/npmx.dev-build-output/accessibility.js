import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "font-mono text-3xl sm:text-4xl font-medium" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:arrow-left rtl-flip w-4 h-4", "aria-hidden": "true" })
const _hoisted_3 = { class: "sr-only sm:not-sr-only" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("strong", { class: "text-fg" }, "npmx")
const _hoisted_5 = { class: "text-lg text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_6 = { class: "text-fg-muted leading-relaxed mb-4" }
const _hoisted_7 = { class: "text-lg text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_8 = { class: "text-fg-muted leading-relaxed mb-4" }
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_15 = { class: "text-lg text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_16 = { class: "text-fg-muted leading-relaxed" }
const _hoisted_17 = { class: "text-lg text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_18 = /*#__PURE__*/ _createElementVNode("strong", { class: "text-fg" }, "npmx")
const _hoisted_19 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:external-link rtl-flip w-4 h-4", "aria-hidden": "true" })

export default /*@__PURE__*/_defineComponent({
  __name: 'accessibility',
  setup(__props) {

definePageMeta({
  name: 'accessibility',
})
useSeoMeta({
  title: () => `${$t('a11y.title')} - npmx`,
  description: () => $t('a11y.welcome', { app: 'npmx' }),
})
defineOgImageComponent('Default', {
  title: () => $t('a11y.title'),
  description: () => $t('a11y.welcome', { app: 'npmx' }),
})
const router = useRouter()
const canGoBack = useCanGoBack()

return (_ctx: any,_cache: any) => {
  const _component_i18n_t = _resolveComponent("i18n-t")
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (_openBlock(), _createElementBlock("main", { class: "container flex-1 py-12 sm:py-16 overflow-x-hidden" }, [ _createElementVNode("article", { class: "max-w-2xl mx-auto" }, [ _createElementVNode("header", { class: "mb-12" }, [ _createElementVNode("div", { class: "flex items-baseline justify-between gap-4 mb-4" }, [ _createElementVNode("h1", _hoisted_1, _toDisplayString(_ctx.$t('a11y.title')), 1 /* TEXT */), (_unref(canGoBack)) ? (_openBlock(), _createElementBlock("button", {
                key: 0,
                type: "button",
                class: "cursor-pointer inline-flex items-center gap-2 font-mono text-sm text-fg-muted hover:text-fg transition-colors duration-200 rounded shrink-0",
                onClick: _cache[0] || (_cache[0] = ($event: any) => (_unref(router).back()))
              }, [ _hoisted_2, _createElementVNode("span", _hoisted_3, _toDisplayString(_ctx.$t('nav.back')), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true) ]) ]), _createElementVNode("section", { class: "prose prose-invert max-w-none space-y-8" }, [ _createElementVNode("p", { class: "text-fg-muted leading-relaxed" }, [ _createVNode(_component_i18n_t, {
              keypath: "a11y.welcome",
              tag: "span",
              scope: "global"
            }, {
              app: _withCtx(() => [
                _hoisted_4
              ]),
              _: 1 /* STABLE */
            }) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", null, [ _createElementVNode("h2", _hoisted_5, _toDisplayString(_ctx.$t('a11y.approach.title')), 1 /* TEXT */), _createElementVNode("p", _hoisted_6, _toDisplayString(_ctx.$t('a11y.approach.p1')), 1 /* TEXT */), _createElementVNode("p", { class: "text-fg-muted leading-relaxed" }, [ _createVNode(_component_i18n_t, {
                keypath: "a11y.approach.p2",
                tag: "span",
                scope: "global"
              }, {
                about: _withCtx(() => [
                  _createVNode(_component_NuxtLink, {
                    to: { name: 'about' },
                    class: "text-fg-muted hover:text-fg underline decoration-fg-subtle/50 hover:decoration-fg"
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_ctx.$t('a11y.approach.about_link')), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["to"])
                ]),
                _: 1 /* STABLE */
              }) ]) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", null, [ _createElementVNode("h2", _hoisted_7, _toDisplayString(_ctx.$t('a11y.measures.title')), 1 /* TEXT */), _createElementVNode("p", _hoisted_8, _toDisplayString(_ctx.$t('a11y.measures.p1')), 1 /* TEXT */), _createElementVNode("ul", { class: "space-y-3 text-fg-muted list-none p-0" }, [ _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_9, _createElementVNode("span", null, _toDisplayString(_ctx.$t('a11y.measures.li1')), 1 /* TEXT */) ]), _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_10, _createElementVNode("span", null, _toDisplayString(_ctx.$t('a11y.measures.li2')), 1 /* TEXT */) ]), _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_11, _createElementVNode("span", null, _toDisplayString(_ctx.$t('a11y.measures.li3')), 1 /* TEXT */) ]), _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_12, _createElementVNode("span", null, _toDisplayString(_ctx.$t('a11y.measures.li4')), 1 /* TEXT */) ]), _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_13, _createElementVNode("span", null, _toDisplayString(_ctx.$t('a11y.measures.li5')), 1 /* TEXT */) ]), _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_14, _createElementVNode("span", null, _toDisplayString(_ctx.$t('a11y.measures.li6')), 1 /* TEXT */) ]) ]) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", null, [ _createElementVNode("h2", _hoisted_15, _toDisplayString(_ctx.$t('a11y.limitations.title')), 1 /* TEXT */), _createElementVNode("p", _hoisted_16, _toDisplayString(_ctx.$t('a11y.limitations.p1')), 1 /* TEXT */) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", null, [ _createElementVNode("h2", _hoisted_17, _toDisplayString(_ctx.$t('a11y.contact.title')), 1 /* TEXT */), _createElementVNode("p", { class: "text-fg-muted leading-relaxed" }, [ _createVNode(_component_i18n_t, {
                keypath: "a11y.contact.p1",
                tag: "span",
                scope: "global"
              }, {
                app: _withCtx(() => [
                  _hoisted_18
                ]),
                link: _withCtx(() => [
                  _createElementVNode("a", {
                    href: "https://github.com/npmx-dev/npmx.dev/issues",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "inline-flex items-center gap-1 text-fg-muted hover:text-fg underline decoration-fg-subtle/50 hover:decoration-fg"
                  }, [
                    _createTextVNode(_toDisplayString(_ctx.$t('a11y.contact.link')) + "\n                  ", 1 /* TEXT */),
                    _hoisted_19
                  ])
                ]),
                _: 1 /* STABLE */
              }) ]) ]) ]) ]) ]))
}
}

})
