import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'StatusSpoiler',
  props: {
    enabled: { type: Boolean, required: false },
    filter: { type: Boolean, required: false },
    isDM: { type: Boolean, required: false },
    sensitiveNonSpoiler: { type: Boolean, required: false }
  },
  setup(__props: any) {

const expandSpoilers = computed(() => {
  const expandCW = currentUser.value ? getExpandSpoilersByDefault(currentUser.value.account) : false
  const expandMedia = currentUser.value ? getExpandMediaByDefault(currentUser.value.account) : false

  return !__props.filter // always prevent expansion if filtered
    && ((__props.sensitiveNonSpoiler && expandMedia)
      || (!__props.sensitiveNonSpoiler && expandCW))
})
const hideContent = __props.enabled || __props.sensitiveNonSpoiler
const showContent = ref(expandSpoilers.value ? true : !hideContent)
const toggleContent = useToggle(showContent)
watchEffect(() => {
  showContent.value = expandSpoilers.value ? true : !hideContent
})
function getToggleText() {
  if (__props.sensitiveNonSpoiler)
    return 'status.spoiler_media_hidden'
  return __props.filter ? 'status.filter_show_anyway' : 'status.spoiler_show_more'
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock(_Fragment, null, [ (_unref(hideContent)) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          flex: "",
          "flex-col": "",
          "items-start": ""
        }, [ _createElementVNode("div", {
            class: "content-rich",
            p: "x-4 b-2.5",
            "text-center": "",
            "text-secondary": "",
            "w-full": "",
            border: "~ base",
            "border-0": "",
            "border-b-dotted": "",
            "border-b-3": "",
            "mt-2": ""
          }, [ _renderSlot(_ctx.$slots, "spoiler") ]), _createElementVNode("div", {
            flex: "~ gap-1 center",
            "w-full": "",
            mb: __props.isDM && !showContent.value ? '4' : '',
            mt: "-4.5"
          }, [ _createElementVNode("button", {
              "btn-text": "",
              "px-2": "",
              "py-1": "",
              "rounded-lg": "",
              bg: __props.isDM ? 'transparent' : 'base',
              flex: "~ center gap-2",
              class: _normalizeClass(showContent.value ? '' : 'filter-saturate-0 hover:filter-saturate-100'),
              "aria-expanded": showContent.value,
              onClick: _cache[0] || (_cache[0] = ($event: any) => (_unref(toggleContent)()))
            }, [ (showContent.value) ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  "i-ri:eye-line": ""
                })) : (_openBlock(), _createElementBlock("div", {
                  key: 1,
                  "i-ri:eye-close-line": ""
                })), _createTextVNode("\n        " + _toDisplayString(showContent.value ? _ctx.$t('status.spoiler_show_less') : _ctx.$t(getToggleText())), 1 /* TEXT */) ], 10 /* CLASS, PROPS */, ["bg", "aria-expanded"]) ], 8 /* PROPS */, ["mb"]) ])) : _createCommentVNode("v-if", true), (!_unref(hideContent) || showContent.value) ? (_openBlock(), _createElementBlock("slot", { key: 0 })) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */))
}
}

})
