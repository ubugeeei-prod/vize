import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeStyle as _normalizeStyle, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-svg-spinners:90-ring-with-bg h-5 w-5 inline-block" })
const _hoisted_2 = { class: "font-medium text-fg truncate" }
const _hoisted_3 = { class: "text-sm text-fg-subtle truncate" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "i-simple-icons:bluesky w-5 h-5 text-fg-subtle ms-auto shrink-0", "aria-hidden": "true" })
const _hoisted_5 = { class: "text-fg-muted whitespace-pre-wrap leading-relaxed mb-3" }
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:heart w-3.5 h-3.5", "aria-hidden": "true" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:repeat w-3.5 h-3.5", "aria-hidden": "true" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:message-circle w-3.5 h-3.5", "aria-hidden": "true" })
import { BLUESKY_API, BSKY_POST_AT_URI_REGEX } from '#shared/utils/constants'

interface PostAuthor {
  did: string
  handle: string
  displayName?: string
  avatar?: string
}
interface EmbedImage {
  thumb: string
  fullsize: string
  alt: string
  aspectRatio?: { width: number; height: number }
}
interface BlueskyPost {
  uri: string
  author: PostAuthor
  record: { text: string; createdAt: string }
  embed?: { $type: string; images?: EmbedImage[] }
  likeCount?: number
  replyCount?: number
  repostCount?: number
}

export default /*@__PURE__*/_defineComponent({
  __name: 'BlueskyPostEmbed.client',
  props: {
    uri: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const postUrl = computed(() => {
  const match = props.uri.match(BSKY_POST_AT_URI_REGEX)
  if (!match) return null
  const [, did, rkey] = match
  return `https://bsky.app/profile/${did}/post/${rkey}`
})
const { data: post, status } = useAsyncData(
  `bsky-post-${props.uri}`,
  async (): Promise<BlueskyPost | null> => {
    const response = await $fetch<{ posts: BlueskyPost[] }>(
      `${BLUESKY_API}/xrpc/app.bsky.feed.getPosts`,
      { query: { uris: props.uri } },
    )
    return response.posts[0] ?? null
  },
  { lazy: true, server: false },
)

return (_ctx: any,_cache: any) => {
  const _component_DateTime = _resolveComponent("DateTime")

  return (_unref(status) === 'pending')
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        class: "rounded-lg border border-border bg-bg-subtle p-6 text-center text-fg-subtle text-sm"
      }, [ _hoisted_1 ]))
      : (_unref(post))
        ? (_openBlock(), _createElementBlock("a", {
          key: 1,
          href: postUrl.value ?? '#',
          target: "_blank",
          rel: "noopener noreferrer",
          class: "block rounded-lg border border-border bg-bg-subtle p-4 sm:p-5 no-underline hover:border-border-hover transition-colors duration-200"
        }, [ _createElementVNode("div", { class: "flex items-center gap-3 mb-3" }, [ (_unref(post).author.avatar) ? (_openBlock(), _createElementBlock("img", {
                key: 0,
                src: `${_unref(post).author.avatar}?size=48`,
                alt: _unref(post).author.displayName || _unref(post).author.handle,
                width: "40",
                height: "40",
                class: "w-10 h-10 rounded-full",
                loading: "lazy"
              })) : _createCommentVNode("v-if", true), _createElementVNode("div", { class: "min-w-0" }, [ _createElementVNode("div", _hoisted_2, _toDisplayString(_unref(post).author.displayName || _unref(post).author.handle), 1 /* TEXT */), _createElementVNode("div", _hoisted_3, "@" + _toDisplayString(_unref(post).author.handle), 1 /* TEXT */) ]), _hoisted_4 ]), _createTextVNode("\n\n    "), _createTextVNode("\n    "), _createElementVNode("p", _hoisted_5, _toDisplayString(_unref(post).record.text), 1 /* TEXT */), _createTextVNode("\n\n    "), _createTextVNode("\n    "), (_unref(post).embed?.images?.length) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(post).embed.images, (img, i) => {
                return (_openBlock(), _createElementBlock("img", {
                  key: i,
                  src: img.fullsize,
                  alt: img.alt,
                  class: "w-full mb-3 rounded-lg object-cover",
                  style: _normalizeStyle(
            img.aspectRatio
              ? { aspectRatio: `${img.aspectRatio.width}/${img.aspectRatio.height}` }
              : undefined
          ),
                  loading: "lazy"
                }, 12 /* STYLE, PROPS */, ["src", "alt"]))
              }), 128 /* KEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    "), _createTextVNode("\n    "), _createElementVNode("div", { class: "flex items-center gap-4 text-sm text-fg-subtle" }, [ _createVNode(_component_DateTime, {
              datetime: _unref(post).record.createdAt,
              "date-style": "medium"
            }, null, 8 /* PROPS */, ["datetime"]), (_unref(post).likeCount) ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                class: "flex items-center gap-1"
              }, [ _hoisted_6, _createTextVNode("\n        "), _toDisplayString(_unref(post).likeCount) ])) : _createCommentVNode("v-if", true), (_unref(post).repostCount) ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                class: "flex items-center gap-1"
              }, [ _hoisted_7, _createTextVNode("\n        "), _toDisplayString(_unref(post).repostCount) ])) : _createCommentVNode("v-if", true), (_unref(post).replyCount) ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                class: "flex items-center gap-1"
              }, [ _hoisted_8, _createTextVNode("\n        "), _toDisplayString(_unref(post).replyCount) ])) : _createCommentVNode("v-if", true) ]) ]))
      : _createCommentVNode("v-if", true)
}
}

})
