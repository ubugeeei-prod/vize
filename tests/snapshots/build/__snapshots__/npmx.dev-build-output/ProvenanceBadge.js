import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'ProvenanceBadge',
  props: {
    provider: { type: String, required: false },
    packageName: { type: String, required: false },
    version: { type: String, required: false },
    compact: { type: Boolean, required: false },
    linked: { type: Boolean, required: false }
  },
  setup(__props: any) {

const props = __props
const providerLabels: Record<string, string> = {
  github: 'GitHub Actions',
  gitlab: 'GitLab CI',
}
const title = computed(() =>
  props.provider
    ? $t('badges.provenance.verified_via', {
        provider: providerLabels[props.provider] ?? props.provider,
      })
    : $t('badges.provenance.verified_title'),
)

return (_ctx: any,_cache: any) => {
  return (__props.packageName && __props.version && __props.linked !== false)
      ? (_openBlock(), _createElementBlock("a", {
        key: 0,
        href: `https://www.npmjs.com/package/${__props.packageName}/v/${__props.version}#provenance`,
        target: "_blank",
        rel: "noopener noreferrer",
        class: "inline-flex items-center justify-center gap-1 text-xs font-mono text-fg-muted hover:text-fg transition-colors duration-200 min-w-6 min-h-6",
        title: title.value
      }, [ _createElementVNode("span", {
          class: _normalizeClass(["i-lucide:shield-check shrink-0", __props.compact ? 'w-3.5 h-3.5' : 'w-4 h-4'])
        }, null, 2 /* CLASS */), (!__props.compact) ? (_openBlock(), _createElementBlock("span", {
            key: 0,
            class: "sr-only sm:not-sr-only"
          }, _toDisplayString(_ctx.$t('badges.provenance.verified')), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]))
      : (_openBlock(), _createElementBlock("span", {
        key: 1,
        class: "inline-flex items-center gap-1 text-xs font-mono text-fg-muted",
        title: title.value
      }, [ _createElementVNode("span", {
          class: _normalizeClass(["i-lucide:shield-check shrink-0", __props.compact ? 'w-3.5 h-3.5' : 'w-4 h-4'])
        }, null, 2 /* CLASS */), (!__props.compact) ? (_openBlock(), _createElementBlock("span", {
            key: 0,
            class: "sr-only sm:not-sr-only"
          }, _toDisplayString(_ctx.$t('badges.provenance.verified')), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]))
}
}

})
