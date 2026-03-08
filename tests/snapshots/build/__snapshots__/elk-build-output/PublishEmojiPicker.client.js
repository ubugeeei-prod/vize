import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, renderSlot as _renderSlot, withCtx as _withCtx } from "vue"

import type { Picker } from 'emoji-mart'
import importEmojiLang from 'virtual:emoji-mart-lang-importer'

export default /*@__PURE__*/_defineComponent({
  __name: 'PublishEmojiPicker.client',
  emits: ["select", "selectCustom"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const { locale } = useI18n()
const el = ref<HTMLElement>()
const picker = ref<Picker>()
const colorMode = useColorMode()
async function openEmojiPicker() {
  await updateCustomEmojis()
  if (picker.value) {
    picker.value.update({
      theme: colorMode,
      custom: customEmojisData.value,
    })
  }
  else {
    const [Picker, dataPromise, i18n] = await Promise.all([
      import('emoji-mart').then(({ Picker }) => Picker),
      import('@emoji-mart/data/sets/14/twitter.json').then((r: any) => r.default).catch(() => {}),
      importEmojiLang(locale.value.split('-')[0]),
    ])
    picker.value = new Picker({
      data: () => dataPromise,
      onEmojiSelect({ native, src, alt, name }: any) {
        if (native)
          emit('select', native)
        else
          emit('selectCustom', { src, alt, 'data-emoji-id': name })
      },
      set: 'twitter',
      theme: colorMode,
      custom: customEmojisData.value,
      i18n,
    })
  }
  await nextTick()
  // TODO: custom picker
  el.value?.appendChild(picker.value as any as HTMLElement)
}
function hideEmojiPicker() {
  if (picker.value)
    el.value?.removeChild(picker.value as any as HTMLElement)
}

return (_ctx: any,_cache: any) => {
  const _component_VDropdown = _resolveComponent("VDropdown")
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")

  return (_openBlock(), _createBlock(_component_CommonTooltip, { content: _ctx.$t('tooltip.add_emojis') }, {
      default: _withCtx(() => [
        _createVNode(_component_VDropdown, {
          "auto-boundary-max-size": "",
          onApplyShow: _cache[0] || (_cache[0] = ($event: any) => (openEmojiPicker())),
          onApplyHide: _cache[1] || (_cache[1] = ($event: any) => (hideEmojiPicker()))
        }, {
          popper: _withCtx(() => [
            _createElementVNode("div", {
              ref_key: "el", ref: el,
              "min-w-10": "",
              "min-h-10": ""
            }, null, 512 /* NEED_PATCH */)
          ]),
          default: _withCtx(() => [
            _renderSlot(_ctx.$slots, "default")
          ]),
          _: 1 /* STABLE */
        })
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["content"]))
}
}

})
