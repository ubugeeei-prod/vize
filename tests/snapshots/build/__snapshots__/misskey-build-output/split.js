import { openBlock as _openBlock, createElementBlock as _createElementBlock, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue";
import { provide } from "vue";
export default {
  __name: "split",
  props: { minWidth: {
    type: Number,
    required: false,
    default: 210
  } },
  setup(__props) {
    const props = __props;
    provide("splited", true);
    const minWidth = props.minWidth + "px";
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(_ctx.$style.root) },
        [_renderSlot(_ctx.$slots, "default")],
        2
        /* CLASS */
      );
    };
  }
};
