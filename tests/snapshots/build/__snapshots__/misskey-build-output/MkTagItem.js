import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue";
import MkButton from "@/components/MkButton.vue";
export default {
  __name: "MkTagItem",
  props: {
    iconClass: {
      type: String,
      required: false
    },
    content: {
      type: String,
      required: true
    },
    exButtonIconClass: {
      type: String,
      required: false
    }
  },
  emits: ["click", "exButtonClick"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        {
          class: _normalizeClass(_ctx.$style.root),
          onClick: _cache[0] || (_cache[0] = (ev) => emit("click", ev))
        },
        [
          __props.iconClass ? (_openBlock(), _createElementBlock(
            "span",
            {
              key: 0,
              class: _normalizeClass([_ctx.$style.icon, __props.iconClass])
            },
            null,
            2
            /* CLASS */
          )) : _createCommentVNode("v-if", true),
          _createElementVNode(
            "span",
            null,
            _toDisplayString(__props.content),
            1
            /* TEXT */
          ),
          __props.exButtonIconClass ? (_openBlock(), _createBlock(
            MkButton,
            {
              key: 0,
              class: _normalizeClass(_ctx.$style.exButton),
              onClick: _cache[1] || (_cache[1] = (ev) => emit("exButtonClick", ev))
            },
            {
              default: _withCtx(() => [_createElementVNode(
                "span",
                { class: _normalizeClass([_ctx.$style.exButtonIcon, __props.exButtonIconClass]) },
                null,
                2
                /* CLASS */
              )]),
              _: 1
            },
            2
            /* CLASS */
          )) : _createCommentVNode("v-if", true)
        ],
        2
        /* CLASS */
      );
    };
  }
};
