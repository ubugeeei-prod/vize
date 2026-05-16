import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { class: "absolute inset-0 bg-accent/20 blur-3xl rounded-full", "aria-hidden": "true" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "relative text-8xl sm:text-9xl animate-bounce-slow inline-block" }, "🏖️")
const _hoisted_3 = { class: "font-mono text-3xl sm:text-4xl font-medium" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:arrow-left rtl-flip w-4 h-4", "aria-hidden": "true" })
const _hoisted_5 = { class: "sr-only sm:not-sr-only" }
const _hoisted_6 = { class: "line-through decoration-fg" }
const _hoisted_7 = { class: "text-fg" }
const _hoisted_8 = { class: "text-lg text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_9 = { class: "text-fg" }
const _hoisted_10 = { class: "font-mono text-fg text-sm" }
const _hoisted_11 = { class: "text-lg text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_12 = { class: "text-lg text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_13 = { class: "font-mono text-2xl sm:text-3xl font-bold text-fg" }
const _hoisted_14 = { class: "text-xs sm:text-sm text-fg-subtle uppercase tracking-wider" }
const _hoisted_15 = { class: "font-mono text-2xl sm:text-3xl font-bold text-fg" }
const _hoisted_16 = { class: "text-xs sm:text-sm text-fg-subtle uppercase tracking-wider" }
const _hoisted_17 = { class: "font-mono text-2xl sm:text-3xl font-bold text-fg" }
const _hoisted_18 = { class: "text-xs sm:text-sm text-fg-subtle uppercase tracking-wider" }

export default /*@__PURE__*/_defineComponent({
  __name: 'recharging',
  setup(__props) {

definePageMeta({
  name: 'vacations',
})
useSeoMeta({
  title: () => `${$t('vacations.title')} - npmx`,
  description: () => $t('vacations.meta_description'),
  ogTitle: () => `${$t('vacations.title')} - npmx`,
  ogDescription: () => $t('vacations.meta_description'),
  twitterTitle: () => `${$t('vacations.title')} - npmx`,
  twitterDescription: () => $t('vacations.meta_description'),
})
defineOgImageComponent('Default', {
  title: () => $t('vacations.title'),
  description: () => $t('vacations.meta_description'),
})
const router = useRouter()
const canGoBack = useCanGoBack()
const { data: stats } = useFetch('/api/repo-stats')
/**
 * Formats a number into a compact human-readable string.
 * e.g. 1142 → "1.1k+", 163 → "160+"
 */
function formatStat(n: number): string {
  if (n >= 1000) {
    const k = Math.floor(n / 100) / 10
    return `${k}k+`
  }
  return `${Math.floor(n / 10) * 10}+`
}
// --- Cosy fireplace easter egg ---
const logClicks = ref(0)
const fireVisible = ref(false)
function pokeLog() {
  logClicks.value++
  if (logClicks.value >= 3) {
    fireVisible.value = true
  }
}
// Icons that tile across the banner, repeating to fill.
// Classes must be written out statically so UnoCSS can detect them at build time.
const icons = [
  'i-lucide:snowflake',
  'i-lucide:mountain',
  'i-lucide:tree-pine',
  'i-lucide:coffee',
  'i-lucide:book',
  'i-lucide:music',
  'i-lucide:snowflake',
  'i-lucide:star',
  'i-lucide:moon',
] as const

return (_ctx: any,_cache: any) => {
  const _component_i18n_t = _resolveComponent("i18n-t")
  const _component_BlueskyPostEmbed = _resolveComponent("BlueskyPostEmbed")
  const _component_LinkBase = _resolveComponent("LinkBase")

  return (_openBlock(), _createElementBlock("main", { class: "container flex-1 py-12 sm:py-16 overflow-x-hidden max-w-full" }, [ _createElementVNode("article", { class: "max-w-2xl mx-auto" }, [ _createElementVNode("header", { class: "mb-12" }, [ _createElementVNode("div", { class: "max-w-2xl mx-auto py-8 bg-none flex justify-center" }, [ _createElementVNode("div", { class: "relative inline-block" }, [ _hoisted_1, _hoisted_2 ]) ]), _createElementVNode("div", { class: "flex items-baseline justify-between gap-4 mb-4" }, [ _createElementVNode("h1", _hoisted_3, _toDisplayString(_ctx.$t('vacations.heading')), 1 /* TEXT */), (_unref(canGoBack)) ? (_openBlock(), _createElementBlock("button", {
                key: 0,
                type: "button",
                class: "cursor-pointer inline-flex items-center gap-2 font-mono text-sm text-fg-muted hover:text-fg transition-colors duration-200 rounded focus-visible:outline-accent/70 shrink-0",
                onClick: _cache[0] || (_cache[0] = ($event: any) => (_unref(router).back()))
              }, [ _hoisted_4, _createElementVNode("span", _hoisted_5, _toDisplayString(_ctx.$t('nav.back')), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true) ]), _createVNode(_component_i18n_t, {
            keypath: "vacations.subtitle",
            tag: "p",
            scope: "global",
            class: "text-fg-muted text-lg sm:text-xl"
          }, {
            some: _withCtx(() => [
              _createElementVNode("span", _hoisted_6, _toDisplayString(_ctx.$t('vacations.stats.subtitle.some')), 1 /* TEXT */),
              _createTextVNode("\n            "),
              _createTextVNode(_toDisplayString(' '), 1 /* TEXT */),
              _createTextVNode("\n            "),
              _createElementVNode("strong", _hoisted_7, _toDisplayString(_ctx.$t('vacations.stats.subtitle.all')), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }) ]), _createTextVNode("\n      " + "\n      "), _createElementVNode("div", { class: "my-8" }, [ _createVNode(_component_BlueskyPostEmbed, { uri: "at://did:plc:u5zp7npt5kpueado77kuihyz/app.bsky.feed.post/3mejzn5mrcc2g" }) ]), _createElementVNode("section", { class: "prose prose-invert max-w-none space-y-8" }, [ _createElementVNode("div", null, [ _createElementVNode("h2", _hoisted_8, _toDisplayString(_ctx.$t('vacations.what.title')), 1 /* TEXT */), _createElementVNode("p", { class: "text-fg-muted leading-relaxed mb-4" }, [ _createVNode(_component_i18n_t, {
                keypath: "vacations.what.p1",
                tag: "span",
                scope: "global"
              }, {
                dates: _withCtx(() => [
                  _createElementVNode("strong", _hoisted_9, _toDisplayString(_ctx.$t('vacations.what.dates')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }) ]), _createElementVNode("p", { class: "text-fg-muted leading-relaxed mb-4" }, [ _createVNode(_component_i18n_t, {
                keypath: "vacations.what.p2",
                tag: "span",
                scope: "global"
              }, {
                garden: _withCtx(() => [
                  _createElementVNode("code", _hoisted_10, _toDisplayString(_ctx.$t('vacations.what.garden')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }) ]) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", null, [ _createElementVNode("h2", _hoisted_11, _toDisplayString(_ctx.$t('vacations.meantime.title')), 1 /* TEXT */), _createElementVNode("p", { class: "text-fg-muted leading-relaxed" }, [ _createVNode(_component_i18n_t, {
                keypath: "vacations.meantime.p1",
                tag: "span",
                scope: "global"
              }, {
                site: _withCtx(() => [
                  _createVNode(_component_LinkBase, {
                    class: "font-sans",
                    to: "/"
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode("npmx.dev")
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                repo: _withCtx(() => [
                  _createVNode(_component_LinkBase, {
                    class: "font-sans",
                    to: "https://repo.npmx.dev"
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_ctx.$t('vacations.meantime.repo_link')), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              }) ]) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", {
            class: "relative mb-12 px-4 border border-border rounded-lg bg-bg-subtle overflow-hidden select-none",
            "aria-label": _ctx.$t('vacations.illustration_alt'),
            role: "group"
          }, [ _createElementVNode("div", { class: "flex items-center gap-4 sm:gap-5 py-3 sm:py-4 w-max" }, [ (_openBlock(), _createElementBlock(_Fragment, null, _renderList(4, (n) => {
                return _createElementVNode("template", { key: `set-${n}` }, [
                  _createElementVNode("button", {
                    type: "button",
                    class: "relative shrink-0 cursor-pointer rounded transition-transform duration-200 hover:scale-110 focus-visible:outline-accent/70 w-5 h-5 sm:w-6 sm:h-6",
                    "aria-label": _ctx.$t('vacations.poke_log'),
                    onClick: pokeLog
                  }, [
                    _createElementVNode("span", {
                      class: _normalizeClass(["absolute inset-0 i-lucide:flame-kindling w-5 h-5 sm:w-6 sm:h-6 text-orange-400 transition-opacity duration-400", fireVisible.value ? 'opacity-100' : 'opacity-0'])
                    }, null, 2 /* CLASS */),
                    _createElementVNode("span", {
                      class: _normalizeClass(["absolute inset-0 i-lucide:tent w-5 h-5 sm:w-6 sm:h-6 transition-colors duration-400", fireVisible.value ? 'text-amber-700' : ''])
                    }, null, 2 /* CLASS */)
                  ], 8 /* PROPS */, ["aria-label"]),
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(icons), (icon, i) => {
                    return (_openBlock(), _createElementBlock("span", {
                      key: `${n}-${i}`,
                      class: _normalizeClass(["shrink-0 w-5 h-5 sm:w-6 sm:h-6 opacity-40", icon]),
                      "aria-hidden": "true"
                    }, 2 /* CLASS */))
                  }), 128 /* KEYED_FRAGMENT */))
                ])
              }), 64 /* STABLE_FRAGMENT */)) ]) ], 8 /* PROPS */, ["aria-label"]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", null, [ _createElementVNode("h2", _hoisted_12, _toDisplayString(_ctx.$t('vacations.return.title')), 1 /* TEXT */), _createElementVNode("p", { class: "text-fg-muted leading-relaxed mb-6" }, [ _createVNode(_component_i18n_t, {
                keypath: "vacations.return.p1",
                tag: "span",
                scope: "global"
              }, {
                social: _withCtx(() => [
                  _createVNode(_component_LinkBase, {
                    class: "font-sans",
                    to: "https://social.npmx.dev"
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_ctx.$t('vacations.return.social_link')), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              }) ]) ]), (_unref(stats)) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "grid grid-cols-3 justify-center gap-4 sm:gap-8 mb-8 py-8 border-y border-border/50"
            }, [ _createElementVNode("div", { class: "space-y-1 text-center" }, [ _createElementVNode("div", _hoisted_13, _toDisplayString(formatStat(_unref(stats).contributors)), 1 /* TEXT */), _createElementVNode("div", _hoisted_14, _toDisplayString(_ctx.$t('vacations.stats.contributors')), 1 /* TEXT */) ]), _createElementVNode("div", { class: "space-y-1 text-center" }, [ _createElementVNode("div", _hoisted_15, _toDisplayString(formatStat(_unref(stats).commits)), 1 /* TEXT */), _createElementVNode("div", _hoisted_16, _toDisplayString(_ctx.$t('vacations.stats.commits')), 1 /* TEXT */) ]), _createElementVNode("div", { class: "space-y-1 text-center" }, [ _createElementVNode("div", _hoisted_17, _toDisplayString(formatStat(_unref(stats).pullRequests)), 1 /* TEXT */), _createElementVNode("div", _hoisted_18, _toDisplayString(_ctx.$t('vacations.stats.pr')), 1 /* TEXT */) ]) ])) : _createCommentVNode("v-if", true) ]) ]) ]))
}
}

})
