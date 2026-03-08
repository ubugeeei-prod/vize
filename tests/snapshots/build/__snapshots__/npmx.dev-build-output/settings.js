import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "font-mono text-3xl sm:text-4xl font-medium" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:arrow-left rtl-flip w-4 h-4", "aria-hidden": "true" })
const _hoisted_3 = { class: "sr-only sm:not-sr-only" }
const _hoisted_4 = { class: "text-fg-muted text-lg" }
const _hoisted_5 = { class: "text-xs text-fg-muted uppercase tracking-wider mb-4" }
const _hoisted_6 = { for: "theme-select", class: "block text-sm text-fg font-medium" }
const _hoisted_7 = { class: "block text-sm text-fg font-medium" }
const _hoisted_8 = { class: "block text-sm text-fg font-medium" }
const _hoisted_9 = { class: "text-xs text-fg-muted uppercase tracking-wider mb-4" }
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("div", { class: "border-t border-border my-4" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("div", { class: "border-t border-border my-4" })
const _hoisted_12 = { class: "text-xs text-fg-muted uppercase tracking-wider mb-4" }
const _hoisted_13 = { for: "search-provider-select", class: "block text-sm text-fg font-medium" }
const _hoisted_14 = { class: "text-xs text-fg-muted mb-3" }
const _hoisted_15 = { class: "text-xs text-fg-subtle mt-2" }
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:external-link w-3 h-3", "aria-hidden": "true" })
const _hoisted_17 = { class: "text-xs text-fg-muted uppercase tracking-wider mb-4" }
const _hoisted_18 = { for: "language-select", class: "block text-sm text-fg font-medium" }
const _hoisted_19 = /*#__PURE__*/ _createElementVNode("span", { class: "i-simple-icons:github w-4 h-4", "aria-hidden": "true" })
const _hoisted_20 = { class: "text-xs text-fg-muted uppercase tracking-wider mb-4" }

export default /*@__PURE__*/_defineComponent({
  __name: 'settings',
  setup(__props) {

const router = useRouter()
const canGoBack = useCanGoBack()
const { settings } = useSettings()
const { locale, locales, setLocale: setNuxti18nLocale } = useI18n()
const colorMode = useColorMode()
const { currentLocaleStatus, isSourceLocale } = useI18nStatus()
const keyboardShortcutsEnabled = useKeyboardShortcuts()
// Escape to go back (but not when focused on form elements or modal is open)
onKeyStroke(
  e =>
    keyboardShortcutsEnabled.value &&
    isKeyWithoutModifiers(e, 'Escape') &&
    !isEditableElement(e.target) &&
    !document.documentElement.matches('html:has(:modal)'),
  e => {
    e.preventDefault()
    router.back()
  },
  { dedupe: true },
)
useSeoMeta({
  title: () => `${$t('settings.title')} - npmx`,
  ogTitle: () => `${$t('settings.title')} - npmx`,
  twitterTitle: () => `${$t('settings.title')} - npmx`,
  description: () => $t('settings.meta_description'),
  ogDescription: () => $t('settings.meta_description'),
  twitterDescription: () => $t('settings.meta_description'),
})
defineOgImageComponent('Default', {
  title: () => $t('settings.title'),
  description: () => $t('settings.tagline'),
  primaryColor: '#60a5fa',
})
const setLocale: typeof setNuxti18nLocale = locale => {
  settings.value.selectedLocale = locale
  return setNuxti18nLocale(locale)
}

return (_ctx: any,_cache: any) => {
  const _component_SelectField = _resolveComponent("SelectField")
  const _component_SettingsAccentColorPicker = _resolveComponent("SettingsAccentColorPicker")
  const _component_SettingsBgThemePicker = _resolveComponent("SettingsBgThemePicker")
  const _component_SettingsToggle = _resolveComponent("SettingsToggle")
  const _component_ClientOnly = _resolveComponent("ClientOnly")
  const _component_SettingsTranslationHelper = _resolveComponent("SettingsTranslationHelper")

  return (_openBlock(), _createElementBlock("main", { class: "container flex-1 py-12 sm:py-16 w-full" }, [ _createElementVNode("article", { class: "max-w-2xl mx-auto" }, [ _createElementVNode("header", { class: "mb-12" }, [ _createElementVNode("div", { class: "flex items-baseline justify-between gap-4 mb-4" }, [ _createElementVNode("h1", _hoisted_1, _toDisplayString(_ctx.$t('settings.title')), 1 /* TEXT */), (_unref(canGoBack)) ? (_openBlock(), _createElementBlock("button", {
                key: 0,
                type: "button",
                class: "cursor-pointer inline-flex items-center gap-2 font-mono text-sm text-fg-muted hover:text-fg transition-colors duration-200 rounded focus-visible:outline-accent/70 shrink-0 p-1.5 -mx-1.5",
                onClick: _cache[0] || (_cache[0] = ($event: any) => (_unref(router).back()))
              }, [ _hoisted_2, _createElementVNode("span", _hoisted_3, _toDisplayString(_ctx.$t('nav.back')), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true) ]), _createElementVNode("p", _hoisted_4, _toDisplayString(_ctx.$t('settings.tagline')), 1 /* TEXT */) ]), _createTextVNode("\n\n      " + "\n      "), _createElementVNode("div", { class: "space-y-8" }, [ _createElementVNode("section", null, [ _createElementVNode("h2", _hoisted_5, _toDisplayString(_ctx.$t('settings.sections.appearance')), 1 /* TEXT */), _createElementVNode("div", { class: "bg-bg-subtle border border-border rounded-lg p-4 sm:p-6 space-y-6" }, [ _createElementVNode("div", { class: "space-y-2" }, [ _createElementVNode("label", _hoisted_6, _toDisplayString(_ctx.$t('settings.theme')), 1 /* TEXT */), _createVNode(_component_SelectField, {
                  id: "theme-select",
                  block: "",
                  size: "sm",
                  class: "max-w-48",
                  items: [
                    { label: _ctx.$t('settings.theme_system'), value: 'system' },
                    { label: _ctx.$t('settings.theme_light'), value: 'light' },
                    { label: _ctx.$t('settings.theme_dark'), value: 'dark' },
                  ],
                  modelValue: _unref(colorMode).preference,
                  "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((_unref(colorMode).preference) = $event))
                }, null, 8 /* PROPS */, ["items", "modelValue"]) ]), _createTextVNode("\n\n            " + "\n            "), _createElementVNode("div", { class: "space-y-3" }, [ _createElementVNode("span", _hoisted_7, _toDisplayString(_ctx.$t('settings.accent_colors')), 1 /* TEXT */), _createVNode(_component_SettingsAccentColorPicker) ]), _createTextVNode("\n\n            " + "\n            "), _createElementVNode("div", { class: "space-y-3" }, [ _createElementVNode("span", _hoisted_8, _toDisplayString(_ctx.$t('settings.background_themes')), 1 /* TEXT */), _createVNode(_component_SettingsBgThemePicker) ]) ]) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("section", null, [ _createElementVNode("h2", _hoisted_9, _toDisplayString(_ctx.$t('settings.sections.display')), 1 /* TEXT */), _createElementVNode("div", { class: "bg-bg-subtle border border-border rounded-lg p-4 sm:p-6" }, [ _createVNode(_component_SettingsToggle, {
                label: _ctx.$t('settings.relative_dates'),
                modelValue: _unref(settings).relativeDates,
                "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((_unref(settings).relativeDates) = $event))
              }, null, 8 /* PROPS */, ["label", "modelValue"]), _createTextVNode("\n\n            " + "\n            "), _hoisted_10, _createTextVNode("\n\n            " + "\n            "), _createVNode(_component_SettingsToggle, {
                label: _ctx.$t('settings.include_types'),
                description: _ctx.$t('settings.include_types_description'),
                modelValue: _unref(settings).includeTypesInInstall,
                "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((_unref(settings).includeTypesInInstall) = $event))
              }, null, 8 /* PROPS */, ["label", "description", "modelValue"]), _createTextVNode("\n\n            " + "\n            "), _hoisted_11, _createTextVNode("\n\n            " + "\n            "), _createVNode(_component_SettingsToggle, {
                label: _ctx.$t('settings.hide_platform_packages'),
                description: _ctx.$t('settings.hide_platform_packages_description'),
                modelValue: _unref(settings).hidePlatformPackages,
                "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((_unref(settings).hidePlatformPackages) = $event))
              }, null, 8 /* PROPS */, ["label", "description", "modelValue"]) ]) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("section", null, [ _createElementVNode("h2", _hoisted_12, _toDisplayString(_ctx.$t('settings.sections.search')), 1 /* TEXT */), _createElementVNode("div", { class: "bg-bg-subtle border border-border rounded-lg p-4 sm:p-6" }, [ _createElementVNode("div", { class: "space-y-2" }, [ _createElementVNode("label", _hoisted_13, _toDisplayString(_ctx.$t('settings.data_source.label')), 1 /* TEXT */), _createElementVNode("p", _hoisted_14, _toDisplayString(_ctx.$t('settings.data_source.description')), 1 /* TEXT */), _createVNode(_component_ClientOnly, null, {
                  fallback: _withCtx(() => [
                    _createVNode(_component_SelectField, {
                      id: "search-provider-select",
                      disabled: "",
                      items: [{ label: _ctx.$t('common.loading'), value: 'loading' }],
                      block: "",
                      size: "sm",
                      class: "max-w-48"
                    }, null, 8 /* PROPS */, ["items"])
                  ]),
                  default: _withCtx(() => [
                    _createVNode(_component_SelectField, {
                      id: "search-provider-select",
                      items: [
                      { label: _ctx.$t('settings.data_source.npm'), value: 'npm' },
                      { label: _ctx.$t('settings.data_source.algolia'), value: 'algolia' },
                    ],
                      block: "",
                      size: "sm",
                      class: "max-w-48",
                      modelValue: _unref(settings).searchProvider,
                      "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((_unref(settings).searchProvider) = $event))
                    }, null, 8 /* PROPS */, ["items", "modelValue"])
                  ]),
                  _: 1 /* STABLE */
                }), _createTextVNode("\n\n              " + "\n              "), _createElementVNode("p", _hoisted_15, _toDisplayString(_unref(settings).searchProvider === 'algolia' ? _ctx.$t('settings.data_source.algolia_description') : _ctx.$t('settings.data_source.npm_description')), 1 /* TEXT */), _createTextVNode("\n\n              " + "\n              "), (_unref(settings).searchProvider === 'algolia') ? (_openBlock(), _createElementBlock("a", {
                    key: 0,
                    href: "https://www.algolia.com/developers",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "inline-flex items-center gap-1 text-xs text-fg-subtle hover:text-fg-muted transition-colors mt-2"
                  }, [ _toDisplayString(_ctx.$t('search.algolia_disclaimer')), _createTextVNode("\n                "), _hoisted_16 ])) : _createCommentVNode("v-if", true) ]) ]) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("section", null, [ _createElementVNode("h2", _hoisted_17, _toDisplayString(_ctx.$t('settings.sections.language')), 1 /* TEXT */), _createElementVNode("div", { class: "bg-bg-subtle border border-border rounded-lg p-4 sm:p-6 space-y-4" }, [ _createElementVNode("div", { class: "space-y-2" }, [ _createElementVNode("label", _hoisted_18, _toDisplayString(_ctx.$t('settings.language')), 1 /* TEXT */), _createVNode(_component_ClientOnly, null, {
                  fallback: _withCtx(() => [
                    _createVNode(_component_SelectField, {
                      id: "language-select",
                      disabled: "",
                      items: [{ label: _ctx.$t('common.loading'), value: 'loading' }],
                      block: "",
                      size: "sm",
                      class: "max-w-48"
                    }, null, 8 /* PROPS */, ["items"])
                  ]),
                  default: _withCtx(() => [
                    _createVNode(_component_SelectField, {
                      id: "language-select",
                      items: _unref(locales).map((loc) => ({
  	label: loc.name ?? "",
  	value: loc.code
  })),
                      "onUpdate:modelValue": [($event: any) => (setLocale($event)), ($event: any) => ((locale).value = $event)],
                      block: "",
                      size: "sm",
                      class: "max-w-48",
                      modelValue: _unref(locale)
                    }, null, 8 /* PROPS */, ["items", "modelValue"])
                  ]),
                  _: 1 /* STABLE */
                }) ]), _createTextVNode("\n\n            " + "\n            "), (_unref(currentLocaleStatus) && !_unref(isSourceLocale)) ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: "border-t border-border pt-4"
                }, [ _createVNode(_component_SettingsTranslationHelper, { status: _unref(currentLocaleStatus) }, null, 8 /* PROPS */, ["status"]) ])) : (_openBlock(), _createElementBlock("a", {
                  key: 1,
                  href: "https://github.com/npmx-dev/npmx.dev/tree/main/i18n/locales",
                  target: "_blank",
                  rel: "noopener noreferrer",
                  class: "inline-flex items-center gap-2 text-sm text-fg-muted hover:text-fg transition-colors duration-200 focus-visible:outline-accent/70 rounded"
                }, [ _hoisted_19, _createTextVNode("\n                "), _toDisplayString(_ctx.$t('settings.help_translate')) ])), _createTextVNode("\n\n            " + "\n            ") ]) ]), _createTextVNode("\n\n        " + "\n        "), _createElementVNode("section", null, [ _createElementVNode("h2", _hoisted_20, _toDisplayString(_ctx.$t('settings.sections.keyboard_shortcuts')), 1 /* TEXT */), _createElementVNode("div", { class: "bg-bg-subtle border border-border rounded-lg p-4 sm:p-6" }, [ _createVNode(_component_SettingsToggle, {
                label: _ctx.$t('settings.keyboard_shortcuts_enabled'),
                description: _ctx.$t('settings.keyboard_shortcuts_enabled_description'),
                modelValue: _unref(settings).keyboardShortcuts,
                "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((_unref(settings).keyboardShortcuts) = $event))
              }, null, 8 /* PROPS */, ["label", "description", "modelValue"]) ]) ]) ]) ]) ]))
}
}

})
