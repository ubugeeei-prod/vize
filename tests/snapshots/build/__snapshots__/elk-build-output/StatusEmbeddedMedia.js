import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { "flex-row": "true", "w-4": "true", "h-4": "true", "pointer-events-none": "true", "i-ri:arrow-right-up-line": "true" })
const _hoisted_2 = { "font-bold": "true", "line-clamp-1": "true", "text-size-base": "true" }
const _hoisted_3 = { "line-clamp-1": "true" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { "text-light": "true", flex: "true", "flex-col": "true", "gap-3": "true", "w-27": "true", "h-27": "true", "pointer-events-none": "true", "i-ri:play-circle-line": "true" })
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusEmbeddedMedia',
  props: {
    status: { type: null, required: true }
  },
  setup(__props: any) {

const vnode = computed(() => {
  if (!__props.status.card?.html)
    return null
  const node = sanitizeEmbeddedIframe(__props.status.card?.html)?.children[0]
  return node ? nodeToVNode(node) : null
})
const overlayToggle = ref(true)
const card = ref(__props.status.card)

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_CommonBlurhash = _resolveComponent("CommonBlurhash")

  return (card.value)
      ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ (overlayToggle.value) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            "h-80": "",
            "cursor-pointer": "",
            relative: ""
          }, [ _createElementVNode("div", {
              "p-3": "",
              absolute: "",
              "w-full": "",
              "h-full": "",
              "z-10": "",
              "rounded-lg": "",
              style: "background: linear-gradient(black, rgba(0,0,0,0.5), transparent, transparent, rgba(0,0,0,0.20))"
            }, [ _createVNode(_component_NuxtLink, {
                flex: "",
                "flex-col": "",
                "gap-1": "",
                "hover:underline": "",
                "text-xs": "",
                "text-light": "",
                "font-light": "",
                target: "_blank",
                href: card.value?.url
              }, {
                default: _withCtx(() => [
                  _createElementVNode("div", {
                    flex: "",
                    "gap-0.5": ""
                  }, [
                    _createElementVNode("p", {
                      "flex-row": "",
                      "line-clamp-1": ""
                    }, [
                      _createTextVNode(_toDisplayString(card.value?.providerName), 1 /* TEXT */),
                      (card.value?.authorName)
                        ? (_openBlock(), _createElementBlock("span", { key: 0 }, " • " + _toDisplayString(card.value?.authorName), 1 /* TEXT */))
                        : _createCommentVNode("v-if", true)
                    ]),
                    _hoisted_1
                  ]),
                  _createElementVNode("p", _hoisted_2, _toDisplayString(card.value?.title), 1 /* TEXT */),
                  _createElementVNode("p", _hoisted_3, _toDisplayString(_ctx.$t('status.embedded_warning')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["href"]), _createElementVNode("div", {
                flex: "",
                "h-50": "",
                "mt-1": "",
                "justify-center": "",
                "flex-items-center": ""
              }, [ _createElementVNode("button", {
                  absolute: "",
                  "bg-primary": "",
                  "opacity-85": "",
                  "rounded-full": "",
                  "hover:bg-primary-active": "",
                  "hover:opacity-95": "",
                  "transition-all": "",
                  "box-shadow-outline": "",
                  onClick: _cache[0] || (_cache[0] = _withModifiers(() => overlayToggle.value = !overlayToggle.value, ["stop","prevent"]))
                }, [ _hoisted_4 ]) ]) ]), (card.value?.image) ? (_openBlock(), _createBlock(_component_CommonBlurhash, {
                key: 0,
                blurhash: card.value.blurhash,
                src: card.value.image,
                "w-full": "",
                "h-full": "",
                "object-cover": "",
                "rounded-lg": ""
              }, null, 8 /* PROPS */, ["blurhash", "src"])) : _createCommentVNode("v-if", true) ])) : (_openBlock(), _createElementBlock("div", { key: 1 }, [ (vnode.value) ? (_openBlock(), _createBlock(_resolveDynamicComponent(vnode.value), {
                key: 0,
                is: vnode.value,
                "rounded-lg": "",
                "h-80": ""
              })) : _createCommentVNode("v-if", true) ])) ]))
      : _createCommentVNode("v-if", true)
}
}

})
