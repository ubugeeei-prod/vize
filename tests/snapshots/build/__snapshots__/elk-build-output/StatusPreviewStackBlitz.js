import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


const _hoisted_1 = { "text-secondary": "true" }
const _hoisted_2 = { "text-primary": "true" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("path", { fill: "currentColor", d: "M109.586 217.013H0L200.34 0l-53.926 150.233H256L55.645 367.246l53.927-150.233z" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", null, "StackBlitz")
import type { mastodon } from 'masto'

interface Meta {
  code?: string
  file?: string
  lines?: string
  project?: string
}
const maxLines = 20

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusPreviewStackBlitz',
  props: {
    card: { type: null, required: true },
    smallPictureOnly: { type: Boolean, required: false },
    root: { type: Boolean, required: false }
  },
  setup(__props: any) {

// Protect against long code snippets
const meta = computed(() => {
  const { description } = __props.card
  const meta = description.match(/.*Code Snippet from (.+), lines (\S+)\n\n(.+)/s)
  const file = meta?.[1]
  const lines = meta?.[2]
  const code = meta?.[3].split('\n').slice(0, maxLines).join('\n')
  const project = __props.card.title?.replace(' - StackBlitz', '')
  return {
    file,
    lines,
    code,
    project,
  } satisfies Meta
})
const vnodeCode = computed(() => {
  if (!meta.value.code)
    return null
  const code = meta.value.code
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/`/g, '&#96;')

  const vnode = contentToVNode(`<p>\`\`\`${meta.value.file?.split('.')?.[1] ?? ''}\n${code}\n\`\`\`\</p>`, {
    markdown: true,
  })
  return vnode
})

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_StatusPreviewCardNormal = _resolveComponent("StatusPreviewCardNormal")

  return (meta.value.code)
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        flex: "",
        "flex-col": "",
        "gap-1": "",
        "display-block": "",
        "of-hidden": "",
        "w-full": "",
        "rounded-lg": "",
        "overflow-hidden": "",
        "pb-2": ""
      }, [ _createElementVNode("div", {
          "whitespace-pre-wrap": "",
          "break-words": ""
        }, [ (vnodeCode.value) ? (_openBlock(), _createElementBlock("span", {
              key: 0,
              class: "content-rich line-compact",
              dir: "auto"
            }, [ _createVNode(_resolveDynamicComponent(vnodeCode.value)) ])) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", {
          flex: "",
          "justify-between": "",
          "display-block": "",
          "of-hidden": "",
          "bg-card": "",
          "w-full": "",
          "p-3": "",
          "pb-4": ""
        }, [ _createElementVNode("div", {
            flex: "",
            "flex-col": ""
          }, [ _createElementVNode("p", {
              flex: "",
              "gap-1": ""
            }, [ _createElementVNode("span", null, _toDisplayString(_ctx.$t('custom_cards.stackblitz.snippet_from', [meta.value.file])), 1 /* TEXT */), _createElementVNode("span", _hoisted_1, _toDisplayString(`- ${_ctx.$t('custom_cards.stackblitz.lines', [meta.value.lines])}`), 1 /* TEXT */) ]), _createElementVNode("div", {
              flex: "",
              "font-bold": "",
              "gap-2": ""
            }, [ _createElementVNode("span", _hoisted_2, _toDisplayString(meta.value.project), 1 /* TEXT */), _createElementVNode("span", {
                flex: "",
                "text-secondary": ""
              }, [ _createElementVNode("span", {
                  flex: "",
                  "items-center": ""
                }, [ _createElementVNode("svg", {
                    "h-5": "",
                    width: "22.27",
                    height: "32",
                    viewBox: "0 0 256 368"
                  }, [ _hoisted_3 ]) ]), _hoisted_4 ]) ]) ]), _createVNode(_component_NuxtLink, {
            external: "",
            target: "_blank",
            "btn-solid": "",
            "pt-0": "",
            "pb-1": "",
            "px-2": "",
            "h-fit": "",
            to: __props.card.url
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_ctx.$t('custom_cards.stackblitz.open')), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["to"]) ]) ]))
      : (_openBlock(), _createBlock(_component_StatusPreviewCardNormal, {
        key: 1,
        card: __props.card,
        "small-picture-only": __props.smallPictureOnly,
        root: __props.root
      }, null, 8 /* PROPS */, ["card", "small-picture-only", "root"]))
}
}

})
