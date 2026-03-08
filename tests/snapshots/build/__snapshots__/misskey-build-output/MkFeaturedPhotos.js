import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, unref as _unref } from "vue"

import { instance } from '@/instance.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkFeaturedPhotos',
  setup(__props) {


return (_ctx: any,_cache: any) => {
  return (_unref(instance))
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        class: _normalizeClass(_ctx.$style.root),
        style: _normalizeStyle({ backgroundImage: `url(${ _unref(instance).backgroundImageUrl })` })
      }))
      : _createCommentVNode("v-if", true)
}
}

})
