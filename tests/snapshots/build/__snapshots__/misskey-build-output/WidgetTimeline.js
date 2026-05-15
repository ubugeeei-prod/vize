import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-minus" });
import { ref, computed } from "vue";
import { useWidgetPropsManager } from "./widget.js";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import MkContainer from "@/components/MkContainer.vue";
import MkStreamingNotesTimeline from "@/components/MkStreamingNotesTimeline.vue";
import { i18n } from "@/i18n.js";
import { availableBasicTimelines, isAvailableBasicTimeline, isBasicTimeline, basicTimelineIconClass } from "@/timelines.js";
const name = "timeline";
export default {
  __name: "WidgetTimeline",
  setup(__props, { expose: __expose, emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const widgetPropsDef = {
      showHeader: {
        type: "boolean",
        default: true
      },
      height: {
        type: "number",
        default: 300
      },
      src: {
        type: "string",
        default: "home",
        hidden: true
      },
      antenna: {
        type: "object",
        default: null,
        hidden: true
      },
      list: {
        type: "object",
        default: null,
        hidden: true
      }
    };
    const { widgetProps, configure, save } = useWidgetPropsManager(name, widgetPropsDef, props, emit);
    const menuOpened = ref(false);
    const headerTitle = computed(() => {
      if (widgetProps.src === "list") {
        return widgetProps.list != null ? widgetProps.list.name : "?";
      } else if (widgetProps.src === "antenna") {
        return widgetProps.antenna != null ? widgetProps.antenna.name : "?";
      } else {
        return i18n.ts._timelines[widgetProps.src] ?? "?";
      }
    });
    const setSrc = (src) => {
      widgetProps.src = src;
      save();
    };
    const choose = async (ev) => {
      menuOpened.value = true;
      const [antennas, lists] = await Promise.all([misskeyApi("antennas/list"), misskeyApi("users/lists/list")]);
      const antennaItems = antennas.map((antenna) => ({
        text: antenna.name,
        icon: "ti ti-antenna",
        action: () => {
          widgetProps.antenna = antenna;
          setSrc("antenna");
        }
      }));
      const listItems = lists.map((list) => ({
        text: list.name,
        icon: "ti ti-list",
        action: () => {
          widgetProps.list = list;
          setSrc("list");
        }
      }));
      const menuItems = [];
      menuItems.push(...availableBasicTimelines().map((tl) => ({
        text: i18n.ts._timelines[tl],
        icon: basicTimelineIconClass(tl),
        action: () => {
          setSrc(tl);
        }
      })));
      if (antennaItems.length > 0) {
        menuItems.push({ type: "divider" });
        menuItems.push(...antennaItems);
      }
      if (listItems.length > 0) {
        menuItems.push({ type: "divider" });
        menuItems.push(...listItems);
      }
      os.popupMenu(menuItems, ev.currentTarget ?? ev.target).then(() => {
        menuOpened.value = false;
      });
    };
    __expose({
      name,
      configure,
      id: props.widget ? props.widget.id : null
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkContainer, {
        showHeader: _unref(widgetProps).showHeader,
        style: _normalizeStyle(`height: ${_unref(widgetProps).height}px;`),
        scrollable: true,
        "data-cy-mkw-timeline": "",
        class: "mkw-timeline"
      }, {
        icon: _withCtx(() => [_unref(isBasicTimeline)(_unref(widgetProps).src) ? (_openBlock(), _createElementBlock(
          "i",
          {
            key: 0,
            class: _normalizeClass(_unref(basicTimelineIconClass)(_unref(widgetProps).src))
          },
          null,
          2
          /* CLASS */
        )) : _unref(widgetProps).src === "list" ? (_openBlock(), _createElementBlock("i", {
          key: 1,
          class: "ti ti-list"
        })) : _unref(widgetProps).src === "antenna" ? (_openBlock(), _createElementBlock("i", {
          key: 2,
          class: "ti ti-antenna"
        })) : _createCommentVNode("v-if", true)]),
        header: _withCtx(() => [_createElementVNode("button", {
          class: "_button",
          onClick: choose
        }, [_createElementVNode(
          "span",
          null,
          _toDisplayString(headerTitle.value),
          1
          /* TEXT */
        ), _createElementVNode(
          "i",
          {
            class: _normalizeClass(menuOpened.value ? "ti ti-chevron-up" : "ti ti-chevron-down"),
            style: "margin-left: 8px;"
          },
          null,
          2
          /* CLASS */
        )])]),
        default: _withCtx(() => [_unref(isBasicTimeline)(_unref(widgetProps).src) && !_unref(isAvailableBasicTimeline)(_unref(widgetProps).src) ? (_openBlock(), _createElementBlock(
          "div",
          {
            key: 0,
            class: _normalizeClass(_ctx.$style.disabled)
          },
          [_createElementVNode(
            "p",
            { class: _normalizeClass(_ctx.$style.disabledTitle) },
            [_hoisted_1, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts._disabledTimeline.title),
              1
              /* TEXT */
            )],
            2
            /* CLASS */
          ), _createElementVNode(
            "p",
            { class: _normalizeClass(_ctx.$style.disabledDescription) },
            _toDisplayString(_unref(i18n).ts._disabledTimeline.description),
            3
            /* TEXT, CLASS */
          )],
          2
          /* CLASS */
        )) : (_openBlock(), _createElementBlock("div", { key: 1 }, [_createVNode(MkStreamingNotesTimeline, {
          key: _unref(widgetProps).src === "list" ? `list:${_unref(widgetProps).list?.id}` : _unref(widgetProps).src === "antenna" ? `antenna:${_unref(widgetProps).antenna?.id}` : _unref(widgetProps).src,
          src: _unref(widgetProps).src,
          list: _unref(widgetProps).list ? _unref(widgetProps).list.id : undefined,
          antenna: _unref(widgetProps).antenna ? _unref(widgetProps).antenna.id : undefined
        }, null, 8, [
          "src",
          "list",
          "antenna"
        ])]))]),
        _: 2
      }, 1036, ["showHeader", "scrollable"]);
    };
  }
};
