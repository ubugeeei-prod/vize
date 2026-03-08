import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, withCtx as _withCtx, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { "text-sm": "true", "text-white": "true", flex: "true", "flex-col": "true", "justify-center": "true", "items-center": "true", "gap-3": "true", "w-6": "true", "h-6": "true", "i-ri:file-download-line": "true" })
import type { mastodon } from 'masto'
import punycode from 'punycode/'
const ogImageWidth = 400

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusPreviewCardNormal',
  props: {
    card: { type: null, required: true },
    smallPictureOnly: { type: Boolean, required: false },
    root: { type: Boolean, required: false }
  },
  setup(__props: any) {

// mastodon's default max og image width
const alt = computed(() => `${__props.card.title} - ${__props.card.title}`)
const isSquare = computed(() => (
  __props.smallPictureOnly
  || !__props.card.image
  || __props.card.width === __props.card.height
  || Number(__props.card.width || 0) < ogImageWidth
  || Number(__props.card.height || 0) < ogImageWidth / 2
))
const providerName = computed(() => __props.card.providerName ? __props.card.providerName : punycode.toUnicode(new URL(__props.card.url).hostname))
// TODO: handle card.type: 'photo' | 'video' | 'rich';
const cardTypeIconMap: Record<mastodon.v1.PreviewCardType, string> = {
  link: 'i-ri:profile-line',
  photo: 'i-ri:image-line',
  video: 'i-ri:play-line',
  rich: 'i-ri:profile-line',
}
const userSettings = useUserSettings()
const shouldLoadAttachment = ref(!getPreferences(userSettings.value, 'enableDataSaving'))
function loadAttachment() {
  shouldLoadAttachment.value = true
}

return (_ctx: any,_cache: any) => {
  const _component_CommonBlurhash = _resolveComponent("CommonBlurhash")
  const _component_StatusPreviewCardInfo = _resolveComponent("StatusPreviewCardInfo")
  const _component_StatusPreviewCardMoreFromAuthor = _resolveComponent("StatusPreviewCardMoreFromAuthor")
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (_openBlock(), _createBlock(_component_NuxtLink, {
      block: "",
      "of-hidden": "",
      to: __props.card.url,
      "bg-card": "",
      "hover:bg-active": "",
      class: _normalizeClass({
        'flex flex-col': isSquare.value,
        'p-4': __props.root,
        'rounded-lg': !__props.root,
      }),
      target: "_blank",
      external: ""
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(isSquare.value ? 'flex' : '')
        }, [
          (__props.card.image)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              flex: "",
              "flex-col": "",
              "display-block": "",
              "of-hidden": "",
              class: _normalizeClass({
            'sm:(min-w-32 w-32 h-32) min-w-24 w-24 h-24': isSquare.value,
            'w-full aspect-[1.91]': !isSquare.value,
            'rounded-lg': __props.root,
          }),
              relative: ""
            }, [
              _createVNode(_component_CommonBlurhash, {
                blurhash: __props.card.blurhash,
                src: __props.card.image,
                width: __props.card.width,
                height: __props.card.height,
                alt: alt.value,
                "should-load-image": shouldLoadAttachment.value,
                "w-full": "",
                "h-full": "",
                "object-cover": "",
                class: _normalizeClass(!shouldLoadAttachment.value ? 'brightness-60' : '')
              }, null, 10 /* CLASS, PROPS */, ["blurhash", "src", "width", "height", "alt", "should-load-image"]),
              (!shouldLoadAttachment.value)
                ? (_openBlock(), _createElementBlock("button", {
                  key: 0,
                  type: "button",
                  absolute: "",
                  class: "status-preview-card-load bg-black/64",
                  "p-2": "",
                  transition: "",
                  rounded: "",
                  "hover:bg-black": "",
                  "cursor-pointer": "",
                  onClick: _cache[0] || (_cache[0] = _withModifiers(($event: any) => (!shouldLoadAttachment.value ? loadAttachment() : null), ["stop","prevent"]))
                }, [
                  _hoisted_1
                ]))
                : _createCommentVNode("v-if", true)
            ]))
            : (_openBlock(), _createElementBlock("div", {
              key: 1,
              "min-w-24": "",
              "w-24": "",
              "h-24": "",
              sm: "min-w-32 w-32 h-32",
              bg: "slate-500/10",
              flex: "",
              "justify-center": "",
              "items-center": "",
              class: _normalizeClass([
            __props.root ? 'rounded-lg' : '',
          ])
            }, [
              _createElementVNode("div", {
                class: _normalizeClass(cardTypeIconMap[__props.card.type]),
                w: "30%",
                h: "30%",
                "text-secondary": ""
              }, null, 2 /* CLASS */)
            ])),
          _createTextVNode("\n      " + "\n      "),
          _createVNode(_component_StatusPreviewCardInfo, {
            p: isSquare.value ? 'x-4' : '4',
            root: __props.root,
            card: __props.card,
            provider: providerName.value
          }, null, 8 /* PROPS */, ["p", "root", "card", "provider"])
        ], 2 /* CLASS */),
        (__props.card?.authors?.[0]?.account)
          ? (_openBlock(), _createBlock(_component_StatusPreviewCardMoreFromAuthor, {
            key: 0,
            account: __props.card.authors[0].account
          }, null, 8 /* PROPS */, ["account"]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 10 /* CLASS, PROPS */, ["to"]))
}
}

})
