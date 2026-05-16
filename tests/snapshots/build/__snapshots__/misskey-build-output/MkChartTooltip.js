import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx } from "vue";
import MkTooltip from "./MkTooltip.vue";
export default {
  __name: "MkChartTooltip",
  props: {
    showing: {
      type: Boolean,
      required: true
    },
    x: {
      type: Number,
      required: true
    },
    y: {
      type: Number,
      required: true
    },
    title: {
      type: [String, null],
      required: false
    },
    series: {
      type: [Array, null],
      required: false
    }
  },
  emits: ["closed"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkTooltip, {
        ref: "tooltip",
        showing: __props.showing,
        x: __props.x,
        y: __props.y,
        maxWidth: 340,
        direction: "top",
        innerMargin: 16,
        onClosed: _cache[0] || (_cache[0] = ($event) => emit("closed"))
      }, {
        default: _withCtx(() => [__props.title || __props.series ? (_openBlock(), _createElementBlock("div", { key: 0 }, [__props.title ? (_openBlock(), _createElementBlock(
          "div",
          {
            key: 0,
            class: _normalizeClass(_ctx.$style.title)
          },
          _toDisplayString(__props.title),
          3
          /* TEXT, CLASS */
        )) : _createCommentVNode("v-if", true), __props.series ? (_openBlock(), _createElementBlock(
          _Fragment,
          { key: 0 },
          [(_openBlock(true), _createElementBlock(
            _Fragment,
            null,
            _renderList(__props.series, (x) => {
              return _openBlock(), _createElementBlock("div", null, [_createElementVNode(
                "span",
                {
                  class: _normalizeClass(_ctx.$style.color),
                  style: _normalizeStyle({
                    background: x.backgroundColor,
                    borderColor: x.borderColor
                  })
                },
                null,
                6
                /* CLASS, STYLE */
              ), _createElementVNode(
                "span",
                null,
                _toDisplayString(x.text),
                1
                /* TEXT */
              )]);
            }),
            256
            /* UNKEYED_FRAGMENT */
          ))],
          64
          /* STABLE_FRAGMENT */
        )) : _createCommentVNode("v-if", true)])) : _createCommentVNode("v-if", true)]),
        _: 2
      }, 1032, [
        "showing",
        "x",
        "y",
        "maxWidth",
        "direction",
        "innerMargin"
      ]);
    };
  }
};
