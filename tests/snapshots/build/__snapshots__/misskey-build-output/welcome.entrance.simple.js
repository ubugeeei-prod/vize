import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, normalizeClass as _normalizeClass, unref as _unref } from "vue"

import MkFeaturedPhotos from '@/components/MkFeaturedPhotos.vue'
import misskeysvg from '/client-assets/misskey.svg'
import MkVisitorDashboard from '@/components/MkVisitorDashboard.vue'
import { instance as meta } from '@/instance.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'welcome.entrance.simple',
  setup(__props) {


return (_ctx: any,_cache: any) => {
  return (_unref(meta))
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        class: _normalizeClass(_ctx.$style.root)
      }, [ _createVNode(MkFeaturedPhotos, {
          class: _normalizeClass(_ctx.$style.bg)
        }), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.logoWrapper)
        }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.poweredBy)
          }, "Powered by"), _createElementVNode("img", {
            src: misskeysvg,
            class: _normalizeClass(_ctx.$style.misskey)
          }, null, 8 /* PROPS */, ["src"]) ]), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.contents)
        }, [ _createVNode(MkVisitorDashboard) ]) ]))
      : _createCommentVNode("v-if", true)
}
}

})
