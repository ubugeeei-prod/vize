import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, renderSlot as _renderSlot, mergeProps as _mergeProps, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue";
export default {
  __name: "link",
  props: {
    to: {
      type: String,
      required: false
    },
    active: {
      type: Boolean,
      required: false
    },
    external: {
      type: Boolean,
      required: false
    },
    behavior: {
      type: [String, null],
      required: false
    },
    inline: {
      type: Boolean,
      required: false
    }
  },
  setup(__props) {
    return (_ctx, _cache) => {
      const _component_MkCondensedLine = _resolveComponent("MkCondensedLine");
      return _openBlock(), _createBlock(
        _resolveDynamicComponent(__props.to ? "div" : "button"),
        { class: _normalizeClass([_ctx.$style.root, {
          [_ctx.$style.inline]: __props.inline,
          "_button": !__props.to
        }]) },
        {
          default: _withCtx(() => [_createVNode(
            _resolveDynamicComponent(__props.to ? __props.external ? "a" : "MkA" : "div"),
            _mergeProps(__props.to ? __props.external ? {
              href: __props.to,
              target: "_blank"
            } : {
              to: __props.to,
              behavior: __props.behavior
            } : {}, { class: ["_button", [_ctx.$style.main, { [_ctx.$style.active]: __props.active }]] }),
            {
              default: _withCtx(() => [
                _createElementVNode(
                  "span",
                  { class: _normalizeClass(_ctx.$style.icon) },
                  [_renderSlot(_ctx.$slots, "icon")],
                  2
                  /* CLASS */
                ),
                _createElementVNode(
                  "div",
                  { class: _normalizeClass(_ctx.$style.headerText) },
                  [_createElementVNode("div", null, [_createVNode(_component_MkCondensedLine, { minScale: 2 / 3 }, {
                    default: _withCtx(() => [_renderSlot(_ctx.$slots, "default")]),
                    _: 3
                  }, 1032, ["minScale"])])],
                  2
                  /* CLASS */
                ),
                _createElementVNode(
                  "span",
                  { class: _normalizeClass(_ctx.$style.suffix) },
                  [_createElementVNode(
                    "span",
                    { class: _normalizeClass(_ctx.$style.suffixText) },
                    [_renderSlot(_ctx.$slots, "suffix")],
                    2
                    /* CLASS */
                  ), _createElementVNode(
                    "i",
                    { class: _normalizeClass(__props.to && __props.external ? "ti ti-external-link" : "ti ti-chevron-right") },
                    null,
                    2
                    /* CLASS */
                  )],
                  2
                  /* CLASS */
                )
              ]),
              _: 3
            },
            1040
            /* FULL_PROPS, DYNAMIC_SLOTS */
          )]),
          _: 3
        },
        1026
        /* CLASS, DYNAMIC_SLOTS */
      );
    };
  }
};
