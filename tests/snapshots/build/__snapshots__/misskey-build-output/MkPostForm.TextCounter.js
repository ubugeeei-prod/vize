import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue";
import { computed } from "vue";
import { instance } from "@/instance.js";
import { i18n } from "@/i18n.js";
import number from "@/filters/number.js";
export default {
  __name: "MkPostForm.TextCounter",
  props: { textLength: {
    type: Number,
    required: true
  } },
  setup(__props) {
    const props = __props;
    const maxTextLength = computed(() => {
      return instance ? instance.maxNoteTextLength : 1e3;
    });
    const textCountPercentage = computed(() => {
      return props.textLength / maxTextLength.value * 100;
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass([_ctx.$style.textCountRoot]) },
        [_createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.textCountLabel) },
          _toDisplayString(_unref(i18n).ts.textCount),
          3
          /* TEXT, CLASS */
        ), _createElementVNode(
          "div",
          { class: _normalizeClass([
            _ctx.$style.textCount,
            { [_ctx.$style.danger]: textCountPercentage.value > 100 },
            { [_ctx.$style.warning]: textCountPercentage.value > 90 && textCountPercentage.value <= 100 }
          ]) },
          [_createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.textCountGraph) },
            null,
            2
            /* CLASS */
          ), _createElementVNode("div", null, [_createElementVNode(
            "span",
            { class: _normalizeClass(_ctx.$style.textCountCurrent) },
            _toDisplayString(number(__props.textLength)),
            3
            /* TEXT, CLASS */
          ), _createTextVNode(
            " / " + _toDisplayString(number(maxTextLength.value)),
            1
            /* TEXT */
          )])],
          2
          /* CLASS */
        )],
        2
        /* CLASS */
      );
    };
  }
};
