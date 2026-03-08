import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { $i } from '@/i.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'announcements',
  setup(__props) {


return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")

  return (_unref($i))
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        class: _normalizeClass(_ctx.$style.root)
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref($i).unreadAnnouncements.filter(x => x.display === 'banner'), (announcement) => {
          return (_openBlock(), _createBlock(_component_MkA, {
            key: announcement.id,
            class: _normalizeClass(_ctx.$style.item),
            to: `/announcements/${announcement.id}`
          }, {
            default: _withCtx(() => [
              _createElementVNode("span", {
                class: _normalizeClass(_ctx.$style.icon)
              }, [
                (announcement.icon === 'info')
                  ? (_openBlock(), _createElementBlock("i", {
                    key: 0,
                    class: "ti ti-info-circle"
                  }))
                  : (announcement.icon === 'warning')
                    ? (_openBlock(), _createElementBlock("i", {
                      key: 1,
                      class: "ti ti-alert-triangle",
                      style: "color: var(--MI_THEME-warn);"
                    }))
                  : (announcement.icon === 'error')
                    ? (_openBlock(), _createElementBlock("i", {
                      key: 2,
                      class: "ti ti-circle-x",
                      style: "color: var(--MI_THEME-error);"
                    }))
                  : (announcement.icon === 'success')
                    ? (_openBlock(), _createElementBlock("i", {
                      key: 3,
                      class: "ti ti-check",
                      style: "color: var(--MI_THEME-success);"
                    }))
                  : _createCommentVNode("v-if", true)
              ]),
              _createElementVNode("span", {
                class: _normalizeClass(_ctx.$style.title)
              }, _toDisplayString(announcement.title), 1 /* TEXT */),
              _createElementVNode("span", {
                class: _normalizeClass(_ctx.$style.body)
              }, _toDisplayString(announcement.text), 1 /* TEXT */)
            ]),
            _: 2 /* DYNAMIC */
          }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["to"]))
        }), 128 /* KEYED_FRAGMENT */)) ]))
      : _createCommentVNode("v-if", true)
}
}

})
