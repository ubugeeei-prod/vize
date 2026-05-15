import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-bell" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-settings" });
import { useWidgetPropsManager } from "./widget.js";
import MkContainer from "@/components/MkContainer.vue";
import MkStreamingNotificationsTimeline from "@/components/MkStreamingNotificationsTimeline.vue";
import * as os from "@/os.js";
import { i18n } from "@/i18n.js";
const name = "notifications";
export default {
  __name: "WidgetNotifications",
  setup(__props, { expose: __expose, emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const widgetPropsDef = {
      showHeader: {
        type: "boolean",
        label: i18n.ts._widgetOptions.showHeader,
        default: true
      },
      height: {
        type: "number",
        label: i18n.ts.height,
        default: 300
      },
      excludeTypes: {
        type: "array",
        hidden: true,
        default: []
      }
    };
    const { widgetProps, configure, save } = useWidgetPropsManager(name, widgetPropsDef, props, emit);
    const configureNotification = async () => {
      const { dispose } = await os.popupAsyncWithDialog(import("@/components/MkNotificationSelectWindow.vue").then((x) => x.default), { excludeTypes: widgetProps.excludeTypes }, {
        done: async (res) => {
          const { excludeTypes } = res;
          widgetProps.excludeTypes = excludeTypes;
          save();
        },
        closed: () => dispose()
      });
    };
    __expose({
      name,
      configure,
      id: props.widget ? props.widget.id : null
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkContainer, {
        style: _normalizeStyle(`height: ${_unref(widgetProps).height}px;`),
        showHeader: _unref(widgetProps).showHeader,
        scrollable: true,
        "data-cy-mkw-notifications": "",
        class: "mkw-notifications"
      }, {
        icon: _withCtx(() => [_hoisted_1]),
        header: _withCtx(() => [_createTextVNode(
          _toDisplayString(_unref(i18n).ts.notifications),
          1
          /* TEXT */
        )]),
        func: _withCtx(({ buttonStyleClass }) => [_createElementVNode("button", {
          class: _normalizeClass(["_button", buttonStyleClass]),
          onClick: ($event) => configureNotification()
        }, [_hoisted_2], 10, ["onClick"])]),
        default: _withCtx(() => [_createElementVNode("div", null, [_createVNode(MkStreamingNotificationsTimeline, { excludeTypes: _unref(widgetProps).excludeTypes }, null, 8, ["excludeTypes"])])]),
        _: 1
      }, 12, ["showHeader", "scrollable"]);
    };
  }
};
