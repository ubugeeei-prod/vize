import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeStyle as _normalizeStyle, unref as _unref } from "vue"


const _hoisted_1 = { class: "sr-only" }

export default /*@__PURE__*/_defineComponent({
  __name: 'BgThemePicker',
  setup(__props) {

const { backgroundThemes, selectedBackgroundTheme, setBackgroundTheme } = useBackgroundTheme()
onPrehydrate(el => {
  const settings = JSON.parse(localStorage.getItem('npmx-settings') || '{}')
  const defaultId = 'neutral'
  const id = settings.preferredBackgroundTheme
  if (id) {
    const input = el.querySelector<HTMLInputElement>(`input[value="${id}"]`)
    if (input) {
      input.checked = true
      input.setAttribute('checked', '')
    }
    // Remove checked from the server-default (clear button, value="")
    if (id !== defaultId) {
      const clearInput = el.querySelector<HTMLInputElement>(`input[value="${defaultId}"]`)
      if (clearInput) {
        clearInput.checked = false
        clearInput.removeAttribute('checked')
      }
    }
  }
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("fieldset", { class: "flex items-center gap-4 has-[input:focus-visible]:(outline-solid outline-accent/70 outline-offset-4) rounded-xl w-fit" }, [ _createElementVNode("legend", _hoisted_1, _toDisplayString(_ctx.$t('settings.background_themes')), 1 /* TEXT */), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(backgroundThemes), (theme) => {
        return (_openBlock(), _createElementBlock("label", {
          key: theme.id,
          class: "size-6 rounded-full transition-transform duration-150 motion-safe:hover:scale-110 has-[:checked]:(ring-2 ring-fg ring-offset-2 ring-offset-bg-subtle) has-[:focus-visible]:(ring-2 ring-fg ring-offset-2 ring-offset-bg-subtle)",
          style: _normalizeStyle({ backgroundColor: theme.value })
        }, [
          _createElementVNode("input", {
            type: "radio",
            name: "background-theme",
            class: "sr-only",
            value: theme.id,
            checked: 
            _unref(selectedBackgroundTheme) === theme.id ||
            (!_unref(selectedBackgroundTheme) && theme.id === 'neutral')
          ,
            "aria-label": theme.name,
            onChange: _cache[0] || (_cache[0] = ($event: any) => (_unref(setBackgroundTheme)(theme.id)))
          }, null, 40 /* PROPS, NEED_HYDRATION */, ["value", "checked", "aria-label"])
        ], 4 /* STYLE */))
      }), 128 /* KEYED_FRAGMENT */)) ]))
}
}

})
