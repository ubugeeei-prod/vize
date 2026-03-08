import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = { class: "font-mono text-balance m-0 hidden sm:block" }
const _hoisted_2 = { class: "mb-2 font-mono text-fg-subtle" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("kbd", { class: "kbd" }, "/")
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("kbd", { class: "kbd" }, "?")
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("kbd", { class: "kbd" }, ",")
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("kbd", { class: "kbd" }, "c")
const _hoisted_7 = { class: "mb-2 font-mono text-fg-subtle" }
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("kbd", { class: "kbd" }, "↑")
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("kbd", { class: "kbd" }, "↓")
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("kbd", { class: "kbd" }, "Enter")
const _hoisted_11 = { class: "mb-2 font-mono text-fg-subtle" }
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("kbd", { class: "kbd" }, ".")
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("kbd", { class: "kbd" }, "d")
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("kbd", { class: "kbd" }, "c")
const _hoisted_15 = { class: "sm:hidden" }
const _hoisted_16 = { class: "hidden sm:inline" }
import { NPMX_DOCS_SITE } from '#shared/utils/constants'

export default /*@__PURE__*/_defineComponent({
  __name: 'AppFooter',
  setup(__props) {

const route = useRoute()
const isHome = computed(() => route.name === 'index')
const modalRef = useTemplateRef('modalRef')
const showModal = () => modalRef.value?.showModal?.()
const closeModal = () => modalRef.value?.close?.()

return (_ctx: any,_cache: any) => {
  const _component_LinkBase = _resolveComponent("LinkBase")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_i18n_t = _resolveComponent("i18n-t")
  const _component_Modal = _resolveComponent("Modal")
  const _component_BuildEnvironment = _resolveComponent("BuildEnvironment")

  return (_openBlock(), _createElementBlock("footer", { class: "border-t border-border mt-auto" }, [ _createElementVNode("div", { class: "container py-3 sm:py-8 flex flex-col gap-2 sm:gap-4 text-fg-subtle text-sm" }, [ _createElementVNode("div", { class: "flex flex-col sm:flex-row sm:flex-wrap items-center sm:items-baseline justify-between gap-2 sm:gap-4" }, [ _createElementVNode("div", null, [ _createElementVNode("p", _hoisted_1, _toDisplayString(_ctx.$t('tagline')), 1 /* TEXT */) ]), _createTextVNode("\n        " + "\n        "), _createElementVNode("div", { class: "hidden sm:flex items-center gap-6 min-h-11 text-xs" }, [ _createVNode(_component_LinkBase, { to: { name: 'about' } }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('footer.about')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to"]), _createVNode(_component_LinkBase, { to: { name: 'privacy' } }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('privacy_policy.title')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to"]), _createVNode(_component_LinkBase, { to: { name: 'accessibility' } }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('a11y.footer_title')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to"]), _createElementVNode("button", {
              type: "button",
              class: "cursor-pointer group inline-flex gap-x-1 items-center justify-center underline-offset-[0.2rem] underline decoration-1 decoration-fg/30 font-mono text-fg hover:(decoration-accent text-accent) focus-visible:(decoration-accent text-accent) transition-colors duration-200",
              onClick: _withModifiers(showModal, ["prevent"]),
              "aria-haspopup": "dialog"
            }, _toDisplayString(_ctx.$t('footer.keyboard_shortcuts')), 1 /* TEXT */), _createVNode(_component_Modal, {
              ref_key: "modalRef", ref: modalRef,
              modalTitle: _ctx.$t('footer.keyboard_shortcuts'),
              class: "w-auto max-w-lg"
            }, {
              default: _withCtx(() => [
                _createElementVNode("p", _hoisted_2, _toDisplayString(_ctx.$t('shortcuts.section.global')), 1 /* TEXT */),
                _createElementVNode("ul", { class: "mb-6 flex flex-col gap-2" }, [
                  _createElementVNode("li", { class: "flex gap-2 items-center" }, [
                    _hoisted_3,
                    _createElementVNode("span", null, _toDisplayString(_ctx.$t('shortcuts.focus_search')), 1 /* TEXT */)
                  ]),
                  _createElementVNode("li", { class: "flex gap-2 items-center" }, [
                    _hoisted_4,
                    _createElementVNode("span", null, _toDisplayString(_ctx.$t('shortcuts.show_kbd_hints')), 1 /* TEXT */)
                  ]),
                  _createElementVNode("li", { class: "flex gap-2 items-center" }, [
                    _hoisted_5,
                    _createElementVNode("span", null, _toDisplayString(_ctx.$t('shortcuts.settings')), 1 /* TEXT */)
                  ]),
                  _createElementVNode("li", { class: "flex gap-2 items-center" }, [
                    _hoisted_6,
                    _createElementVNode("span", null, _toDisplayString(_ctx.$t('shortcuts.compare')), 1 /* TEXT */)
                  ])
                ]),
                _createElementVNode("p", _hoisted_7, _toDisplayString(_ctx.$t('shortcuts.section.search')), 1 /* TEXT */),
                _createElementVNode("ul", { class: "mb-6 flex flex-col gap-2" }, [
                  _createElementVNode("li", { class: "flex gap-2 items-center" }, [
                    _hoisted_8,
                    _createTextVNode("/"),
                    _hoisted_9,
                    _createElementVNode("span", null, _toDisplayString(_ctx.$t('shortcuts.navigate_results')), 1 /* TEXT */)
                  ]),
                  _createElementVNode("li", { class: "flex gap-2 items-center" }, [
                    _hoisted_10,
                    _createElementVNode("span", null, _toDisplayString(_ctx.$t('shortcuts.go_to_result')), 1 /* TEXT */)
                  ])
                ]),
                _createElementVNode("p", _hoisted_11, _toDisplayString(_ctx.$t('shortcuts.section.package')), 1 /* TEXT */),
                _createElementVNode("ul", { class: "mb-8 flex flex-col gap-2" }, [
                  _createElementVNode("li", { class: "flex gap-2 items-center" }, [
                    _hoisted_12,
                    _createElementVNode("span", null, _toDisplayString(_ctx.$t('shortcuts.open_code_view')), 1 /* TEXT */)
                  ]),
                  _createElementVNode("li", { class: "flex gap-2 items-center" }, [
                    _hoisted_13,
                    _createElementVNode("span", null, _toDisplayString(_ctx.$t('shortcuts.open_docs')), 1 /* TEXT */)
                  ]),
                  _createElementVNode("li", { class: "flex gap-2 items-center" }, [
                    _hoisted_14,
                    _createElementVNode("span", null, _toDisplayString(_ctx.$t('shortcuts.compare_from_package')), 1 /* TEXT */)
                  ])
                ]),
                _createElementVNode("p", { class: "text-fg-muted leading-relaxed" }, [
                  _createVNode(_component_i18n_t, {
                    keypath: "shortcuts.disable_shortcuts",
                    tag: "span",
                    scope: "global"
                  }, {
                    settings: _withCtx(() => [
                      _createVNode(_component_NuxtLink, {
                        to: { name: 'settings' },
                        class: "hover:text-fg underline decoration-fg-subtle/50 hover:decoration-fg",
                        onClick: closeModal
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_ctx.$t('settings.title')), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["to"])
                    ]),
                    _: 1 /* STABLE */
                  })
                ])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modalTitle"]), _createVNode(_component_LinkBase, { to: _unref(NPMX_DOCS_SITE) }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('footer.docs')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to"]), _createVNode(_component_LinkBase, { to: "https://repo.npmx.dev" }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('footer.source')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }), _createVNode(_component_LinkBase, { to: "https://social.npmx.dev" }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('footer.social')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }), _createVNode(_component_LinkBase, { to: "https://chat.npmx.dev" }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('footer.chat')), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }) ]) ]), (!isHome.value) ? (_openBlock(), _createBlock(_component_BuildEnvironment, {
            key: 0,
            footer: ""
          })) : _createCommentVNode("v-if", true), _createElementVNode("p", { class: "text-xs text-fg-muted text-center sm:text-start m-0" }, [ _createElementVNode("span", _hoisted_15, _toDisplayString(_ctx.$t('non_affiliation_disclaimer')), 1 /* TEXT */), _createElementVNode("span", _hoisted_16, _toDisplayString(_ctx.$t('trademark_disclaimer')), 1 /* TEXT */) ]) ]) ]))
}
}

})
