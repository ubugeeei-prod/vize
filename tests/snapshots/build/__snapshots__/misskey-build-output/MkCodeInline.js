import { openBlock as _openBlock, createElementBlock as _createElementBlock, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue";
export default {
  __name: "MkCodeInline",
  props: { code: {
    type: String,
    required: true
  } },
  setup(__props) {
    const props = __props;
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "code",
        { class: _normalizeClass(_ctx.$style.root) },
        _toDisplayString(__props.code),
        3
        /* TEXT, CLASS */
      );
    };
  }
};
