import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode } from "vue";
import XValue from "./MkObjectView.value.vue";
export default {
  __name: "MkObjectView",
  props: { value: {
    type: null,
    required: true
  } },
  setup(__props) {
    const props = __props;
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock("div", null, [_createVNode(XValue, {
        value: __props.value,
        collapsed: false
      }, null, 8, ["value", "collapsed"])]);
    };
  }
};
