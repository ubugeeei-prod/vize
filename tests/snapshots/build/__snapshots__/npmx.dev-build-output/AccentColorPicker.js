import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, unref as _unref } from "vue"


const _hoisted_1 = { class: "sr-only" }
import { useAccentColor } from '~/composables/useSettings'

export default /*@__PURE__*/_defineComponent({
  __name: 'AccentColorPicker',
  setup(__props) {

const { accentColors, selectedAccentColor, setAccentColor } = useAccentColor()
onPrehydrate(el => {
  const settings = JSON.parse(localStorage.getItem('npmx-settings') || '{}')
  const defaultId = 'sky'
  const id = settings.accentColorId
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
  return (_openBlock(), _createElementBlock("fieldset", { class: "flex items-center gap-4 has-[input:focus-visible]:(outline-solid outline-accent/70 outline-offset-4) rounded-xl w-fit" }, [ _createElementVNode("legend", _hoisted_1, _toDisplayString(_ctx.$t('settings.accent_colors')), 1 /* TEXT */), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(accentColors), (color) => {
        return (_openBlock(), _createElementBlock("label", {
          key: color.id,
          class: _normalizeClass(["size-6 rounded-full transition-transform duration-150 motion-safe:hover:scale-110 has-[:checked]:(ring-2 ring-fg ring-offset-2 ring-offset-bg-subtle) has-[:focus-visible]:(ring-2 ring-fg ring-offset-2 ring-offset-bg-subtle)", color.id === 'neutral' ? 'flex items-center justify-center bg-fg' : '']),
          style: _normalizeStyle({ backgroundColor: `var(--swatch-${color.id})` })
        }, [
          _createElementVNode("input", {
            type: "radio",
            name: "accent-color",
            class: "sr-only",
            value: color.id,
            checked: _unref(selectedAccentColor) === color.id || (!_unref(selectedAccentColor) && color.id === 'sky'),
            "aria-label": color.id === 'neutral' ? _ctx.$t('settings.clear_accent') : color.name,
            onChange: _cache[0] || (_cache[0] = ($event: any) => (_unref(setAccentColor)(color.id)))
          }, null, 40 /* PROPS, NEED_HYDRATION */, ["value", "checked", "aria-label"]),
          (color.id === 'neutral')
            ? (_openBlock(), _createElementBlock("span", {
              key: 0,
              class: "i-lucide:ban size-4 text-bg",
              "aria-hidden": "true"
            }))
            : _createCommentVNode("v-if", true)
        ], 6 /* CLASS, STYLE */))
      }), 128 /* KEYED_FRAGMENT */)) ]))
}
}

})
