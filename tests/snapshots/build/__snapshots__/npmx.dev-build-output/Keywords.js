import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'Keywords',
  props: {
    keywords: { type: Array, required: false }
  },
  setup(__props: any) {

const { model } = useGlobalSearch()

return (_ctx: any,_cache: any) => {
  const _component_LinkBase = _resolveComponent("LinkBase")
  const _component_CollapsibleSection = _resolveComponent("CollapsibleSection")

  return (__props.keywords?.length)
      ? (_openBlock(), _createBlock(_component_CollapsibleSection, {
        key: 0,
        title: _ctx.$t('package.keywords_title'),
        id: "keywords"
      }, {
        default: _withCtx(() => [
          _createElementVNode("ul", { class: "flex flex-wrap gap-1.5 list-none m-0 p-1" }, [
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.keywords.slice(0, 15), (keyword) => {
              return (_openBlock(), _createElementBlock("li", { key: keyword }, [
                _createVNode(_component_LinkBase, {
                  variant: "button-secondary",
                  size: "small",
                  to: { name: 'search', query: { q: `keyword:${keyword}` } },
                  onClick: _cache[0] || (_cache[0] = ($event: any) => (model.value = `keyword:${keyword}`))
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(keyword), 1 /* TEXT */)
                  ]),
                  _: 2 /* DYNAMIC */
                }, 8 /* PROPS */, ["to"])
              ]))
            }), 128 /* KEYED_FRAGMENT */))
          ])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["title"]))
      : _createCommentVNode("v-if", true)
}
}

})
