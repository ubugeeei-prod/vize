import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", {
  class: "ti ti-plane-departure",
  style: "margin-right: 8px;"
});
import { ref } from "vue";
import MkMention from "./MkMention.vue";
import { i18n } from "@/i18n.js";
import { host as localHost } from "@@/js/config.js";
import { misskeyApi } from "@/utility/misskey-api.js";
export default {
  __name: "MkAccountMoved",
  props: { movedTo: {
    type: String,
    required: true
  } },
  setup(__props) {
    const props = __props;
    const user = ref();
    misskeyApi("users/show", { userId: props.movedTo }).then((u) => user.value = u);
    return (_ctx, _cache) => {
      return user.value ? (_openBlock(), _createElementBlock(
        "div",
        {
          key: 0,
          class: _normalizeClass(_ctx.$style.root)
        },
        [
          _hoisted_1,
          _createTextVNode(
            " " + _toDisplayString(_unref(i18n).ts.accountMoved) + " ",
            1
            /* TEXT */
          ),
          _createVNode(MkMention, {
            class: _normalizeClass(_ctx.$style.link),
            username: user.value.username,
            host: user.value.host ?? _unref(localHost)
          }, null, 10, ["username", "host"])
        ],
        2
        /* CLASS */
      )) : _createCommentVNode("v-if", true);
    };
  }
};
