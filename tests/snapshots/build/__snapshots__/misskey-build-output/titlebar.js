import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-arrow-left" });
import { host } from "@@/js/config.js";
import { ref } from "vue";
import { instance } from "@/instance.js";
export default {
  __name: "titlebar",
  setup(__props) {
    const canBack = ref(true);
    function goBack() {
      window.history.back();
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(_ctx.$style.root) },
        [_createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.title) },
          [_createElementVNode("img", {
            src: _unref(instance).iconUrl || "/favicon.ico",
            alt: "",
            class: _normalizeClass(_ctx.$style.instanceIcon)
          }, null, 10, ["src"]), _createElementVNode(
            "span",
            { class: _normalizeClass(_ctx.$style.instanceTitle) },
            _toDisplayString(_unref(instance).name ?? _unref(host)),
            3
            /* TEXT, CLASS */
          )],
          2
          /* CLASS */
        ), _createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.controls) },
          [_createElementVNode(
            "span",
            { class: _normalizeClass(_ctx.$style.left) },
            [canBack.value ? (_openBlock(), _createElementBlock(
              "button",
              {
                key: 0,
                class: _normalizeClass(["_button", _ctx.$style.button]),
                onClick: goBack
              },
              [_hoisted_1],
              2
              /* CLASS */
            )) : _createCommentVNode("v-if", true)],
            2
            /* CLASS */
          ), _createElementVNode(
            "span",
            { class: _normalizeClass(_ctx.$style.right) },
            null,
            2
            /* CLASS */
          )],
          2
          /* CLASS */
        )],
        2
        /* CLASS */
      );
    };
  }
};
