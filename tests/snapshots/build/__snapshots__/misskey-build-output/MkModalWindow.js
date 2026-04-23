import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-x" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check" });
import { useTemplateRef } from "vue";
import MkModal from "@/components/MkModal.vue";
import MkButton from "@/components/MkButton.vue";
import { i18n } from "@/i18n";
import { deviceKind } from "@/utility/device-kind.js";
export default {
  __name: "MkModalWindow",
  props: {
    withOkButton: {
      type: Boolean,
      required: false,
      default: false
    },
    withCloseButton: {
      type: Boolean,
      required: false,
      default: true
    },
    okButtonDisabled: {
      type: Boolean,
      required: false,
      default: false
    },
    width: {
      type: Number,
      required: false,
      default: 400
    },
    height: {
      type: Number,
      required: false,
      default: 500
    }
  },
  emits: [
    "click",
    "close",
    "closed",
    "ok",
    "esc"
  ],
  setup(__props, { expose: __expose, emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const modal = useTemplateRef("modal");
    function close() {
      modal.value?.close();
    }
    function onBgClick() {
      emit("click");
    }
    __expose({ close });
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkModal, {
        ref_key: "modal",
        ref: modal,
        preferType: _unref(deviceKind) === "smartphone" ? "drawer" : "dialog",
        onClick: onBgClick,
        onClosed: _cache[0] || (_cache[0] = ($event) => emit("closed")),
        onEsc: _cache[1] || (_cache[1] = ($event) => emit("esc"))
      }, {
        default: _withCtx(({ type }) => [_createElementVNode(
          "div",
          {
            ref: "rootEl",
            class: _normalizeClass([_ctx.$style.root, type === "drawer" ? _ctx.$style.asDrawer : null]),
            style: _normalizeStyle({
              width: type === "drawer" ? "" : `${__props.width}px`,
              height: type === "drawer" ? "" : `min(${__props.height}px, 100%)`
            })
          },
          [
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.header) },
              [
                __props.withCloseButton ? (_openBlock(), _createElementBlock("button", {
                  key: 0,
                  class: _normalizeClass(["_button", _ctx.$style.headerButton]),
                  "data-cy-modal-window-close": "",
                  onClick: ($event) => emit("close")
                }, [_hoisted_1], 10, ["onClick"])) : _createCommentVNode("v-if", true),
                _createElementVNode(
                  "span",
                  { class: _normalizeClass(_ctx.$style.title) },
                  [_renderSlot(_ctx.$slots, "header")],
                  2
                  /* CLASS */
                ),
                __props.withOkButton ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  style: "padding: 0 16px; place-content: center;"
                }, [_createVNode(MkButton, {
                  primary: "",
                  gradate: "",
                  small: "",
                  rounded: "",
                  disabled: __props.okButtonDisabled,
                  onClick: ($event) => emit("ok")
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(
                      _toDisplayString(_unref(i18n).ts.done),
                      1
                      /* TEXT */
                    ),
                    _createTextVNode(" "),
                    _hoisted_2
                  ]),
                  _: 1
                }, 8, ["disabled", "onClick"])])) : _createCommentVNode("v-if", true)
              ],
              2
              /* CLASS */
            ),
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.body) },
              [_renderSlot(_ctx.$slots, "default")],
              2
              /* CLASS */
            ),
            _ctx.$slots.footer ? (_openBlock(), _createElementBlock(
              "div",
              {
                key: 0,
                class: _normalizeClass(_ctx.$style.footer)
              },
              [_renderSlot(_ctx.$slots, "footer")],
              2
              /* CLASS */
            )) : _createCommentVNode("v-if", true)
          ],
          6
          /* CLASS, STYLE */
        )]),
        _: 3
      }, 1032, ["preferType"]);
    };
  }
};
