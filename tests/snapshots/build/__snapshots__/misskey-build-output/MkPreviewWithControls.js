import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderSlot as _renderSlot, normalizeClass as _normalizeClass, unref as _unref } from "vue";
import { prefer } from "@/preferences.js";
export default {
  __name: "MkPreviewWithControls",
  props: { previewLoading: {
    type: Boolean,
    required: false,
    default: false
  } },
  setup(__props) {
    const props = __props;
    return (_ctx, _cache) => {
      const _component_MkLoading = _resolveComponent("MkLoading");
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(_ctx.$style.root) },
        [_createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.container) },
          [_createElementVNode(
            "div",
            { class: _normalizeClass([_ctx.$style.preview, _unref(prefer).s.animation ? _ctx.$style.animatedBg : null]) },
            [_createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.previewContent) },
              [_renderSlot(_ctx.$slots, "preview")],
              2
              /* CLASS */
            ), __props.previewLoading ? (_openBlock(), _createElementBlock(
              "div",
              {
                key: 0,
                class: _normalizeClass(_ctx.$style.previewLoading)
              },
              [_createVNode(_component_MkLoading)],
              2
              /* CLASS */
            )) : _createCommentVNode("v-if", true)],
            2
            /* CLASS */
          ), _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.controls) },
            [_renderSlot(_ctx.$slots, "controls")],
            2
            /* CLASS */
          )],
          2
          /* CLASS */
        )],
        2
        /* CLASS */
      );
    };
  }
};
