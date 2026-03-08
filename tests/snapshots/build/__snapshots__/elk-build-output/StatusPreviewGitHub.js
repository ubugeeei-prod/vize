import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { "text-secondary-light": "true" }, "/")
const _hoisted_2 = { "text-primary": "true", "font-bold": "true" }
const _hoisted_3 = { "text-secondary": "true", "leading-tight": "true" }
const _hoisted_4 = { "text-lg": "true", "text-primary": "true" }
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("div", { "text-3xl": "true", "i-ri:github-fill": "true", "text-secondary": "true" })
import type { mastodon } from 'masto'
import reservedNames from 'github-reserved-names'

type UrlType = 'user' | 'repo' | 'issue' | 'pull'
interface Meta {
  type: UrlType
  user?: string
  titleUrl: string
  avatar: string
  details: string
  repo?: string
  number?: string
  author?: {
    avatar: string
    user: string
  }
}

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusPreviewGitHub',
  props: {
    card: { type: null, required: true }
  },
  setup(__props: any) {

// Supported paths
// /user
// /user/repo
// /user/repo/issues/number
// /user/repo/pull/number
// /sponsors/user
const supportedReservedRoutes = ['sponsors']
const meta = computed(() => {
  const { url } = __props.card
  const path = url.split('https://github.com/')[1]
  const [firstName, secondName] = path?.split('/') || []
  if (!firstName || (reservedNames.check(firstName) && !supportedReservedRoutes.includes(firstName)))
    return undefined

  const firstIsUser = firstName && !supportedReservedRoutes.includes(firstName)
  const user = firstIsUser ? firstName : secondName
  const repo = firstIsUser ? secondName : undefined

  let type: UrlType = repo ? 'repo' : 'user'
  let number: string | undefined
  let details = (__props.card.title ?? '').replace('GitHub - ', '').split(' · ')[0]

  if (repo) {
    const repoPath = `${user}/${repo}`
    details = details.replace(`${repoPath}: `, '')
    const inRepoPath = path.split(`${repoPath}/`)?.[1]
    if (inRepoPath) {
      number = inRepoPath.match(/issues\/(\d+)/)?.[1]
      if (number) {
        type = 'issue'
      }
      else {
        number = inRepoPath.match(/pull\/(\d+)/)?.[1]
        if (number)
          type = 'pull'
      }
    }
  }

  const avatar = `https://github.com/${user}.png?size=256`

  const author = __props.card.authorName
  return {
    type,
    user,
    titleUrl: `https://github.com/${user}${repo ? `/${repo}` : ''}`,
    details,
    repo,
    number,
    avatar,
    author: author
      ? {
          avatar: `https://github.com/${author}.png?size=64`,
          user: author,
        }
      : undefined,
  } satisfies Meta
})

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_StatusPreviewCardNormal = _resolveComponent("StatusPreviewCardNormal")

  return (__props.card.image && meta.value)
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        flex: "",
        "flex-col": "",
        "display-block": "",
        "of-hidden": "",
        "bg-card": "",
        relative: "",
        "w-full": "",
        "min-h-50": "",
        "md:min-h-60": "",
        "justify-center": "",
        "rounded-lg": ""
      }, [ _createElementVNode("div", {
          p4: "",
          "sm:px-8": "",
          flex: "",
          "flex-col": "",
          "justify-between": "",
          "min-h-50": "",
          "md:min-h-60": "",
          "h-full": ""
        }, [ _createElementVNode("div", {
            flex: "",
            "justify-between": "",
            "items-center": "",
            "gap-2": "",
            "sm:gap-6": "",
            "h-full": "",
            "mb-2": "",
            "min-h-35": "",
            "md:min-h-45": ""
          }, [ _createElementVNode("div", {
              flex: "",
              "flex-col": "",
              "gap-2": ""
            }, [ _createVNode(_component_NuxtLink, {
                flex: "",
                "gap-1": "",
                "text-xl": "",
                "sm:text-3xl": "",
                "flex-wrap": "",
                "leading-none": "",
                href: meta.value.titleUrl,
                target: "_blank",
                external: ""
              }, {
                default: _withCtx(() => [
                  (meta.value.repo)
                    ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                      _createElementVNode("span", null, _toDisplayString(meta.value.user), 1 /* TEXT */),
                      _hoisted_1,
                      _createElementVNode("span", _hoisted_2, _toDisplayString(meta.value.repo), 1 /* TEXT */)
                    ], 64 /* STABLE_FRAGMENT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(meta.value.user), 1 /* TEXT */))
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["href"]), _createVNode(_component_NuxtLink, {
                "sm:text-lg": "",
                href: __props.card.url,
                target: "_blank",
                external: ""
              }, {
                default: _withCtx(() => [
                  (meta.value.type === 'issue')
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      "text-secondary-light": "",
                      "me-2": ""
                    }, "\n              #" + _toDisplayString(meta.value.number), 1 /* TEXT */))
                    : _createCommentVNode("v-if", true),
                  (meta.value.type === 'pull')
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      "text-secondary-light": "",
                      "me-2": ""
                    }, "\n              PR #" + _toDisplayString(meta.value.number), 1 /* TEXT */))
                    : _createCommentVNode("v-if", true),
                  _createElementVNode("span", _hoisted_3, _toDisplayString(meta.value.details), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["href"]) ]), _createElementVNode("div", {
              "shrink-0": "",
              "w-18": "",
              "sm:w-30": ""
            }, [ _createVNode(_component_NuxtLink, {
                href: meta.value.titleUrl,
                target: "_blank",
                external: ""
              }, {
                default: _withCtx(() => [
                  _createElementVNode("img", {
                    "w-full": "",
                    "aspect-square": "",
                    width: "112",
                    height: "112",
                    "rounded-2": "",
                    src: meta.value.avatar
                  }, null, 8 /* PROPS */, ["src"])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["href"]) ]) ]), _createElementVNode("div", {
            flex: "",
            "justify-between": ""
          }, [ (meta.value.author) ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                flex: "",
                class: "gap-2.5",
                "items-center": ""
              }, [ _createElementVNode("div", null, [ _createElementVNode("img", {
                    "w-8": "",
                    "aspect-square": "",
                    width: "25",
                    height: "25",
                    "rounded-full": "",
                    src: meta.value.author?.avatar
                  }, null, 8 /* PROPS */, ["src"]) ]), _createElementVNode("span", _hoisted_4, "@" + _toDisplayString(meta.value.author?.user), 1 /* TEXT */) ])) : (_openBlock(), _createElementBlock("div", { key: 1 })), _hoisted_5 ]) ]) ]))
      : (_openBlock(), _createBlock(_component_StatusPreviewCardNormal, {
        key: 1,
        card: __props.card
      }, null, 8 /* PROPS */, ["card"]))
}
}

})
