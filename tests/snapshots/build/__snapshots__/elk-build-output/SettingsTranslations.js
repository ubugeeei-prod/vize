import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, unref as _unref, vModelSelect as _vModelSelect, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = { class: " mb-2" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "block i-ri:close-line", "aria-hidden": "true" })
import ISO6391 from 'iso-639-1'

export default /*@__PURE__*/_defineComponent({
  __name: 'SettingsTranslations',
  setup(__props) {

const supportedTranslationLanguages = ISO6391.getLanguages([...supportedTranslationCodes])
const userSettings = useUserSettings()
const language = ref<string | null>(null)
const availableOptions = computed(() => {
  return Object.values(supportedTranslationLanguages).filter((value) => {
    return !userSettings.value.disabledTranslationLanguages.includes(value.code)
  })
})
function addDisabledTranslation() {
  if (language.value) {
    const uniqueValues = new Set(userSettings.value.disabledTranslationLanguages)
    uniqueValues.add(language.value)
    userSettings.value.disabledTranslationLanguages = [...uniqueValues]
    language.value = null
  }
}
function removeDisabledTranslation(code: string) {
  const uniqueValues = new Set(userSettings.value.disabledTranslationLanguages)
  uniqueValues.delete(code)
  userSettings.value.disabledTranslationLanguages = [...uniqueValues]
}

return (_ctx: any,_cache: any) => {
  const _component_CommonCheckbox = _resolveComponent("CommonCheckbox")

  return (_openBlock(), _createElementBlock("div", null, [ _createVNode(_component_CommonCheckbox, {
        label: _ctx.$t('settings.preferences.hide_translation'),
        modelValue: _unref(userSettings).preferences.hideTranslation,
        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((_unref(userSettings).preferences.hideTranslation) = $event))
      }, null, 8 /* PROPS */, ["label", "modelValue"]), (!_unref(userSettings).preferences.hideTranslation) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "mt-1 ms-2"
        }, [ _createElementVNode("p", _hoisted_1, _toDisplayString(_ctx.$t('settings.language.translations.hide_specific')), 1 /* TEXT */), _createElementVNode("div", { class: "ms-4" }, [ _createElementVNode("ul", null, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(userSettings).disabledTranslationLanguages, (langCode) => {
                return (_openBlock(), _createElementBlock("li", {
                  key: langCode,
                  class: "flex items-center"
                }, [
                  _createElementVNode("div", null, _toDisplayString(ISO6391.getNativeName(langCode)), 1 /* TEXT */),
                  _createElementVNode("button", {
                    class: "btn-text",
                    type: "button",
                    title: _ctx.$t('settings.language.translations.remove'),
                    onClick: _cache[1] || (_cache[1] = _withModifiers(($event: any) => (removeDisabledTranslation(langCode)), ["prevent"]))
                  }, [
                    _hoisted_2
                  ], 8 /* PROPS */, ["title"])
                ]))
              }), 128 /* KEYED_FRAGMENT */)) ]), _createElementVNode("div", { class: "flex items-center mt-2" }, [ _withDirectives(_createElementVNode("select", {
                "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((language).value = $event)),
                class: "select-settings"
              }, [ _createElementVNode("option", {
                  disabled: "",
                  selected: "",
                  value: null
                }, _toDisplayString(_ctx.$t('settings.language.translations.choose_language')), 9 /* TEXT, PROPS */, ["value"]), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(availableOptions.value, (availableOption) => {
                  return (_openBlock(), _createElementBlock("option", {
                    key: availableOption.code,
                    value: availableOption.code
                  }, _toDisplayString(availableOption.nativeName), 9 /* TEXT, PROPS */, ["value"]))
                }), 128 /* KEYED_FRAGMENT */)) ], 512 /* NEED_PATCH */), [ [_vModelSelect, language.value] ]), _createElementVNode("button", {
                class: "btn-text shrink-0",
                onClick: addDisabledTranslation
              }, _toDisplayString(_ctx.$t('settings.language.translations.add')), 1 /* TEXT */) ]) ]) ])) : _createCommentVNode("v-if", true) ]))
}
}

})
