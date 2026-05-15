import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-chart-line" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-dots" });
import { ref } from "vue";
import MkContainer from "@/components/MkContainer.vue";
import MkChart from "@/components/MkChart.vue";
import * as os from "@/os.js";
import { i18n } from "@/i18n.js";
export default {
  __name: "index.activity",
  props: {
    user: {
      type: null,
      required: true
    },
    limit: {
      type: Number,
      required: false,
      default: 50
    }
  },
  setup(__props) {
    const props = __props;
    const chartSrc = ref("per-user-notes");
    function showMenu(ev) {
      os.popupMenu([{
        text: i18n.ts.notes,
        active: chartSrc.value === "per-user-notes",
        action: () => {
          chartSrc.value = "per-user-notes";
        }
      }, {
        text: i18n.ts.numberOfProfileView,
        active: chartSrc.value === "per-user-pv",
        action: () => {
          chartSrc.value = "per-user-pv";
        }
      }], ev.currentTarget ?? ev.target);
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkContainer, null, {
        icon: _withCtx(() => [_hoisted_1]),
        header: _withCtx(() => [_createTextVNode(
          _toDisplayString(_unref(i18n).ts.activity),
          1
          /* TEXT */
        )]),
        func: _withCtx(({ buttonStyleClass }) => [_createElementVNode(
          "button",
          {
            class: _normalizeClass(["_button", buttonStyleClass]),
            onClick: showMenu
          },
          [_hoisted_2],
          2
          /* CLASS */
        )]),
        default: _withCtx(() => [_createElementVNode("div", { style: "padding: 8px;" }, [_createVNode(MkChart, {
          src: chartSrc.value,
          args: {
            user: __props.user,
            withoutAll: true
          },
          span: "day",
          limit: __props.limit,
          bar: true,
          stacked: true,
          detailed: false,
          aspectRatio: 5
        }, null, 8, [
          "src",
          "args",
          "limit",
          "bar",
          "stacked",
          "detailed",
          "aspectRatio"
        ])])]),
        _: 1
      });
    };
  }
};
