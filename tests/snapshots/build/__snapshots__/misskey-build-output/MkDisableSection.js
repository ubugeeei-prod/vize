import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue";
export default {
  __name: "MkDisableSection",
  props: { disabled: {
    type: Boolean,
    required: false
  } },
  setup(__props) {
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass([_ctx.$style.root]) },
        [_createElementVNode("div", {
          inert: __props.disabled,
          class: _normalizeClass([{ [_ctx.$style.disabled]: __props.disabled }])
        }, [_renderSlot(_ctx.$slots, "default")], 10, ["inert"]), __props.disabled ? (_openBlock(), _createElementBlock(
          "div",
          {
            key: 0,
            class: _normalizeClass([_ctx.$style.cover])
          },
          null,
          2
          /* CLASS */
        )) : _createCommentVNode("v-if", true)],
        2
        /* CLASS */
      );
    };
  }
};
