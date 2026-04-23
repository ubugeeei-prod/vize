import { openBlock as _openBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, unref as _unref } from "vue";
import { instance } from "@/instance.js";
export default {
  __name: "MkFeaturedPhotos",
  setup(__props) {
    return (_ctx, _cache) => {
      return _unref(instance) ? (_openBlock(), _createElementBlock(
        "div",
        {
          key: 0,
          class: _normalizeClass(_ctx.$style.root),
          style: _normalizeStyle({ backgroundImage: `url(${_unref(instance).backgroundImageUrl})` })
        },
        null,
        6
        /* CLASS, STYLE */
      )) : _createCommentVNode("v-if", true);
    };
  }
};
