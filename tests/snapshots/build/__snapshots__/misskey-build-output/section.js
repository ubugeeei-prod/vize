import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue";
export default {
  __name: "section",
  props: { first: {
    type: Boolean,
    required: false
  } },
  setup(__props) {
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.rootFirst]: __props.first }]) },
        [
          _createElementVNode(
            "div",
            { class: _normalizeClass([_ctx.$style.label, { [_ctx.$style.labelFirst]: __props.first }]) },
            [_renderSlot(_ctx.$slots, "label")],
            2
            /* CLASS */
          ),
          _createElementVNode(
            "div",
            { class: _normalizeClass([_ctx.$style.description]) },
            [_renderSlot(_ctx.$slots, "description")],
            2
            /* CLASS */
          ),
          _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.main) },
            [_renderSlot(_ctx.$slots, "default")],
            2
            /* CLASS */
          )
        ],
        2
        /* CLASS */
      );
    };
  }
};
