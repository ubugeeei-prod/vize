import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, withCtx as _withCtx, unref as _unref } from "vue"

import { definePage } from '@/page.js'
import { i18n } from '@/i18n.js'
import { instance } from '@/instance.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'ads',
  setup(__props) {

definePage(() => ({
	title: i18n.ts.ads,
	icon: 'ti ti-ad',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkAd = _resolveComponent("MkAd")
  const _component_MkResult = _resolveComponent("MkResult")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, null, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 500px;"
        }, [
          (_unref(instance).ads.length > 0)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "_gaps"
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(instance).ads, (ad) => {
                return (_openBlock(), _createBlock(_component_MkAd, {
                  key: ad.id,
                  specify: ad
                }, null, 8 /* PROPS */, ["specify"]))
              }), 128 /* KEYED_FRAGMENT */))
            ]))
            : (_openBlock(), _createBlock(_component_MkResult, {
              key: 1,
              type: "empty"
            }))
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
