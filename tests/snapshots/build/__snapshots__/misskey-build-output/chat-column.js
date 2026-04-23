import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", {
  class: "ti ti-messages",
  style: "margin-right: 8px;"
});
import { ensureSignin } from "@/i.js";
import { i18n } from "../../i18n.js";
import XColumn from "./column.vue";
import MkInfo from "@/components/MkInfo.vue";
import MkChatHistories from "@/components/MkChatHistories.vue";
export default {
  __name: "chat-column",
  props: {
    column: {
      type: null,
      required: true
    },
    isStacked: {
      type: Boolean,
      required: true
    }
  },
  setup(__props) {
    const $i = ensureSignin();
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(XColumn, {
        column: __props.column,
        isStacked: __props.isStacked
      }, {
        header: _withCtx(() => [_hoisted_1, _createTextVNode(
          _toDisplayString(__props.column.name || _unref(i18n).ts._deck._columns.chat),
          1
          /* TEXT */
        )]),
        default: _withCtx(() => [_createElementVNode("div", {
          style: "padding: 8px;",
          class: "_gaps"
        }, [_unref($i).policies.chatAvailability === "readonly" ? (_openBlock(), _createBlock(MkInfo, { key: 0 }, {
          default: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts._chat.chatIsReadOnlyForThisAccountOrServer),
            1
            /* TEXT */
          )]),
          _: 1
        })) : _unref($i).policies.chatAvailability === "unavailable" ? (_openBlock(), _createBlock(MkInfo, {
          key: 1,
          warn: ""
        }, {
          default: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts._chat.chatNotAvailableForThisAccountOrServer),
            1
            /* TEXT */
          )]),
          _: 1
        })) : _createCommentVNode("v-if", true), _unref($i).policies.chatAvailability !== "unavailable" ? (_openBlock(), _createBlock(MkChatHistories, { key: 0 })) : _createCommentVNode("v-if", true)])]),
        _: 1
      }, 8, ["column", "isStacked"]);
    };
  }
};
