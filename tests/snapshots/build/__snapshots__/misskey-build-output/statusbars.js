import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"

import { defineAsyncComponent } from 'vue'
import { instance } from '@/instance.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'statusbars',
  setup(__props) {

const XRss = defineAsyncComponent(() => import('./statusbar-rss.vue'));
const XFederation = defineAsyncComponent(() => import('./statusbar-federation.vue'));
const XUserList = defineAsyncComponent(() => import('./statusbar-user-list.vue'));

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(prefer).r.statusbars.value, (x) => {
        return (_openBlock(), _createElementBlock("div", {
          key: x.id,
          class: _normalizeClass([_ctx.$style.item, { [_ctx.$style.black]: x.black,
  			[_ctx.$style.verySmall]: x.size === 'verySmall',
  			[_ctx.$style.small]: x.size === 'small',
  			[_ctx.$style.large]: x.size === 'large',
  			[_ctx.$style.veryLarge]: x.size === 'veryLarge',
  		}])
        }, [
          _createElementVNode("span", {
            class: _normalizeClass(_ctx.$style.name)
          }, _toDisplayString(x.name), 1 /* TEXT */),
          (x.type === 'rss')
            ? (_openBlock(), _createBlock(XRss, {
              key: 0,
              class: _normalizeClass(_ctx.$style.body),
              refreshIntervalSec: x.props.refreshIntervalSec,
              marqueeDuration: x.props.marqueeDuration,
              marqueeReverse: x.props.marqueeReverse,
              display: x.props.display,
              url: x.props.url,
              shuffle: x.props.shuffle
            }, null, 8 /* PROPS */, ["refreshIntervalSec", "marqueeDuration", "marqueeReverse", "display", "url", "shuffle"]))
            : (x.type === 'federation' && _unref(instance).federation !== 'none')
              ? (_openBlock(), _createBlock(XFederation, {
                key: 1,
                class: _normalizeClass(_ctx.$style.body),
                refreshIntervalSec: x.props.refreshIntervalSec,
                marqueeDuration: x.props.marqueeDuration,
                marqueeReverse: x.props.marqueeReverse,
                display: x.props.display,
                colored: x.props.colored
              }, null, 8 /* PROPS */, ["refreshIntervalSec", "marqueeDuration", "marqueeReverse", "display", "colored"]))
            : (x.type === 'userList')
              ? (_openBlock(), _createBlock(XUserList, {
                key: 2,
                class: _normalizeClass(_ctx.$style.body),
                refreshIntervalSec: x.props.refreshIntervalSec,
                marqueeDuration: x.props.marqueeDuration,
                marqueeReverse: x.props.marqueeReverse,
                display: x.props.display,
                userListId: x.props.userListId
              }, null, 8 /* PROPS */, ["refreshIntervalSec", "marqueeDuration", "marqueeReverse", "display", "userListId"]))
            : _createCommentVNode("v-if", true)
        ], 2 /* CLASS */))
      }), 128 /* KEYED_FRAGMENT */)) ]))
}
}

})
