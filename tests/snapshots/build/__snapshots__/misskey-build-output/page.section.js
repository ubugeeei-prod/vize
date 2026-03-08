import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveDynamicComponent as _resolveDynamicComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"

import { defineAsyncComponent } from 'vue'
import * as Misskey from 'misskey-js'

export default /*@__PURE__*/_defineComponent({
  __name: 'page.section',
  props: {
    block: { type: null, required: true },
    h: { type: Number, required: true },
    page: { type: null, required: true }
  },
  setup(__props: any) {

const XBlock = defineAsyncComponent(() => import('./page.block.vue'));

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("section", null, [ _createVNode(_resolveDynamicComponent('h' + __props.h), {
        class: _normalizeClass({
  			'h2': __props.h === 2,
  			'h3': __props.h === 3,
  			'h4': __props.h === 4,
  		})
      }, {
        default: _withCtx(() => [
          _createTextVNode(_toDisplayString(__props.block.title), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 2 /* CLASS */), _createElementVNode("div", { class: "_gaps" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.block.children, (child) => {
          return (_openBlock(), _createBlock(XBlock, {
            key: child.id,
            page: __props.page,
            block: child,
            h: __props.h + 1
          }, null, 8 /* PROPS */, ["page", "block", "h"]))
        }), 128 /* KEYED_FRAGMENT */)) ]) ]))
}
}

})
