import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, unref as _unref, vModelText as _vModelText } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("hr", { class: "border-base " })
import Fuse from 'fuse.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'PublishLanguagePicker',
  props: {
    "modelValue": { required: true },
    "modelModifiers": {},
  },
  emits: ["update:modelValue"],
  setup(__props) {

const modelValue = _useModel(__props, "modelValue")
const { t } = useI18n()
const userSettings = useUserSettings()
const languageKeyword = ref('')
const fuse = new Fuse(languagesNameList, {
  keys: ['code', 'nativeName', 'name'],
  shouldSort: true,
})
const languages = computed(() =>
  languageKeyword.value.trim()
    ? fuse.search(languageKeyword.value).map(r => r.item)
    : [...languagesNameList].filter(entry => !userSettings.value.disabledTranslationLanguages.includes(entry.code)).sort(({ code: a }, { code: b }) => {
        // Put English on the top
        if (a === 'en')
          return -1
        return a === modelValue.value ? -1 : b === modelValue.value ? 1 : a.localeCompare(b)
      }),
)
const preferredLanguages = computed(() => {
  const result = []
  for (const langCode of userSettings.value.disabledTranslationLanguages) {
    const completeLang = languagesNameList.find(listEntry => listEntry.code === langCode)
    if (completeLang)
      result.push(completeLang)
  }
  return result
},
)
function chooseLanguage(language: string) {
  modelValue.value = language
}

return (_ctx: any,_cache: any) => {
  const _component_CommonDropdownItem = _resolveComponent("CommonDropdownItem")

  return (_openBlock(), _createElementBlock("div", {
      relative: "",
      "of-x-hidden": ""
    }, [ _createElementVNode("div", { p2: "" }, [ _withDirectives(_createElementVNode("input", {
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((languageKeyword).value = $event)),
          placeholder: _unref(t)('language.search'),
          p2: "",
          "border-rounded": "",
          "w-full": "",
          "bg-transparent": "",
          "outline-none": "",
          border: "~ base"
        }, null, 8 /* PROPS */, ["placeholder"]), [ [_vModelText, languageKeyword.value] ]) ]), _createElementVNode("div", {
        "max-h-40vh": "",
        "overflow-auto": ""
      }, [ (!languageKeyword.value.trim()) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(preferredLanguages.value, ({ code, nativeName, name }) => {
              return (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                key: code,
                text: nativeName,
                description: name,
                checked: code === modelValue.value,
                onClick: _cache[1] || (_cache[1] = ($event: any) => (chooseLanguage(_ctx.code)))
              }, null, 8 /* PROPS */, ["text", "description", "checked"]))
            }), 128 /* KEYED_FRAGMENT */)), _hoisted_1 ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(languages.value, ({ code, nativeName, name }) => {
          return (_openBlock(), _createBlock(_component_CommonDropdownItem, {
            key: code,
            text: nativeName,
            description: name,
            checked: code === modelValue.value,
            onClick: _cache[2] || (_cache[2] = ($event: any) => (chooseLanguage(_ctx.code)))
          }, null, 8 /* PROPS */, ["text", "description", "checked"]))
        }), 128 /* KEYED_FRAGMENT */)) ]) ]))
}
}

})
