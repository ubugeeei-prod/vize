import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = { style: "color: #41b781;" };
import { ref } from "vue";
import { useWidgetPropsManager } from "./widget.js";
import { misskeyApiGet } from "@/utility/misskey-api.js";
import { useInterval } from "@@/js/use-interval.js";
import { i18n } from "@/i18n.js";
import number from "@/filters/number.js";
const name = "onlineUsers";
export default {
  __name: "WidgetOnlineUsers",
  setup(__props, { expose: __expose, emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const widgetPropsDef = { transparent: {
      type: "boolean",
      label: i18n.ts._widgetOptions.transparent,
      default: true
    } };
    const { widgetProps, configure } = useWidgetPropsManager(name, widgetPropsDef, props, emit);
    const onlineUsersCount = ref(0);
    const tick = () => {
      misskeyApiGet("get-online-users-count").then((res) => {
        onlineUsersCount.value = res.count;
      });
    };
    useInterval(tick, 1e3 * 15, {
      immediate: true,
      afterMounted: true
    });
    __expose({
      name,
      configure,
      id: props.widget ? props.widget.id : null
    });
    return (_ctx, _cache) => {
      const _component_I18n = _resolveComponent("I18n");
      return _openBlock(), _createElementBlock(
        "div",
        {
          "data-cy-mkw-onlineUsers": "",
          class: _normalizeClass([_ctx.$style.root, {
            _panel: !_unref(widgetProps).transparent,
            [_ctx.$style.pad]: !_unref(widgetProps).transparent
          }])
        },
        [_createElementVNode(
          "span",
          { class: _normalizeClass(_ctx.$style.text) },
          [onlineUsersCount.value ? (_openBlock(), _createBlock(_component_I18n, {
            key: 0,
            src: _unref(i18n).ts.onlineUsersCount,
            textTag: "span"
          }, {
            n: _withCtx(() => [_createElementVNode(
              "b",
              _hoisted_1,
              _toDisplayString(number(onlineUsersCount.value)),
              1
              /* TEXT */
            )]),
            _: 1
          }, 8, ["src"])) : _createCommentVNode("v-if", true)],
          2
          /* CLASS */
        )],
        2
        /* CLASS */
      );
    };
  }
};
