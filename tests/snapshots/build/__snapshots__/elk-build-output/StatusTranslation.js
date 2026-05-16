import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { block: "true", "i-ri:loader-2-fill": "true" })
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusTranslation',
  props: {
    status: { type: null, required: true }
  },
  async setup(__props: any) {

let __temp: any, __restore: any

const {
  toggle: _toggleTranslation,
  translation,
  enabled: isTranslationEnabled,
} = await useTranslation(__props.status, getLanguageCode())
const preferenceHideTranslation = usePreferences('hideTranslation')
const showButton = computed(() =>
  !preferenceHideTranslation.value
  && isTranslationEnabled
  && __props.status.content.trim().length,
)
const translating = ref(false)
async function toggleTranslation() {
  translating.value = true
  try {
;(
  ([__temp,__restore] = _withAsyncContext(() => _toggleTranslation())),
  await __temp,
  __restore()
)
  }
  finally {
    translating.value = false
  }
}

return (_ctx: any,_cache: any) => {
  return (showButton.value)
      ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createElementVNode("button", {
          "p-0": "",
          flex: "~ center",
          "gap-2": "",
          "text-sm": "",
          disabled: translating.value,
          "disabled-bg-transparent": "",
          "btn-text": "",
          class: "disabled-text-$c-text-btn-disabled-deeper",
          onClick: toggleTranslation
        }, [ (translating.value) ? (_openBlock(), _createElementBlock("span", {
              key: 0,
              block: "",
              "animate-spin": "",
              "preserve-3d": ""
            }, [ _hoisted_1 ])) : (_openBlock(), _createElementBlock("div", {
              key: 1,
              "i-ri:translate": ""
            })), _createTextVNode("\n      " + _toDisplayString(_unref(translation).visible ? _ctx.$t('menu.show_untranslated') : _ctx.$t('menu.translate_post')), 1 /* TEXT */) ], 8 /* PROPS */, ["disabled"]) ]))
      : _createCommentVNode("v-if", true)
}
}

})
