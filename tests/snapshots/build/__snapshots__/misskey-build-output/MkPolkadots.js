import { openBlock as _openBlock, createElementBlock as _createElementBlock, normalizeClass as _normalizeClass } from "vue";
export default {
  __name: "MkPolkadots",
  props: {
    accented: {
      type: Boolean,
      required: false,
      default: false
    },
    revered: {
      type: Boolean,
      required: false,
      default: false
    },
    height: {
      type: Number,
      required: false,
      default: 200
    }
  },
  setup(__props) {
    const props = __props;
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass([
          _ctx.$style.root,
          __props.accented ? _ctx.$style.accented : null,
          __props.revered ? _ctx.$style.revered : null
        ]) },
        null,
        2
        /* CLASS */
      );
    };
  }
};
