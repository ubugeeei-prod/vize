import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, normalizeClass as _normalizeClass } from "vue";
import XNotification from "@/components/MkNotification.vue";
export default {
  __name: "notification",
  props: { notification: {
    type: null,
    required: true
  } },
  setup(__props) {
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(_ctx.$style.root) },
        [_createVNode(XNotification, {
          notification: __props.notification,
          class: "notification _acrylic",
          full: false
        }, null, 8, ["notification", "full"])],
        2
        /* CLASS */
      );
    };
  }
};
