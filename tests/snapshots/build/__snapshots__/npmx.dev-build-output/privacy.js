import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "font-mono text-3xl sm:text-4xl font-medium" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:arrow-left rtl-flip w-4 h-4", "aria-hidden": "true" })
const _hoisted_3 = { class: "sr-only sm:not-sr-only" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("strong", { class: "text-fg" }, "npmx")
const _hoisted_5 = { class: "text-lg text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_6 = { class: "text-fg-muted leading-relaxed" }
const _hoisted_7 = { class: "text-lg text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_8 = { class: "text-fg" }
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_11 = { class: "text-lg text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_12 = { class: "text-fg" }
const _hoisted_13 = { class: "text-fg" }
const _hoisted_14 = { class: "text-lg text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_15 = { class: "text-fg" }
const _hoisted_16 = { class: "text-fg-muted leading-relaxed mb-4" }
const _hoisted_17 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_18 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:external-link rtl-flip w-4 h-4", "aria-hidden": "true" })
const _hoisted_19 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_20 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:external-link rtl-flip w-4 h-4", "aria-hidden": "true" })
const _hoisted_21 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_22 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:external-link rtl-flip w-4 h-4", "aria-hidden": "true" })
const _hoisted_23 = { class: "text-lg text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_24 = { class: "text-fg" }
const _hoisted_25 = { class: "text-fg-muted leading-relaxed mb-4" }
const _hoisted_26 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_27 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_28 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_29 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_30 = { class: "text-fg-muted leading-relaxed" }
const _hoisted_31 = { class: "text-lg text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_32 = { class: "text-fg" }
const _hoisted_33 = { class: "text-lg text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_34 = { class: "text-fg-muted leading-relaxed" }
const _hoisted_35 = { class: "text-lg text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_36 = { class: "text-fg-muted leading-relaxed mb-4" }
const _hoisted_37 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_38 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_39 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_40 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle shrink-0" }, "&mdash;")
const _hoisted_41 = { class: "text-fg-muted leading-relaxed" }
const _hoisted_42 = { class: "text-lg text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_43 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:external-link rtl-flip w-4 h-4", "aria-hidden": "true" })
const _hoisted_44 = { class: "text-lg text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_45 = { class: "text-fg-muted leading-relaxed" }

export default /*@__PURE__*/_defineComponent({
  __name: 'privacy',
  setup(__props) {

definePageMeta({
  name: 'privacy',
})
useSeoMeta({
  title: () => `${$t('privacy_policy.title')} - npmx`,
  description: () => $t('privacy_policy.welcome', { app: 'npmx' }),
})
defineOgImageComponent('Default', {
  title: () => $t('privacy_policy.title'),
  description: () => $t('privacy_policy.welcome', { app: 'npmx' }),
})
const router = useRouter()
const canGoBack = useCanGoBack()
const buildInfo = useAppConfig().buildInfo
const { locale } = useI18n()

return (_ctx: any,_cache: any) => {
  const _component_NuxtTime = _resolveComponent("NuxtTime")
  const _component_i18n_t = _resolveComponent("i18n-t")
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (_openBlock(), _createElementBlock("main", { class: "container flex-1 py-12 sm:py-16 overflow-x-hidden" }, [ _createElementVNode("article", { class: "max-w-2xl mx-auto" }, [ _createElementVNode("header", { class: "mb-12" }, [ _createElementVNode("div", { class: "flex items-baseline justify-between gap-4 mb-4" }, [ _createElementVNode("h1", _hoisted_1, _toDisplayString(_ctx.$t('privacy_policy.title')), 1 /* TEXT */), (_unref(canGoBack)) ? (_openBlock(), _createElementBlock("button", {
                key: 0,
                type: "button",
                class: "cursor-pointer inline-flex items-center gap-2 font-mono text-sm text-fg-muted hover:text-fg transition-colors duration-200 rounded focus-visible:outline-accent/70 shrink-0",
                onClick: _cache[0] || (_cache[0] = ($event: any) => (_unref(router).back()))
              }, [ _hoisted_2, _createElementVNode("span", _hoisted_3, _toDisplayString(_ctx.$t('nav.back')), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true) ]), _createVNode(_component_i18n_t, {
            keypath: "privacy_policy.last_updated",
            tag: "p",
            scope: "global",
            class: "text-fg-muted text-lg"
          }, {
            date: _withCtx(() => [
              _createVNode(_component_NuxtTime, {
                locale: _unref(locale),
                datetime: _unref(buildInfo).privacyPolicyDate,
                "date-style": "long",
                "time-style": "medium"
              }, null, 8 /* PROPS */, ["locale", "datetime"])
            ]),
            _: 1 /* STABLE */
          }) ]), _createElementVNode("section", { class: "prose prose-invert max-w-none space-y-8" }, [ _createElementVNode("p", { class: "text-fg-muted leading-relaxed" }, [ _createVNode(_component_i18n_t, {
              keypath: "privacy_policy.welcome",
              tag: "span",
              scope: "global"
            }, {
              app: _withCtx(() => [
                _hoisted_4
              ]),
              _: 1 /* STABLE */
            }) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", null, [ _createElementVNode("h2", _hoisted_5, _toDisplayString(_ctx.$t('privacy_policy.cookies.what_are.title')), 1 /* TEXT */), _createElementVNode("p", _hoisted_6, _toDisplayString(_ctx.$t('privacy_policy.cookies.what_are.p1')), 1 /* TEXT */) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", null, [ _createElementVNode("h2", _hoisted_7, _toDisplayString(_ctx.$t('privacy_policy.cookies.types.title')), 1 /* TEXT */), _createElementVNode("p", { class: "text-fg-muted leading-relaxed mb-4" }, [ _createVNode(_component_i18n_t, {
                keypath: "privacy_policy.cookies.types.p1",
                tag: "span",
                scope: "global"
              }, {
                bold: _withCtx(() => [
                  _createElementVNode("strong", _hoisted_8, _toDisplayString(_ctx.$t('privacy_policy.cookies.types.bold')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }) ]), _createElementVNode("ul", { class: "space-y-3 text-fg-muted list-none p-0" }, [ _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_9, _createElementVNode("span", null, [ _createVNode(_component_i18n_t, {
                    keypath: "privacy_policy.cookies.types.li1",
                    tag: "span"
                  }, {
                    li11: _withCtx(() => [
                      _createElementVNode("strong", { class: "text-fg font-mono text-sm" }, [
                        _createElementVNode("bdi", null, _toDisplayString(_ctx.$t('privacy_policy.cookies.types.cookie_vdpl')), 1 /* TEXT */)
                      ])
                    ]),
                    separator: _withCtx(() => [
                      _createElementVNode("bdi", null, _toDisplayString(_ctx.$t('privacy_policy.cookies.types.separator')), 1 /* TEXT */)
                    ]),
                    li12: _withCtx(() => [
                      _createElementVNode("bdi", null, _toDisplayString(_ctx.$t('privacy_policy.cookies.types.cookie_vdpl_desc')), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }) ]) ]), _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_10, _createElementVNode("span", null, [ _createVNode(_component_i18n_t, {
                    keypath: "privacy_policy.cookies.types.li2",
                    tag: "span"
                  }, {
                    li21: _withCtx(() => [
                      _createElementVNode("strong", { class: "text-fg font-mono text-sm" }, [
                        _createElementVNode("bdi", null, _toDisplayString(_ctx.$t('privacy_policy.cookies.types.cookie_h3')), 1 /* TEXT */)
                      ])
                    ]),
                    separator: _withCtx(() => [
                      _createElementVNode("bdi", null, _toDisplayString(_ctx.$t('privacy_policy.cookies.types.separator')), 1 /* TEXT */)
                    ]),
                    li22: _withCtx(() => [
                      _createElementVNode("bdi", null, _toDisplayString(_ctx.$t('privacy_policy.cookies.types.cookie_h3_desc')), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }) ]) ]) ]) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", null, [ _createElementVNode("h2", _hoisted_11, _toDisplayString(_ctx.$t('privacy_policy.cookies.local_storage.title')), 1 /* TEXT */), _createElementVNode("p", { class: "text-fg-muted leading-relaxed mb-4" }, [ _createVNode(_component_i18n_t, {
                keypath: "privacy_policy.cookies.local_storage.p1",
                tag: "span",
                scope: "global"
              }, {
                bold: _withCtx(() => [
                  _createElementVNode("strong", _hoisted_12, _toDisplayString(_ctx.$t('privacy_policy.cookies.local_storage.bold')), 1 /* TEXT */)
                ]),
                settings: _withCtx(() => [
                  _createVNode(_component_NuxtLink, {
                    to: { name: 'settings' },
                    class: "text-fg-muted hover:text-fg underline decoration-fg-subtle/50 hover:decoration-fg"
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_ctx.$t('privacy_policy.cookies.local_storage.settings')), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["to"])
                ]),
                _: 1 /* STABLE */
              }) ]), _createElementVNode("p", { class: "text-fg-muted leading-relaxed" }, [ _createVNode(_component_i18n_t, {
                keypath: "privacy_policy.cookies.local_storage.p2",
                tag: "span",
                scope: "global"
              }, {
                bold2: _withCtx(() => [
                  _createElementVNode("strong", _hoisted_13, _toDisplayString(_ctx.$t('privacy_policy.cookies.local_storage.bold2')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }) ]) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", null, [ _createElementVNode("h2", _hoisted_14, _toDisplayString(_ctx.$t('privacy_policy.cookies.management.title')), 1 /* TEXT */), _createElementVNode("p", { class: "text-fg-muted leading-relaxed mb-4" }, [ _createVNode(_component_i18n_t, {
                keypath: "privacy_policy.cookies.management.p1",
                tag: "span",
                scope: "global"
              }, {
                bold: _withCtx(() => [
                  _createElementVNode("strong", _hoisted_15, _toDisplayString(_ctx.$t('privacy_policy.cookies.management.bold')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }) ]), _createElementVNode("p", _hoisted_16, _toDisplayString(_ctx.$t('privacy_policy.cookies.management.p2')), 1 /* TEXT */), _createElementVNode("ul", { class: "space-y-3 text-fg-muted list-none p-0" }, [ _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_17, _createElementVNode("a", {
                  href: "https://support.google.com/chrome/answer/95647?hl=en",
                  target: "_blank",
                  rel: "noopener noreferrer",
                  class: "inline-flex items-center gap-1 text-fg-muted hover:text-fg underline decoration-fg-subtle/50 hover:decoration-fg"
                }, [ _createTextVNode(_toDisplayString(_ctx.$t('privacy_policy.cookies.management.chrome')) + "\n                ", 1 /* TEXT */), _hoisted_18 ]) ]), _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_19, _createElementVNode("a", {
                  href: "https://support.mozilla.org/en-US/kb/clear-cookies-and-site-data-firefox",
                  target: "_blank",
                  rel: "noopener noreferrer",
                  class: "inline-flex items-center gap-1 text-fg-muted hover:text-fg underline decoration-fg-subtle/50 hover:decoration-fg"
                }, [ _createTextVNode(_toDisplayString(_ctx.$t('privacy_policy.cookies.management.firefox')) + "\n                ", 1 /* TEXT */), _hoisted_20 ]) ]), _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_21, _createElementVNode("a", {
                  href: "https://support.microsoft.com/en-us/windows/manage-cookies-in-microsoft-edge-view-allow-block-delete-and-use-168dab11-0753-043d-7c16-ede5947fc64d",
                  target: "_blank",
                  rel: "noopener noreferrer",
                  class: "inline-flex items-center gap-1 text-fg-muted hover:text-fg underline decoration-fg-subtle/50 hover:decoration-fg"
                }, [ _createTextVNode(_toDisplayString(_ctx.$t('privacy_policy.cookies.management.edge')) + "\n                ", 1 /* TEXT */), _hoisted_22 ]) ]) ]) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", null, [ _createElementVNode("h2", _hoisted_23, _toDisplayString(_ctx.$t('privacy_policy.analytics.title')), 1 /* TEXT */), _createElementVNode("p", { class: "text-fg-muted leading-relaxed mb-4" }, [ _createVNode(_component_i18n_t, {
                keypath: "privacy_policy.analytics.p1",
                tag: "span",
                scope: "global"
              }, {
                bold: _withCtx(() => [
                  _createElementVNode("strong", _hoisted_24, _toDisplayString(_ctx.$t('privacy_policy.analytics.bold')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }) ]), _createElementVNode("p", _hoisted_25, _toDisplayString(_ctx.$t('privacy_policy.analytics.p2')), 1 /* TEXT */), _createElementVNode("ul", { class: "space-y-3 text-fg-muted list-none p-0 mb-4" }, [ _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_26, _createElementVNode("span", null, _toDisplayString(_ctx.$t('privacy_policy.analytics.li1')), 1 /* TEXT */) ]), _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_27, _createElementVNode("span", null, _toDisplayString(_ctx.$t('privacy_policy.analytics.li2')), 1 /* TEXT */) ]), _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_28, _createElementVNode("span", null, _toDisplayString(_ctx.$t('privacy_policy.analytics.li3')), 1 /* TEXT */) ]), _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_29, _createElementVNode("span", null, _toDisplayString(_ctx.$t('privacy_policy.analytics.li4')), 1 /* TEXT */) ]) ]), _createElementVNode("p", _hoisted_30, _toDisplayString(_ctx.$t('privacy_policy.analytics.p3')), 1 /* TEXT */) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", null, [ _createElementVNode("h2", _hoisted_31, _toDisplayString(_ctx.$t('privacy_policy.authenticated.title')), 1 /* TEXT */), _createElementVNode("p", { class: "text-fg-muted leading-relaxed mb-4" }, [ _createVNode(_component_i18n_t, {
                keypath: "privacy_policy.authenticated.p1",
                tag: "span",
                scope: "global"
              }, {
                bold: _withCtx(() => [
                  _createElementVNode("strong", _hoisted_32, _toDisplayString(_ctx.$t('privacy_policy.authenticated.bold')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }) ]), _createElementVNode("p", { class: "text-fg-muted leading-relaxed" }, [ _createVNode(_component_i18n_t, {
                keypath: "privacy_policy.authenticated.p2",
                tag: "span",
                scope: "global"
              }, {
                settings: _withCtx(() => [
                  _createVNode(_component_NuxtLink, {
                    to: { name: 'settings' },
                    class: "text-fg-muted hover:text-fg underline decoration-fg-subtle/50 hover:decoration-fg"
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_ctx.$t('privacy_policy.authenticated.settings')), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["to"])
                ]),
                _: 1 /* STABLE */
              }) ]) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", null, [ _createElementVNode("h2", _hoisted_33, _toDisplayString(_ctx.$t('privacy_policy.data_retention.title')), 1 /* TEXT */), _createElementVNode("p", _hoisted_34, _toDisplayString(_ctx.$t('privacy_policy.data_retention.p1')), 1 /* TEXT */) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", null, [ _createElementVNode("h2", _hoisted_35, _toDisplayString(_ctx.$t('privacy_policy.your_rights.title')), 1 /* TEXT */), _createElementVNode("p", _hoisted_36, _toDisplayString(_ctx.$t('privacy_policy.your_rights.p1')), 1 /* TEXT */), _createElementVNode("ul", { class: "space-y-3 text-fg-muted list-none p-0 mb-4" }, [ _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_37, _createElementVNode("span", null, _toDisplayString(_ctx.$t('privacy_policy.your_rights.li1')), 1 /* TEXT */) ]), _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_38, _createElementVNode("span", null, _toDisplayString(_ctx.$t('privacy_policy.your_rights.li2')), 1 /* TEXT */) ]), _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_39, _createElementVNode("span", null, _toDisplayString(_ctx.$t('privacy_policy.your_rights.li3')), 1 /* TEXT */) ]), _createElementVNode("li", { class: "flex items-start gap-3" }, [ _hoisted_40, _createElementVNode("span", null, _toDisplayString(_ctx.$t('privacy_policy.your_rights.li4')), 1 /* TEXT */) ]) ]), _createElementVNode("p", _hoisted_41, _toDisplayString(_ctx.$t('privacy_policy.your_rights.p2')), 1 /* TEXT */) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", null, [ _createElementVNode("h2", _hoisted_42, _toDisplayString(_ctx.$t('privacy_policy.contact.title')), 1 /* TEXT */), _createElementVNode("p", { class: "text-fg-muted leading-relaxed" }, [ _createVNode(_component_i18n_t, {
                keypath: "privacy_policy.contact.p1",
                tag: "span",
                scope: "global"
              }, {
                link: _withCtx(() => [
                  _createElementVNode("a", {
                    href: "https://github.com/npmx-dev/npmx.dev/issues",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "inline-flex items-center gap-1 text-fg-muted hover:text-fg underline decoration-fg-subtle/50 hover:decoration-fg"
                  }, [
                    _createTextVNode(_toDisplayString(_ctx.$t('privacy_policy.contact.link')) + "\n                  ", 1 /* TEXT */),
                    _hoisted_43
                  ])
                ]),
                _: 1 /* STABLE */
              }) ]) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("div", null, [ _createElementVNode("h2", _hoisted_44, _toDisplayString(_ctx.$t('privacy_policy.changes.title')), 1 /* TEXT */), _createElementVNode("p", _hoisted_45, _toDisplayString(_ctx.$t('privacy_policy.changes.p1')), 1 /* TEXT */) ]) ]) ]) ]))
}
}

})
