import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue";
export default {
  __name: "slot",
  setup(__props) {
    function focus() {
      // TODO
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock("div", null, [
        _createElementVNode(
          "div",
          {
            class: _normalizeClass(_ctx.$style.label),
            onClick: focus
          },
          [_renderSlot(_ctx.$slots, "label")],
          2
          /* CLASS */
        ),
        _createElementVNode("div", null, [_renderSlot(_ctx.$slots, "default")]),
        _createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.caption) },
          [_renderSlot(_ctx.$slots, "caption")],
          2
          /* CLASS */
        )
      ]);
    };
  }
};
