import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, normalizeClass as _normalizeClass, unref as _unref } from "vue"

import { defineAsyncComponent } from 'vue'
import * as mfm from 'mfm-js'
import * as Misskey from 'misskey-js'
import { extractUrlFromMfm } from '@/utility/extract-url-from-mfm.js'
import { isEnabledUrlPreview } from '@/utility/url-preview.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'page.text',
  props: {
    block: { type: null, required: true },
    page: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const MkUrlPreview = defineAsyncComponent(() => import('@/components/MkUrlPreview.vue'));
const urls = props.block.text ? extractUrlFromMfm(mfm.parse(props.block.text)) : [];

return (_ctx: any,_cache: any) => {
  const _component_Mfm = _resolveComponent("Mfm")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["_gaps", _ctx.$style.textRoot])
    }, [ _createVNode(_component_Mfm, {
        text: __props.block.text ?? '',
        isNote: false
      }, null, 8 /* PROPS */, ["text", "isNote"]), (_unref(isEnabledUrlPreview)) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "_gaps_s"
        }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(urls), (url) => {
            return (_openBlock(), _createBlock(MkUrlPreview, {
              key: url,
              url: url
            }, null, 8 /* PROPS */, ["url"]))
          }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true) ]))
}
}

})
