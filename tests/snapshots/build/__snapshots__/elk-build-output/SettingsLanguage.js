import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, unref as _unref, vModelSelect as _vModelSelect } from "vue"

import type { LocaleObject } from '@nuxtjs/i18n'
import type { ComputedRef } from 'vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'SettingsLanguage',
  setup(__props) {

const userSettings = useUserSettings()
const { locales } = useI18n() as { locales: ComputedRef<LocaleObject[]> }

return (_ctx: any,_cache: any) => {
  return _withDirectives((_openBlock(), _createElementBlock("select", {
      "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((_unref(userSettings).language) = $event))
    }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(locales), (item) => {
        return (_openBlock(), _createElementBlock("option", {
          key: item.code,
          value: item.code,
          selected: _unref(userSettings).language === item.code
        }, _toDisplayString(item.name), 9 /* TEXT, PROPS */, ["value", "selected"]))
      }), 128 /* KEYED_FRAGMENT */)) ], 512 /* NEED_PATCH */)), [ [_vModelSelect, _unref(userSettings).language] ])
}
}

})
