import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", class: "flex-shrink-1 flex-grow-1" })
const _hoisted_2 = { class: "sr-only" }
const _hoisted_3 = { class: "font-mono" }
const _hoisted_4 = { class: "sr-only" }
const _hoisted_5 = { class: "sr-only" }
const _hoisted_6 = { class: "sr-only" }
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:chart-line w-3.5 h-3.5", "aria-hidden": "true" })
const _hoisted_8 = { class: "font-mono" }
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", class: "flex-shrink-1 flex-grow-1" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:chart-line w-3.5 h-3.5", "aria-hidden": "true" })
const _hoisted_11 = { class: "font-mono text-xs" }
import type { StructuredFilters } from '#shared/types/preferences'

export default /*@__PURE__*/_defineComponent({
  __name: 'Card',
  props: {
    result: { type: null, required: true },
    headingLevel: { type: String, required: false },
    showPublisher: { type: Boolean, required: false },
    prefetch: { type: Boolean, required: false },
    index: { type: Number, required: false },
    filters: { type: null, required: false },
    searchQuery: { type: String, required: false }
  },
  emits: ["clickKeyword"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
/** Check if this package is an exact match for the search query */
const isExactMatch = computed(() => {
  if (!props.searchQuery) return false
  const query = props.searchQuery.trim().toLowerCase()
  const name = props.result.package.name.toLowerCase()
  return query === name
})
// Process package description
const pkgDescription = useMarkdown(() => ({
  text: props.result.package.description ?? '',
  plain: true,
  packageName: props.result.package.name,
}))
const numberFormatter = useNumberFormatter()

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_ProvenanceBadge = _resolveComponent("ProvenanceBadge")
  const _component_DateTime = _resolveComponent("DateTime")
  const _component_ButtonBase = _resolveComponent("ButtonBase")
  const _component_BaseCard = _resolveComponent("BaseCard")

  return (_openBlock(), _createBlock(_component_BaseCard, { isExactMatch: isExactMatch.value }, {
      default: _withCtx(() => [
        _createElementVNode("div", { class: "mb-2 flex items-baseline justify-start gap-2" }, [
          _createVNode(_resolveDynamicComponent(__props.headingLevel ?? 'h3'), { class: "font-mono text-sm sm:text-base font-medium text-fg group-hover:text-fg transition-colors duration-200 min-w-0 break-all" }, {
            default: _withCtx(() => [
              _createVNode(_component_NuxtLink, {
                to: _ctx.packageRoute(__props.result.package.name),
                "prefetch-on": __props.prefetch ? 'visibility' : 'interaction',
                class: "decoration-none scroll-mt-48 scroll-mb-6 after:content-[''] after:absolute after:inset-0",
                "data-result-index": __props.index,
                dir: "ltr"
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(__props.result.package.name), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to", "prefetch-on", "data-result-index"]),
              (isExactMatch.value)
                ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  class: "text-xs px-1.5 py-0.5 ms-2 rounded bg-bg-elevated border border-border-hover text-fg"
                }, _toDisplayString(_ctx.$t('search.exact_match')), 1 /* TEXT */))
                : _createCommentVNode("v-if", true)
            ]),
            _: 1 /* STABLE */
          }),
          _hoisted_1,
          _createTextVNode("\n      " + "\n      "),
          _createElementVNode("div", { class: "sm:hidden text-fg-subtle flex items-center gap-1.5 shrink-0" }, [
            (__props.result.package.version)
              ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                class: "font-mono text-xs truncate max-w-20",
                title: __props.result.package.version
              }, "\n          v" + _toDisplayString(__props.result.package.version), 1 /* TEXT */))
              : _createCommentVNode("v-if", true),
            (__props.result.package.publisher?.trustedPublisher)
              ? (_openBlock(), _createBlock(_component_ProvenanceBadge, {
                key: 0,
                provider: __props.result.package.publisher.trustedPublisher.id,
                "package-name": __props.result.package.name,
                version: __props.result.package.version,
                linked: false,
                compact: ""
              }, null, 8 /* PROPS */, ["provider", "package-name", "version", "linked"]))
              : _createCommentVNode("v-if", true)
          ])
        ]),
        _createElementVNode("div", { class: "flex justify-start items-start gap-4 sm:gap-8" }, [
          _createElementVNode("div", { class: "min-w-0" }, [
            (_unref(pkgDescription))
              ? (_openBlock(), _createElementBlock("p", {
                key: 0,
                class: "text-fg-muted text-xs sm:text-sm line-clamp-2 mb-2 sm:mb-3"
              }, [
                _createElementVNode("span", { innerHTML: _unref(pkgDescription) }, null, 8 /* PROPS */, ["innerHTML"])
              ]))
              : _createCommentVNode("v-if", true),
            _createElementVNode("div", { class: "flex flex-wrap items-center gap-x-3 sm:gap-x-4 gap-y-2 text-xs text-fg-muted" }, [
              (__props.showPublisher || __props.result.package.date)
                ? (_openBlock(), _createElementBlock("dl", {
                  key: 0,
                  class: "flex items-center gap-4 m-0"
                }, [
                  (__props.showPublisher && __props.result.package.publisher?.username)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "flex items-center gap-1.5"
                    }, [
                      _createElementVNode("dt", _hoisted_2, _toDisplayString(_ctx.$t('package.card.publisher')), 1 /* TEXT */),
                      _createElementVNode("dd", _hoisted_3, _toDisplayString(__props.result.package.publisher.username), 1 /* TEXT */)
                    ]))
                    : _createCommentVNode("v-if", true),
                  (__props.result.package.date)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "flex items-center gap-1.5"
                    }, [
                      _createElementVNode("dt", _hoisted_4, _toDisplayString(_ctx.$t('package.card.published')), 1 /* TEXT */),
                      _createElementVNode("dd", null, [
                        _createVNode(_component_DateTime, {
                          datetime: __props.result.package.date,
                          year: "numeric",
                          month: "short",
                          day: "numeric"
                        }, null, 8 /* PROPS */, ["datetime"])
                      ])
                    ]))
                    : _createCommentVNode("v-if", true),
                  (__props.result.package.license)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "flex items-center gap-1.5"
                    }, [
                      _createElementVNode("dt", _hoisted_5, _toDisplayString(_ctx.$t('package.card.license')), 1 /* TEXT */),
                      _createElementVNode("dd", null, _toDisplayString(__props.result.package.license), 1 /* TEXT */)
                    ]))
                    : _createCommentVNode("v-if", true)
                ]))
                : _createCommentVNode("v-if", true)
            ]),
            _createTextVNode("\n        " + "\n        "),
            (__props.result.downloads?.weekly)
              ? (_openBlock(), _createElementBlock("dl", {
                key: 0,
                class: "sm:hidden flex items-center gap-4 mt-2 text-xs text-fg-muted m-0"
              }, [
                _createElementVNode("div", { class: "flex items-center gap-1.5" }, [
                  _createElementVNode("dt", _hoisted_6, _toDisplayString(_ctx.$t('package.card.weekly_downloads')), 1 /* TEXT */),
                  _createElementVNode("dd", { class: "flex items-center gap-1.5" }, [
                    _hoisted_7,
                    _createElementVNode("span", _hoisted_8, _toDisplayString(_ctx.$n(__props.result.downloads.weekly)) + "/w", 1 /* TEXT */)
                  ])
                ])
              ]))
              : _createCommentVNode("v-if", true)
          ]),
          _hoisted_9,
          _createTextVNode("\n      " + "\n      "),
          _createElementVNode("div", { class: "hidden sm:flex flex-col gap-2 shrink-0" }, [
            _createElementVNode("div", { class: "text-fg-subtle flex items-start gap-2 justify-end" }, [
              (__props.result.package.version)
                ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  class: "font-mono text-xs truncate max-w-32",
                  title: __props.result.package.version
                }, "\n            v" + _toDisplayString(__props.result.package.version), 1 /* TEXT */))
                : _createCommentVNode("v-if", true),
              (__props.result.package.publisher?.trustedPublisher)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: "flex items-center gap-1.5 shrink-0 max-w-32"
                }, [
                  _createVNode(_component_ProvenanceBadge, {
                    provider: __props.result.package.publisher.trustedPublisher.id,
                    "package-name": __props.result.package.name,
                    version: __props.result.package.version,
                    linked: false,
                    compact: ""
                  }, null, 8 /* PROPS */, ["provider", "package-name", "version", "linked"])
                ]))
                : _createCommentVNode("v-if", true)
            ]),
            (__props.result.downloads?.weekly)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: "text-fg-subtle gap-2 flex items-center justify-end"
              }, [
                _hoisted_10,
                _createElementVNode("span", _hoisted_11, _toDisplayString(_ctx.$n(__props.result.downloads.weekly)) + " " + _toDisplayString(_ctx.$t('common.per_week')), 1 /* TEXT */)
              ]))
              : _createCommentVNode("v-if", true)
          ])
        ]),
        (__props.result.package.keywords?.length)
          ? (_openBlock(), _createElementBlock("ul", {
            key: 0,
            role: "list",
            "aria-label": _ctx.$t('package.card.keywords'),
            class: "relative z-10 flex flex-wrap gap-1.5 mt-3 pt-3 border-t border-border list-none m-0 p-0 pointer-events-none items-center"
          }, [
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.result.package.keywords.slice(0, 5), (keyword) => {
              return (_openBlock(), _createElementBlock("li", { key: keyword }, [
                _createVNode(_component_ButtonBase, {
                  class: "pointer-events-auto",
                  size: "small",
                  "aria-pressed": props.filters?.keywords.includes(keyword),
                  title: `Filter by ${keyword}`,
                  onClick: _cache[0] || (_cache[0] = _withModifiers(($event: any) => (emit('clickKeyword', keyword)), ["stop"]))
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(keyword), 1 /* TEXT */)
                  ]),
                  _: 2 /* DYNAMIC */
                }, 8 /* PROPS */, ["aria-pressed", "title"])
              ]))
            }), 128 /* KEYED_FRAGMENT */)),
            _createElementVNode("li", null, [
              (__props.result.package.keywords.length > 5)
                ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  class: "text-fg-subtle text-xs pointer-events-auto",
                  title: __props.result.package.keywords.slice(5).join(', ')
                }, "\n          +" + _toDisplayString(_unref(numberFormatter).format(__props.result.package.keywords.length - 5)), 1 /* TEXT */))
                : _createCommentVNode("v-if", true)
            ])
          ]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["isExactMatch"]))
}
}

})
