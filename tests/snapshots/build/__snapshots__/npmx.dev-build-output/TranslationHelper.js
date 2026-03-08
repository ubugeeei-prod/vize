import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeStyle as _normalizeStyle, unref as _unref } from "vue"


const _hoisted_1 = { class: "tabular-nums" }
const _hoisted_2 = { class: "text-xs text-fg-muted font-medium" }
const _hoisted_3 = { class: "text-xs text-fg-muted" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:pen w-3.5 h-3.5", "aria-hidden": "true" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:file-text w-3.5 h-3.5", "aria-hidden": "true" })
import type { I18nLocaleStatus } from '#shared/types'
const INITIAL_SHOW_COUNT = 5

export default /*@__PURE__*/_defineComponent({
  __name: 'TranslationHelper',
  props: {
    status: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
// Show first N missing keys by default
const showAll = shallowRef(false)
const missingKeysToShow = computed(() => {
  if (showAll.value || props.status.missingKeys.length <= INITIAL_SHOW_COUNT) {
    return props.status.missingKeys
  }
  return props.status.missingKeys.slice(0, INITIAL_SHOW_COUNT)
})
const hasMoreKeys = computed(
  () => props.status.missingKeys.length > INITIAL_SHOW_COUNT && !showAll.value,
)
const remainingCount = computed(() => props.status.missingKeys.length - INITIAL_SHOW_COUNT)
// Generate a GitHub URL that pre-fills the edit with guidance
const contributionGuideUrl =
  'https://github.com/npmx-dev/npmx.dev/blob/main/CONTRIBUTING.md#localization-i18n'
// Copy missing keys as JSON template to clipboard
const { copy, copied } = useClipboard()
const numberFormatter = useNumberFormatter()
const percentageFormatter = useNumberFormatter({ style: 'percent' })
function copyMissingKeysTemplate() {
  // Create a template showing what needs to be added
  const template = props.status.missingKeys.map(key => `  "${key}": ""`).join(',\n')
  const fullTemplate = `// Missing translations for ${props.status.label} (${props.status.lang})
// Add these keys to: i18n/locales/${props.status.lang}.json
${template}`
  copy(fullTemplate)
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "space-y-3" }, [ _createElementVNode("div", { class: "space-y-1.5" }, [ _createElementVNode("div", { class: "flex items-center justify-between text-xs text-fg-muted" }, [ _createElementVNode("span", null, _toDisplayString(_ctx.$t('settings.translation_progress')), 1 /* TEXT */), _createElementVNode("span", _hoisted_1, _toDisplayString(_unref(numberFormatter).format(__props.status.completedKeys)) + "/" + _toDisplayString(_unref(numberFormatter).format(__props.status.totalKeys)) + "\n          (" + _toDisplayString(_unref(percentageFormatter).format(__props.status.percentComplete / 100)) + ")", 1 /* TEXT */) ]), _createElementVNode("div", { class: "h-1.5 bg-bg rounded-full overflow-hidden" }, [ _createElementVNode("div", {
            class: "h-full bg-accent transition-all duration-300 motion-reduce:transition-none",
            style: _normalizeStyle({ width: `${__props.status.percentComplete}%` })
          }, null, 4 /* STYLE */) ]) ]), _createTextVNode("\n\n    " + "\n    "), (__props.status.missingKeys.length > 0) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "space-y-2"
        }, [ _createElementVNode("div", { class: "flex items-center justify-between" }, [ _createElementVNode("h4", _hoisted_2, _toDisplayString(_ctx.$t( 'i18n.missing_keys', { count: _unref(numberFormatter).format(__props.status.missingKeys.length) }, __props.status.missingKeys.length, )), 1 /* TEXT */), _createElementVNode("button", {
              type: "button",
              class: "text-xs text-accent hover:underline rounded focus-visible:outline-accent/70",
              onClick: copyMissingKeysTemplate
            }, _toDisplayString(_unref(copied) ? _ctx.$t('common.copied') : _ctx.$t('i18n.copy_keys')), 1 /* TEXT */) ]), _createElementVNode("ul", { class: "space-y-1 text-xs font-mono bg-bg rounded-md p-2 max-h-32 overflow-y-auto" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(missingKeysToShow.value, (key) => {
              return (_openBlock(), _createElementBlock("li", {
                key: key,
                class: "text-fg-muted truncate",
                title: key
              }, _toDisplayString(key), 9 /* TEXT, PROPS */, ["title"]))
            }), 128 /* KEYED_FRAGMENT */)) ]), (hasMoreKeys.value) ? (_openBlock(), _createElementBlock("button", {
              key: 0,
              type: "button",
              class: "text-xs text-fg-muted hover:text-fg rounded focus-visible:outline-accent/70",
              onClick: _cache[0] || (_cache[0] = ($event: any) => (showAll.value = true))
            }, _toDisplayString(_ctx.$t( 'i18n.show_more_keys', { count: _unref(numberFormatter).format(remainingCount.value) }, remainingCount.value, )), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    " + "\n    "), _createElementVNode("div", { class: "pt-2 border-t border-border space-y-2" }, [ _createElementVNode("p", _hoisted_3, _toDisplayString(_ctx.$t('i18n.contribute_hint')), 1 /* TEXT */), _createElementVNode("div", { class: "flex flex-wrap gap-2" }, [ _createElementVNode("a", {
            href: __props.status.githubEditUrl,
            target: "_blank",
            rel: "noopener noreferrer",
            class: "inline-flex items-center gap-1.5 px-2.5 py-1.5 text-xs bg-bg hover:bg-bg-subtle border border-border rounded-md transition-colors focus-visible:outline-accent/70"
          }, [ _hoisted_4, _createTextVNode("\n          " + _toDisplayString(_ctx.$t('i18n.edit_on_github')), 1 /* TEXT */) ], 8 /* PROPS */, ["href"]), _createElementVNode("a", {
            href: contributionGuideUrl,
            target: "_blank",
            rel: "noopener noreferrer",
            class: "inline-flex items-center gap-1.5 px-2.5 py-1.5 text-xs text-fg-muted hover:text-fg rounded transition-colors focus-visible:outline-accent/70"
          }, [ _hoisted_5, _createTextVNode("\n          " + _toDisplayString(_ctx.$t('i18n.view_guide')), 1 /* TEXT */) ], 8 /* PROPS */, ["href"]) ]) ]) ]))
}
}

})
