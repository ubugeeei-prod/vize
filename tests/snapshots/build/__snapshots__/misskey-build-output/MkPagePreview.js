import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import * as Misskey from 'misskey-js'
import { userName } from '@/filters/user.js'
import MediaImage from '@/components/MkMediaImage.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkPagePreview',
  props: {
    page: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(), _createBlock(_component_MkA, {
      to: `/@${__props.page.user.username}/pages/${__props.page.name}`,
      class: "vhpxefrj"
    }, {
      default: _withCtx(() => [
        (__props.page.eyeCatchingImage)
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "thumbnail"
          }, [
            _createVNode(MediaImage, {
              image: __props.page.eyeCatchingImage,
              disableImageLink: true,
              controls: false,
              cover: true,
              class: _normalizeClass(_ctx.$style.eyeCatchingImageRoot)
            }, null, 8 /* PROPS */, ["image", "disableImageLink", "controls", "cover"])
          ]))
          : _createCommentVNode("v-if", true),
        _createElementVNode("article", null, [
          _createElementVNode("header", null, [
            _createElementVNode("h1", { title: __props.page.title }, _toDisplayString(__props.page.title), 9 /* TEXT, PROPS */, ["title"])
          ]),
          (__props.page.summary)
            ? (_openBlock(), _createElementBlock("p", {
              key: 0,
              title: __props.page.summary
            }, _toDisplayString(__props.page.summary.length > 85 ? __props.page.summary.slice(0, 85) + '…' : __props.page.summary), 1 /* TEXT */))
            : _createCommentVNode("v-if", true),
          _createElementVNode("footer", null, [
            (__props.page.user.avatarUrl)
              ? (_openBlock(), _createElementBlock("img", {
                key: 0,
                class: "icon",
                src: __props.page.user.avatarUrl
              }))
              : _createCommentVNode("v-if", true),
            _createElementVNode("p", null, _toDisplayString(_unref(userName)(__props.page.user)), 1 /* TEXT */)
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["to"]))
}
}

})
