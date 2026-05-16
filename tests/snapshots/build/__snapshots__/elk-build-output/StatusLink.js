import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, renderSlot as _renderSlot, normalizeClass as _normalizeClass, withKeys as _withKeys } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusLink',
  props: {
    status: { type: null, required: true },
    hover: { type: Boolean, required: false },
    disableLink: { type: Boolean, required: false }
  },
  setup(__props: any) {

const el = ref<HTMLElement>()
const router = useRouter()
const statusRoute = computed(() => getStatusRoute(__props.status))
function onclick(evt: MouseEvent | KeyboardEvent) {
  if (__props.disableLink)
    return
  const path = evt.composedPath() as HTMLElement[]
  const el = path.find(el => ['A', 'BUTTON', 'IMG', 'VIDEO'].includes(el.tagName?.toUpperCase()))
  const text = window.getSelection()?.toString()
  const isCustomEmoji = el?.parentElement?.classList.contains('custom-emoji')
  if ((!el && !text) || isCustomEmoji)
    go(evt)
}
function go(evt: MouseEvent | KeyboardEvent) {
  if (evt.metaKey || evt.ctrlKey) {
    window.open(statusRoute.value.href)
  }
  else {
    cacheStatus(__props.status)
    router.push(statusRoute.value)
  }
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      id: `status-${__props.status.id}`,
      ref_key: "el", ref: el,
      relative: "",
      flex: "~ col gap1",
      p: "b-2 is-3 ie-4",
      class: _normalizeClass({ 'hover:bg-active': __props.hover }),
      tabindex: "0",
      "focus:outline-none": "",
      "focus-visible:ring": "2 primary inset",
      "aria-roledescription": "status-card",
      lang: __props.status.language ?? undefined,
      onClick: onclick,
      onKeydown: _withKeys(onclick, ["enter"])
    }, [ _renderSlot(_ctx.$slots, "default") ], 42 /* CLASS, PROPS, NEED_HYDRATION */, ["id", "lang"]))
}
}

})
