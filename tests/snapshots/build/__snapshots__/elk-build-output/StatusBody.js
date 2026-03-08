import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { my2: "true", "h-px": "true", border: "b base", "bg-base": "true" })
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusBody',
  props: {
    status: { type: null, required: true },
    newer: { type: null, required: false },
    withAction: { type: Boolean, required: false, default: true },
    isNested: { type: Boolean, required: false, default: false }
  },
  async setup(__props: any) {

let __temp: any, __restore: any

const hasQuote = computed(() => 'quote' in __props.status && !!__props.status.quote)
const { translation } = await useTranslation(__props.status, getLanguageCode())
const emojisObject = useEmojisFallback(() => __props.status.emojis)
const vnode = computed(() => {
  if (!__props.status.content)
    return null
  return contentToVNode(__props.status.content, {
    emojis: emojisObject.value,
    mentions: 'mentions' in __props.status ? __props.status.mentions : undefined,
    markdown: true,
    collapseMentionLink: !!('inReplyToId' in __props.status && __props.status.inReplyToId),
    status: 'id' in __props.status ? __props.status : undefined,
    inReplyToStatus: __props.newer,
  })
})

return (_ctx: any,_cache: any) => {
  const _component_StatusQuote = _resolveComponent("StatusQuote")
  const _component_ContentRich = _resolveComponent("ContentRich")

  return (_openBlock(), _createElementBlock("div", {
      "whitespace-pre-wrap": "",
      "break-words": "",
      class: _normalizeClass(["status-body", { 'with-action': __props.withAction, 'has-quote': hasQuote.value }]),
      relative: ""
    }, [ (__props.status.content) ? (_openBlock(), _createElementBlock("span", {
          key: 0,
          class: "content-rich line-compact",
          dir: "auto",
          lang: ('language' in __props.status && __props.status.language) || undefined
        }, [ (vnode.value) ? (_openBlock(), _createBlock(_resolveDynamicComponent(vnode.value), {
              key: 0,
              is: vnode.value
            })) : _createCommentVNode("v-if", true) ])) : (_openBlock(), _createElementBlock("div", { key: 1 })), _createVNode(_component_StatusQuote, {
        status: __props.status,
        "is-nested": __props.isNested
      }, null, 8 /* PROPS */, ["status", "is-nested"]), (_unref(translation).visible) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _hoisted_1, (_unref(translation).success) ? (_openBlock(), _createBlock(_component_ContentRich, {
              key: 0,
              class: "line-compact",
              content: _unref(translation).text,
              emojis: __props.status.emojis
            }, null, 8 /* PROPS */, ["content", "emojis"])) : (_openBlock(), _createElementBlock("div", {
              key: 1,
              "text-red-4": ""
            }, "\n        Error: " + _toDisplayString(_unref(translation).error), 1 /* TEXT */)) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ], 2 /* CLASS */))
}
}

})
