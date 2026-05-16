import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { hidden: "true" }
const _hoisted_2 = { "font-bold": "true", "text-xl": "true", "text-secondary": "true" }
const _hoisted_3 = { "whitespace-pre-wrap": "true" }
const _hoisted_4 = { "aria-hidden": "true", "font-bold": "true", "text-sm": "true", "rounded-1": "true", "bg-black:65": "true", "text-white": "true", "px1.2": "true", "py0.2": "true", "pointer-events-none": "true" }
import type { mastodon } from 'masto'
import { clamp } from '@vueuse/core'
import { decode } from 'blurhash'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusAttachment',
  props: {
    attachment: { type: null, required: true },
    attachments: { type: Array, required: false },
    fullSize: { type: Boolean, required: false, default: false },
    isPreview: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {

const src = computed(() => __props.attachment.previewUrl || __props.attachment.url || __props.attachment.remoteUrl!)
const srcset = computed(() => [
  [__props.attachment.url, __props.attachment.meta?.original?.width],
  [__props.attachment.remoteUrl, __props.attachment.meta?.original?.width],
  [__props.attachment.previewUrl, __props.attachment.meta?.small?.width],
].filter(([url]) => url).map(([url, size]) => `${url} ${size}w`).join(', '))
const rawAspectRatio = computed(() => {
  if (__props.attachment.meta?.original?.aspect)
    return __props.attachment.meta.original.aspect
  if (__props.attachment.meta?.small?.aspect)
    return __props.attachment.meta.small.aspect
  return undefined
})
const aspectRatio = computed(() => {
  if (__props.fullSize)
    return rawAspectRatio.value
  if (rawAspectRatio.value)
    return clamp(rawAspectRatio.value, 0.8, 6)
  return undefined
})
const objectPosition = computed(() => {
  const focusX = __props.attachment.meta?.focus?.x || 0
  const focusY = __props.attachment.meta?.focus?.y || 0
  const x = ((focusX / 2) + 0.5) * 100
  const y = ((focusY / -2) + 0.5) * 100

  return `${x}% ${y}%`
})
const typeExtsMap = {
  video: ['mp4', 'webm', 'mov', 'avi', 'mkv', 'flv', 'wmv', 'mpg', 'mpeg'],
  audio: ['mp3', 'wav', 'ogg', 'flac', 'aac', 'm4a', 'wma'],
  image: ['jpg', 'jpeg', 'png', 'svg', 'webp', 'bmp'],
  gifv: ['gifv', 'gif'],
}
const type = computed(() => {
  if (__props.attachment.type && __props.attachment.type !== 'unknown')
    return __props.attachment.type
  // some server returns unknown type, we need to guess it based on file extension
  for (const [type, exts] of Object.entries(typeExtsMap)) {
    if (exts.some(ext => src.value?.toLowerCase().endsWith(`.${ext}`)))
      return type
  }
  return 'unknown'
})
const video = ref<HTMLVideoElement | undefined>()
const prefersReducedMotion = usePreferredReducedMotion()
const isAudio = computed(() => __props.attachment.type === 'audio')
const isVideo = computed(() => __props.attachment.type === 'video')
const isGif = computed(() => __props.attachment.type === 'gifv')
const enableAutoplay = usePreferences('enableAutoplay')
const unmuteVideos = usePreferences('unmuteVideos')
useIntersectionObserver(video, (entries) => {
  const ready = video.value?.dataset.ready === 'true'
  if (prefersReducedMotion.value === 'reduce' || !enableAutoplay.value) {
    if (ready && !video.value?.paused)
      video.value?.pause()
    return
  }
  entries.forEach((entry) => {
    if (entry.intersectionRatio <= 0.75) {
      if (ready && !video.value?.paused)
        video.value?.pause()
    }
    else {
      video.value?.play().then(() => {
        video.value!.dataset.ready = 'true'
      }).catch(noop)
    }
  })
}, { threshold: 0.75 })
const userSettings = useUserSettings()
const shouldLoadAttachment = ref(__props.isPreview || !getPreferences(userSettings.value, 'enableDataSaving'))
function loadAttachment() {
  shouldLoadAttachment.value = true
}
const blurHashSrc = computed(() => {
  if (!__props.attachment.blurhash)
    return ''
  const pixels = decode(__props.attachment.blurhash, 32, 32)
  return getDataUrlFromArr(pixels, 32, 32)
})
const videoThumbnail = ref(shouldLoadAttachment.value
  ? __props.attachment.previewUrl
  : blurHashSrc.value)
watch(shouldLoadAttachment, () => {
  videoThumbnail.value = shouldLoadAttachment.value
    ? __props.attachment.previewUrl
    : blurHashSrc.value
})

return (_ctx: any,_cache: any) => {
  const _component_CommonBlurhash = _resolveComponent("CommonBlurhash")
  const _component_VDropdown = _resolveComponent("VDropdown")
  const _directive_close_popper = _resolveDirective("close-popper")

  return (_openBlock(), _createElementBlock("div", {
      relative: "",
      ma: "",
      flex: "",
      gap: isAudio.value ? '2' : ''
    }, [ (type.value === 'video') ? (_openBlock(), _createElementBlock("button", {
          key: 0,
          type: "button",
          relative: "",
          onClick: _cache[0] || (_cache[0] = ($event: any) => (!shouldLoadAttachment.value ? loadAttachment() : null))
        }, [ _createElementVNode("video", {
            ref_key: "video", ref: video,
            preload: "none",
            poster: videoThumbnail.value,
            muted: !_unref(unmuteVideos),
            loop: "",
            playsinline: "",
            controls: shouldLoadAttachment.value,
            "rounded-lg": "",
            "object-cover": "",
            "fullscreen:object-contain": "",
            width: __props.attachment.meta?.original?.width,
            height: __props.attachment.meta?.original?.height,
            style: _normalizeStyle({
              aspectRatio: aspectRatio.value,
              objectPosition: objectPosition.value,
            }),
            class: _normalizeClass(!shouldLoadAttachment.value ? 'brightness-60 hover:brightness-70 transition-filter' : '')
          }, [ _createElementVNode("source", {
              src: __props.attachment.url || __props.attachment.previewUrl,
              type: "video/mp4"
            }, null, 8 /* PROPS */, ["src"]) ], 14 /* CLASS, STYLE, PROPS */, ["poster", "muted", "controls", "width", "height"]), (!shouldLoadAttachment.value) ? (_openBlock(), _createElementBlock("span", {
              key: 0,
              class: "status-attachment-load",
              absolute: "",
              "text-sm": "",
              "text-white": "",
              flex: "",
              "flex-col": "",
              "justify-center": "",
              "items-center": "",
              "gap-3": "",
              "w-6": "",
              "h-6": "",
              "pointer-events-none": "",
              "i-ri:video-download-line": ""
            })) : _createCommentVNode("v-if", true) ])) : (type.value === 'gifv') ? (_openBlock(), _createElementBlock("button", {
            key: 1,
            type: "button",
            relative: "",
            onClick: _cache[1] || (_cache[1] = ($event: any) => (!shouldLoadAttachment.value ? loadAttachment() : _ctx.openMediaPreview(__props.attachments ? __props.attachments : [__props.attachment], __props.attachments?.indexOf(__props.attachment) || 0)))
          }, [ _createElementVNode("video", {
              ref_key: "video", ref: video,
              preload: "none",
              poster: videoThumbnail.value,
              muted: !_unref(unmuteVideos),
              loop: "",
              playsinline: "",
              "rounded-lg": "",
              "object-cover": "",
              width: __props.attachment.meta?.original?.width,
              height: __props.attachment.meta?.original?.height,
              style: _normalizeStyle({
              aspectRatio: aspectRatio.value,
              objectPosition: objectPosition.value,
            })
            }, [ _createElementVNode("source", {
                src: __props.attachment.url || __props.attachment.previewUrl,
                type: "video/mp4"
              }, null, 8 /* PROPS */, ["src"]) ], 12 /* STYLE, PROPS */, ["poster", "muted", "width", "height"]), (!shouldLoadAttachment.value) ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                class: "status-attachment-load",
                absolute: "",
                "text-sm": "",
                "text-white": "",
                flex: "",
                "flex-col": "",
                "justify-center": "",
                "items-center": "",
                "gap-3": "",
                "w-6": "",
                "h-6": "",
                "pointer-events-none": "",
                "i-ri:video-download-line": ""
              })) : _createCommentVNode("v-if", true) ])) : (type.value === 'audio') ? (_openBlock(), _createElementBlock("audio", {
            key: 2,
            controls: "",
            "h-15": ""
          }, [ _createElementVNode("source", {
              src: __props.attachment.url || __props.attachment.previewUrl,
              type: "audio/mp3"
            }, null, 8 /* PROPS */, ["src"]) ])) : (_openBlock(), _createElementBlock("button", {
          key: 3,
          type: "button",
          "focus:outline-none": "",
          "focus:ring": "2 primary inset",
          "rounded-lg": "",
          "h-full": "",
          "w-full": "",
          "aria-label": _ctx.$t('action.open_image_preview_dialog'),
          relative: "",
          onClick: _cache[2] || (_cache[2] = ($event: any) => (!shouldLoadAttachment.value ? loadAttachment() : _ctx.openMediaPreview(__props.attachments ? __props.attachments : [__props.attachment], __props.attachments?.indexOf(__props.attachment) || 0)))
        }, [ _createVNode(_component_CommonBlurhash, {
            blurhash: __props.attachment.blurhash || '',
            src: src.value,
            srcset: srcset.value,
            width: __props.attachment.meta?.original?.width,
            height: __props.attachment.meta?.original?.height,
            alt: __props.attachment.description ?? 'Image',
            style: _normalizeStyle({
              aspectRatio: aspectRatio.value,
              objectPosition: objectPosition.value,
            }),
            "should-load-image": shouldLoadAttachment.value,
            "rounded-lg": "",
            "h-full": "",
            "w-full": "",
            "object-cover": "",
            draggable: shouldLoadAttachment.value,
            class: _normalizeClass(["status-attachment-image", !shouldLoadAttachment.value ? 'brightness-60 hover:brightness-70 transition-filter' : ''])
          }, null, 14 /* CLASS, STYLE, PROPS */, ["blurhash", "src", "srcset", "width", "height", "alt", "should-load-image", "draggable"]), (!shouldLoadAttachment.value) ? (_openBlock(), _createElementBlock("span", {
              key: 0,
              class: "status-attachment-load",
              absolute: "",
              "text-sm": "",
              "text-white": "",
              flex: "",
              "flex-col": "",
              "justify-center": "",
              "items-center": "",
              "gap-3": "",
              "w-6": "",
              "h-6": "",
              "pointer-events-none": "",
              "i-ri:file-download-line": ""
            })) : _createCommentVNode("v-if", true) ])), _createElementVNode("div", {
        class: _normalizeClass(isAudio.value ? [] : [
          'absolute left-2',
          isVideo.value ? 'top-2' : 'bottom-2',
        ]),
        flex: "",
        "gap-col-2": ""
      }, [ (__props.attachment.description && !_ctx.getPreferences(_unref(userSettings), 'hideAltIndicatorOnPosts')) ? (_openBlock(), _createBlock(_component_VDropdown, {
            key: 0,
            distance: 6,
            placement: "bottom-start"
          }, {
            popper: _withCtx(() => [
              _createElementVNode("div", {
                p4: "",
                flex: "",
                "flex-col": "",
                "gap-2": "",
                "max-w-130": ""
              }, [
                _createElementVNode("div", {
                  flex: "",
                  "justify-between": ""
                }, [
                  _createElementVNode("h2", _hoisted_2, _toDisplayString(_ctx.$t('status.img_alt.desc')), 1 /* TEXT */),
                  _createElementVNode("button", {
                    "text-sm": "",
                    "btn-outline": "",
                    py0: "",
                    px2: "",
                    "text-secondary": "",
                    "border-base": ""
                  }, _toDisplayString(_ctx.$t('status.img_alt.dismiss')), 1 /* TEXT */)
                ]),
                _createElementVNode("p", _hoisted_3, _toDisplayString(__props.attachment.description), 1 /* TEXT */)
              ])
            ]),
            default: _withCtx(() => [
              _createElementVNode("button", {
                "font-bold": "",
                "text-sm": "",
                class: _normalizeClass(isAudio.value
              ? 'rounded-full h-15 w-15 btn-outline border-base text-secondary hover:bg-active hover:text-active'
              : 'rounded-1 bg-black/65 text-white hover:bg-black px1.2 py0.2')
              }, [
                _createElementVNode("div", _hoisted_1, _toDisplayString(_ctx.$t('status.img_alt.read', [__props.attachment.type])), 1 /* TEXT */),
                _createTextVNode("\n          " + _toDisplayString(_ctx.$t('status.img_alt.ALT')), 1 /* TEXT */)
              ], 2 /* CLASS */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["distance"])) : _createCommentVNode("v-if", true), (isGif.value && !_ctx.getPreferences(_unref(userSettings), 'hideGifIndicatorOnPosts')) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createElementVNode("button", _hoisted_4, _toDisplayString(_ctx.$t('status.gif')), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true) ], 2 /* CLASS */) ], 8 /* PROPS */, ["gap"]))
}
}

})
