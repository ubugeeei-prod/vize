import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderSlot as _renderSlot, normalizeClass as _normalizeClass, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-copy" });
import { copyToClipboard } from "@/utility/copy-to-clipboard.js";
import { i18n } from "@/i18n.js";
export default {
  __name: "MkKeyValue",
  props: {
    copy: {
      type: [String, null],
      required: false,
      default: null
    },
    oneline: {
      type: Boolean,
      required: false,
      default: false
    }
  },
  setup(__props) {
    const props = __props;
    const copy_ = () => {
      copyToClipboard(props.copy);
    };
    return (_ctx, _cache) => {
      const _directive_tooltip = _resolveDirective("tooltip");
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.oneline]: __props.oneline }]) },
        [_createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.key) },
          [_renderSlot(_ctx.$slots, "key")],
          2
          /* CLASS */
        ), _createElementVNode(
          "div",
          { class: _normalizeClass(["_selectable", _ctx.$style.value]) },
          [_renderSlot(_ctx.$slots, "value"), __props.copy ? _withDirectives((_openBlock(), _createElementBlock("button", {
            key: 0,
            class: "_textButton",
            style: "margin-left: 0.5em;",
            onClick: copy_
          }, [_hoisted_1])), [[_directive_tooltip, _unref(i18n).ts.copy]]) : _createCommentVNode("v-if", true)],
          2
          /* CLASS */
        )],
        2
        /* CLASS */
      );
    };
  }
};
