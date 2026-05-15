import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, normalizeClass as _normalizeClass } from "vue";
import MkRetentionHeatmap from "@/components/MkRetentionHeatmap.vue";
export default {
  __name: "overview.retention",
  setup(__props) {
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(["_panel", _ctx.$style.root]) },
        [_createVNode(MkRetentionHeatmap)],
        2
        /* CLASS */
      );
    };
  }
};
