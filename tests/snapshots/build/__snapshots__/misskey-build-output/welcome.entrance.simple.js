import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, normalizeClass as _normalizeClass, unref as _unref } from "vue";
import MkFeaturedPhotos from "@/components/MkFeaturedPhotos.vue";
import misskeysvg from "/client-assets/misskey.svg";
import MkVisitorDashboard from "@/components/MkVisitorDashboard.vue";
import { instance as meta } from "@/instance.js";
export default {
  __name: "welcome.entrance.simple",
  setup(__props) {
    return (_ctx, _cache) => {
      return _unref(meta) ? (_openBlock(), _createElementBlock(
        "div",
        {
          key: 0,
          class: _normalizeClass(_ctx.$style.root)
        },
        [
          _createVNode(
            MkFeaturedPhotos,
            { class: _normalizeClass(_ctx.$style.bg) },
            null,
            2
            /* CLASS */
          ),
          _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.logoWrapper) },
            [_createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.poweredBy) },
              "Powered by",
              2
              /* CLASS */
            ), _createElementVNode("img", {
              src: misskeysvg,
              class: _normalizeClass(_ctx.$style.misskey)
            }, null, 10, ["src"])],
            2
            /* CLASS */
          ),
          _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.contents) },
            [_createVNode(MkVisitorDashboard)],
            2
            /* CLASS */
          )
        ],
        2
        /* CLASS */
      )) : _createCommentVNode("v-if", true);
    };
  }
};
