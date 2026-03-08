import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:shield-check w-4 h-4 shrink-0 text-emerald-500 mt-0.5", "aria-hidden": "true" })
const _hoisted_2 = { class: "font-mono text-xs text-fg-muted m-0" }
const _hoisted_3 = { class: "font-mono text-xs text-fg-muted m-0" }
const _hoisted_4 = { class: "font-mono text-xs text-fg-muted m-0" }
import type { ProvenanceDetails } from '#shared/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'PackageProvenanceSection',
  props: {
    details: { type: null, required: true }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  const _component_LinkBase = _resolveComponent("LinkBase")
  const _component_i18n_t = _resolveComponent("i18n-t")

  return (_openBlock(), _createElementBlock("section", {
      id: "provenance",
      "aria-labelledby": "provenance-heading",
      class: "scroll-mt-20"
    }, [ _createElementVNode("h2", {
        id: "provenance-heading",
        class: "group text-xs text-fg-subtle uppercase tracking-wider mb-3"
      }, [ _createVNode(_component_LinkBase, { to: "#provenance" }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_ctx.$t('package.provenance_section.title')), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }) ]), _createElementVNode("div", { class: "space-y-3 border border-border rounded-lg p-4 sm:p-5" }, [ _createElementVNode("div", { class: "space-y-1" }, [ _createElementVNode("p", { class: "flex items-start gap-2 text-sm text-fg m-0" }, [ _hoisted_1, _createVNode(_component_i18n_t, {
              keypath: "package.provenance_section.built_and_signed_on",
              tag: "span"
            }, {
              provider: _withCtx(() => [
                _createElementVNode("strong", null, _toDisplayString(__props.details.providerLabel), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }) ]), (__props.details.buildSummaryUrl) ? (_openBlock(), _createElementBlock("a", {
              key: 0,
              href: __props.details.buildSummaryUrl,
              target: "_blank",
              rel: "noopener noreferrer",
              class: "link text-sm text-fg-muted inline-flex"
            }, _toDisplayString(_ctx.$t('package.provenance_section.view_build_summary')), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]), _createElementVNode("dl", { class: "m-0 mt-4 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3" }, [ (__props.details.sourceCommitUrl) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "min-w-0 flex flex-col gap-0.5"
            }, [ _createElementVNode("dt", _hoisted_2, _toDisplayString(_ctx.$t('package.provenance_section.source_commit')), 1 /* TEXT */), _createElementVNode("dd", { class: "m-0 min-w-0" }, [ _createElementVNode("a", {
                  href: __props.details.sourceCommitUrl,
                  target: "_blank",
                  rel: "noopener noreferrer",
                  class: "link font-mono text-sm block min-w-0 truncate",
                  title: __props.details.sourceCommitSha ?? __props.details.sourceCommitUrl
                }, _toDisplayString(__props.details.sourceCommitSha ? `${__props.details.sourceCommitSha.slice(0, 12)}` : __props.details.sourceCommitUrl), 9 /* TEXT, PROPS */, ["href", "title"]) ]) ])) : _createCommentVNode("v-if", true), (__props.details.buildFileUrl) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "min-w-0 flex flex-col gap-0.5"
            }, [ _createElementVNode("dt", _hoisted_3, _toDisplayString(_ctx.$t('package.provenance_section.build_file')), 1 /* TEXT */), _createElementVNode("dd", { class: "m-0 min-w-0" }, [ _createElementVNode("a", {
                  href: __props.details.buildFileUrl,
                  target: "_blank",
                  rel: "noopener noreferrer",
                  class: "link font-mono text-sm block min-w-0 break-words",
                  title: __props.details.buildFilePath ?? __props.details.buildFileUrl
                }, _toDisplayString(__props.details.buildFilePath ?? __props.details.buildFileUrl), 9 /* TEXT, PROPS */, ["href", "title"]) ]) ])) : _createCommentVNode("v-if", true), (__props.details.publicLedgerUrl) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "min-w-0 flex flex-col gap-0.5"
            }, [ _createElementVNode("dt", _hoisted_4, _toDisplayString(_ctx.$t('package.provenance_section.public_ledger')), 1 /* TEXT */), _createElementVNode("dd", { class: "m-0 min-w-0" }, [ _createElementVNode("a", {
                  href: __props.details.publicLedgerUrl,
                  target: "_blank",
                  rel: "noopener noreferrer",
                  class: "link text-sm inline-flex"
                }, _toDisplayString(_ctx.$t('package.provenance_section.transparency_log_entry')), 9 /* TEXT, PROPS */, ["href"]) ]) ])) : _createCommentVNode("v-if", true) ]) ]) ]))
}
}

})
